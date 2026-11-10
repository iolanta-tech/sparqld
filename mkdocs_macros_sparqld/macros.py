"""Register query macros and filters for a MkDocs environment."""

from __future__ import annotations

from pathlib import Path
from typing import Any
from urllib.parse import urlparse

from mkdocs_macros_sparqld import render
from mkdocs_macros_sparqld.configuration import configure
from mkdocs_macros_sparqld.server import (
    ensure_endpoint,
    run_query,
    run_query_in,
    stop_server,
)
from mkdocs_macros_sparqld.table import SelectBindings

MAX_SELECT_ROWS = 10

__all__ = [
    'configure',
    'define_env',
    'ensure_endpoint',
    'on_post_build',
    'run_query',
    'run_query_in',
]


def _project_file(project_dir: Path, relative: str) -> Path:
    path = (project_dir / relative).resolve()
    if not path.is_relative_to(project_dir):
        raise ValueError(f'Invalid path outside the MkDocs project: {relative}')
    if not path.is_file():
        raise ValueError(f'File does not exist: {relative}')
    return path


def _execute(query: str, source: str) -> render.QueryOutput:
    content_type, body = run_query(query)
    query_output = render.parse_result(content_type, body)
    if (
        isinstance(query_output, SelectBindings)
        and len(query_output.rows) > MAX_SELECT_ROWS
    ):
        message = ' '.join(
            (
                f'{source} returned {len(query_output.rows)} SELECT rows;',
                f'the maximum is {MAX_SELECT_ROWS}.',
                'Narrow the query to the resources the example demonstrates.',
            )
        )
        raise ValueError(message)
    return query_output


class SparqldMacros:
    """Macros and filters bound to one MkDocs environment."""

    def __init__(self, env: Any) -> None:
        self._project_dir = configure(env)
        port = urlparse(ensure_endpoint()).port
        if port is None:
            raise RuntimeError('sparqld endpoint does not include a port')
        env.variables['sparqld_port'] = port

    def sparql(self, query: str) -> render.QueryOutput:
        """Run a verbatim SPARQL query. SELECT returns bindings for sparql_table."""
        return _execute(query, 'Inline SPARQL example')

    def stored_sparql(self, path: str) -> render.QueryOutput:
        """Run a `.rq` file relative to the MkDocs project root."""
        return _execute(_project_file(self._project_dir, path).read_text(), path)

    def sparql_table(
        self,
        rows: SelectBindings | list[dict[str, str]],
    ) -> str:
        """Render SELECT bindings from sparql() / stored_sparql() as a Markdown table."""
        return render.bindings_table(rows)


def define_env(env: Any) -> None:
    """Register sparqld documentation macros and filters."""
    macros = SparqldMacros(env)
    env.macro(macros.sparql)
    env.macro(macros.stored_sparql)
    env.filter(macros.sparql_table)


def on_post_build(env: Any) -> None:
    """Stop the endpoint used to render live documentation macros."""
    stop_server()
