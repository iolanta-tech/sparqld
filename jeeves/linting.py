"""Lint and evolve commands."""

import sh
import typer

from jeeves.clippy import clippy_findings
from jeeves.diff import modified_lines, text_findings
from jeeves.formatting import (
    cargo_fmt,
    cargo_fmt_command,
    fmt_python,
    fmt_rust,
    ruff_format,
    ruff_format_command,
)
from jeeves.repository import PROJECT_ROOT, SH_CWD
from jeeves.shell import command_output, in_parallel


def lint(diff: bool = False):
    """Check Rust and Python sources."""
    if diff:
        lint_diff()
        return
    in_parallel(lint_rust, lint_python)


def evolve():
    """Format a clean tree, then lint it."""
    status = sh.git(
        'status',
        '--porcelain',
        _cwd=SH_CWD,
        _tty_out=False,
    )
    if str(status).strip():
        typer.echo('Working tree has uncommitted changes.')
        typer.echo(status, nl=False)
        raise typer.Exit(code=1)
    if in_parallel(fmt_rust, fmt_python):
        typer.echo('Formatting changed files.')
        raise typer.Exit(code=1)
    lint()


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


def lint_diff():
    """Report lint findings that sit on lines changed relative to HEAD."""
    modified = modified_lines()
    findings = text_findings(check_output(), modified)
    findings.extend(clippy_findings(modified))
    if findings:
        typer.echo('\n'.join(findings))
        raise typer.Exit(code=1)


def check_output():
    """Return formatter and Python linter output for the whole tree."""
    chunks = (
        command_output(ruff_format_command(check=True), str(PROJECT_ROOT)),
        command_output(
            sh.ruff.check.bake('--output-format', 'concise'),
            str(PROJECT_ROOT),
        ),
        command_output(sh.flake8, str(PROJECT_ROOT)),
        command_output(sh.ty.check.bake('--output-format', 'concise')),
        command_output(cargo_fmt_command(check=True)),
    )
    return '\n'.join(chunk for chunk in chunks if str(chunk).strip())
