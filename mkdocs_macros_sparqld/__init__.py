"""mkdocs-macros pluglet: build-time sparqld queries for documentation."""

from __future__ import annotations

from pathlib import Path

from . import render
from .server import configure as configure_server
from .server import ensure_endpoint, run_query, stop_server

__all__ = [
    'configure',
    'define_env',
    'ensure_endpoint',
    'on_post_build',
    'run_query',
]


def _extra_config(env) -> dict:
    extra = env.conf.get('extra') or {}
    config = extra.get('sparqld') or {}
    if not isinstance(config, dict):
        raise TypeError('extra.sparqld must be a mapping')
    return config


def _project_dir(env) -> Path:
    project_dir = getattr(env, 'project_dir', None)
    if project_dir is not None:
        return Path(project_dir)
    return Path(env.conf['docs_dir']).resolve().parent


def configure(env) -> Path:
    """Read `extra.sparqld` and configure the shared sparqld server."""
    project_dir = _project_dir(env)
    config = _extra_config(env)
    directory = project_dir / config.get('directory', 'docs')
    binary = config.get('binary', 'sparqld')
    configure_server(
        project_dir=project_dir,
        directory=directory.resolve(),
        binary=binary,
    )
    return project_dir.resolve()


def _project_file(project_dir: Path, relative: str) -> Path:
    path = (project_dir / relative).resolve()
    if not path.is_relative_to(project_dir):
        raise ValueError(f'Invalid path outside the MkDocs project: {relative}')
    if not path.is_file():
        raise ValueError(f'File does not exist: {relative}')
    return path


def _execute(query: str) -> list[dict[str, str]] | bool | str:
    content_type, body = run_query(query)
    return render.parse_result(content_type, body)


class SparqldMacros:
    """Macros and filters bound to one MkDocs environment."""

    def __init__(self, env):
        self._project_dir = configure(env)

    def sparql(self, query):
        """Run a verbatim SPARQL query. SELECT returns a list of binding dicts."""
        return _execute(query)

    def stored_sparql(self, path):
        """Run a `.rq` file relative to the MkDocs project root."""
        return _execute(_project_file(self._project_dir, path).read_text())

    def sparql_table(self, rows):
        """Render SELECT bindings from sparql() / stored_sparql() as a Markdown table."""
        return render.bindings_table(rows)


def define_env(env):
    """Register sparqld documentation macros and filters."""
    macros = SparqldMacros(env)
    env.macro(macros.sparql)
    env.macro(macros.stored_sparql)
    env.filter(macros.sparql_table)


def on_post_build(env):
    """Stop the endpoint used to render live documentation macros."""
    stop_server()
