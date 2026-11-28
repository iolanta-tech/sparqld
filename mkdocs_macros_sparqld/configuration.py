"""Configure the build-time sparqld server for a MkDocs project."""

from __future__ import annotations

import json
from contextlib import suppress
from pathlib import Path
from typing import Any

import tomllib

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


def _create_settings(path: Path, directory: str | Path) -> None:
    quoted = json.dumps(str(directory), ensure_ascii=False).replace('\x7f', r'\u007f')
    with (
        suppress(FileExistsError),
        path.open('x', encoding='utf-8') as settings,
    ):
        settings.write(f'directory = {quoted}\n')


def _docs_fallback(path: Path) -> Path | None:
    # Let sparqld report errors in the file.
    with (
        suppress(OSError, UnicodeError, tomllib.TOMLDecodeError),
        path.open('rb') as settings,
    ):
        if 'directory' not in tomllib.load(settings):
            return path.parent / 'docs'
    return None


def configure(env: Any) -> Path:
    """Read `extra.sparqld` and configure the shared sparqld server."""
    project_dir = _project_dir(env).resolve()
    config = _extra_config(env)
    override = config.get('directory')
    settings = project_dir / 'sparqld.toml'
    _create_settings(settings, 'docs' if override is None else Path(override))
    directory = _docs_fallback(settings) if override is None else project_dir / override
    configure_server(
        project_dir=project_dir,
        directory=directory,
        binary=config.get('binary', 'sparqld'),
    )
    return project_dir
