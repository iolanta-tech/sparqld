"""Format a clean tree, then lint it."""

import sh
import typer

from jeeves.formatting import fmt_python, fmt_rust
from jeeves.linting import lint
from jeeves.repository import SH_CWD
from jeeves.shell import in_parallel


def evolve() -> None:
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
