"""Rust and Python formatters."""

import sh
import typer

from jeeves.repository import PROJECT_ROOT, SH_CWD
from jeeves.shell import in_parallel


def fmt():
    """Format Rust and Python sources."""
    if in_parallel(fmt_rust, fmt_python):
        raise typer.Exit(code=1)


def fmt_rust():
    """Format Rust sources. Return whether `cargo fmt` changed files."""
    try:
        cargo_fmt(check=True)
    except sh.ErrorReturnCode_1:
        cargo_fmt(check=False)
        return True
    return False


def fmt_python():
    """Format Python sources. Return whether Ruff changed files."""
    try:
        ruff_format(check=False)
    except sh.ErrorReturnCode_1:
        return True
    return False


def cargo_fmt(*, check: bool) -> None:
    """Run `cargo fmt`, or `cargo fmt -- --check` when `check` is true."""
    cargo_fmt_command(check=check)(_cwd=SH_CWD, _fg=True)


def cargo_fmt_command(*, check: bool):
    """Return `cargo fmt`, baking rustfmt's `--check` when requested."""
    command = sh.cargo.fmt
    if check:
        return command.bake('--', '--check')
    return command


def ruff_format(*, check: bool) -> None:
    """Run Ruff format. Writing exits 1 when Ruff changes a file."""
    ruff_format_command(check=check)(
        str(PROJECT_ROOT),
        _cwd=SH_CWD,
        _fg=True,
    )


def ruff_format_command(*, check: bool):
    """Return `ruff format`, baking `--check` or the write-and-exit flag."""
    command = sh.ruff.format
    if check:
        return command.bake('--check')
    return command.bake('--exit-non-zero-on-format')
