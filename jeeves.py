"""Project automation commands."""

import sys

import sh
import typer


docs = typer.Typer(no_args_is_help=True)


@docs.command(name='serve')
def _serve():
    """Serve the documentation with MkDocs."""
    sh.Command(sys.executable)('-m', 'mkdocs', 'serve', _fg=True)


def lint():
    """Lint the project."""
    # TODO: cargo fmt & lint
    # TODO: in parallel, python fmt & lint
