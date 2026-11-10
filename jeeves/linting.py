"""Lint commands."""

from collections.abc import Sequence
from typing import Annotated

import sh
import typer
from typer._click.parser import _Option, _OptionParser, _ParsingState

from jeeves.clippy import clippy_findings
from jeeves.debtmap import lint_debtmap
from jeeves.diff import modified_lines
from jeeves.findings import text_findings
from jeeves.formatting import (
    cargo_fmt,
    cargo_fmt_command,
    ruff_format,
    ruff_format_command,
)
from jeeves.repository import PROJECT_ROOT, SH_CWD
from jeeves.shell import command_output, in_parallel

_DIFF_OPTION_VALUE = _OptionParser._get_value_from_state

# Reader-facing prose; keep in sync with the Vale job in `.github/workflows/ci.yml`.
VALE_PATHS = (
    'README.md',
    'docs/index.md',
    'docs/quickstart',
    'docs/reference',
    'docs/project',
    'docs/conversations',
)


def _diff_option_value(
    parser: _OptionParser,
    option_name: str,
    option: _Option,
    state: _ParsingState,
) -> str | Sequence[str]:
    """Allow `--diff` alone, and `--diff REF`."""
    if option.dest != 'diff':
        return _DIFF_OPTION_VALUE(parser, option_name, option, state)
    if state.rargs and not state.rargs[0].startswith('-'):
        return state.rargs.pop(0)
    return 'HEAD'


_OptionParser._get_value_from_state = _diff_option_value  # ty: ignore[invalid-assignment]


def lint(
    diff: Annotated[
        str | None,
        typer.Option(
            '--diff',
            help='Report findings on lines changed from HEAD, or from a git ref.',
        ),
    ] = None,
) -> None:
    """Check Rust, Python, Vale prose, then debtmap density."""
    if diff is not None:
        lint_diff(diff)
        return
    in_parallel(lint_rust, lint_python)
    lint_vale()
    lint_debtmap()


def lint_rust():
    """Check Rust formatting and run Clippy."""
    cargo_fmt(check=True)
    sh.cargo('clippy', _cwd=SH_CWD, _fg=True)


def lint_python():
    """Check Python formatting, Ruff, Flake8, and ty."""
    ruff_format(check=True)
    sh.ruff.check(str(PROJECT_ROOT), _cwd=SH_CWD, _fg=True)
    sh.ty.check(_fg=True)
    sh.flake8(str(PROJECT_ROOT), _cwd=SH_CWD, _fg=True)


def lint_vale() -> None:
    """Lint reader-facing Markdown; fail on Vale error-level alerts only."""
    sh.vale(
        '--no-global',
        '--minAlertLevel=error',
        *VALE_PATHS,
        _cwd=SH_CWD,
        _fg=True,
    )


def lint_diff(base: str) -> None:
    """Report lint findings on lines changed from `base`."""
    modified = modified_lines(base)
    findings = text_findings(check_output(), modified)
    findings.extend(clippy_findings(modified))
    if findings:
        typer.echo('\n'.join(findings))
        raise typer.Exit(code=1)


def check_output():
    """Return formatter, Python, and Vale linter output for the whole tree."""
    chunks = (
        command_output(ruff_format_command(check=True), str(PROJECT_ROOT)),
        command_output(
            sh.ruff.check.bake('--output-format', 'concise'),
            str(PROJECT_ROOT),
        ),
        command_output(sh.flake8, str(PROJECT_ROOT)),
        command_output(sh.ty.check.bake('--output-format', 'concise')),
        command_output(cargo_fmt_command(check=True)),
        command_output(
            sh.vale.bake(
                '--no-global',
                '--output=line',
                '--minAlertLevel=error',
            ),
            *VALE_PATHS,
        ),
    )
    return '\n'.join(chunk for chunk in chunks if str(chunk).strip())
