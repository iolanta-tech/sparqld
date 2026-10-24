"""Paths and small text helpers for documentation macros."""

import textwrap
from pathlib import Path
from typing import Any

ROOT_DIR = Path(__file__).resolve().parents[1]
DOCS_DIR = ROOT_DIR / 'docs'
EXAMPLES_DIR = DOCS_DIR / 'examples'
CLIENTS_DIR = DOCS_DIR / 'reference' / 'clients'
CONVERSATIONS_DIR = DOCS_DIR / 'conversations'
QUERIES_DIR = DOCS_DIR / 'queries'
RESULTS_DIR = DOCS_DIR / 'results'
REPO_URL = 'https://github.com/iolanta-tech/sparqld'
CLIENT_COMPATIBILITY_PATH = RESULTS_DIR / 'client-compatibility.yamlld'


def example_path(name: str) -> Path:
    path = (EXAMPLES_DIR / name).resolve()
    if not path.is_relative_to(EXAMPLES_DIR.resolve()):
        raise ValueError(f'Invalid example path: {name}')
    return path


def github_url(
    relative: str | Path,
    repo_url: str,
    directory: bool = False,
) -> str:
    relative = Path(relative).as_posix()
    operation = 'tree' if directory else 'blob'
    return f'{repo_url.rstrip("/")}/{operation}/main/{relative}'


def repository_url(env: Any) -> str:
    return env.conf.get('repo_url') or REPO_URL


def indent_block(body: str, indent: int) -> str:
    return textwrap.indent(body, ' ' * indent, lambda _: True)
