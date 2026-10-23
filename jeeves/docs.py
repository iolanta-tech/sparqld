"""Documentation commands."""

import sh
import typer

from jeeves.history import (
    LANGUAGE_HISTORY_PATH,
    history_points,
    render_language_history,
)
from jeeves.repository import PROJECT_ROOT

docs = typer.Typer(no_args_is_help=True)


@docs.command()
def serve():
    """Serve the documentation with MkDocs."""
    sh.mkdocs.serve(_fg=True)


@docs.command()
def language_history():
    """Generate the language-line chart for the Project page."""
    points = history_points()
    render_language_history(points)
    written = LANGUAGE_HISTORY_PATH.relative_to(PROJECT_ROOT)
    typer.echo(f'Wrote {written}')
