"""Configure the build-time sparqld server for a MkDocs project."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from mkdocs_macros_sparqld.server import configure as configure_server


def _extra_config(env: Any) -> dict:
    extra = env.conf.get('extra') or {}
    config = extra.get('sparqld') or {}
    if not isinstance(config, dict):
        raise TypeError('extra.sparqld must be a mapping')
    return config


def _project_dir(env: Any) -> Path:
    project_dir = getattr(env, 'project_dir', None)
    if project_dir is not None:
        return Path(project_dir)
    return Path(env.conf['docs_dir']).resolve().parent


def configure(env: Any) -> Path:
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
