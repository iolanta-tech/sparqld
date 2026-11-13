"""Macros that read the repository URL from the MkDocs environment."""

from pathlib import Path
from typing import Any

from ruamel.yaml import YAML

from docs_macros.conversation import agent_message, conversation_paths
from docs_macros.locations import (
    CLIENTS_DIR,
    RESULTS_DIR,
    ROOT_DIR,
    github_url,
    repository_url,
)
from docs_macros.sources import source_data
from docs_macros.tree import append_directory


class EnvironmentMacros:
    """Expose project-aware macros to MkDocs."""

    def __init__(self, env: Any) -> None:
        self._env = env

    def source(self, path: str | Path, indent: int = 0, title: str = 'Source') -> str:
        """Render any project file as an example admonition."""
        file_path = (ROOT_DIR / path).resolve()
        if not file_path.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid path outside the project: {path}')
        return source_data(
            file_path,
            Path(path),
            indent,
            title,
            repository_url(self._env),
        )

    def example_data(self, name: str, indent: int = 0, title: str = 'Source') -> str:
        return self.source(Path('docs/examples') / name, indent=indent, title=title)

    def result_data(self, name: str, indent: int = 0, title: str = 'Result') -> str:
        path = RESULTS_DIR / name
        return source_data(
            path,
            Path('docs/results') / name,
            indent,
            title,
            repository_url(self._env),
        )

    def client_data(
        self,
        name: str,
        indent: int = 0,
        title: str = 'Configuration',
    ) -> str:
        path = CLIENTS_DIR / name
        return source_data(
            path,
            Path('docs/reference/clients') / name,
            indent,
            title,
            repository_url(self._env),
        )

    def agent_conversation(self, name: str) -> str:
        prompt_path, response_path = conversation_paths(name)
        repo_url = repository_url(self._env)
        prompt = agent_message('You', 'agent-user', prompt_path, name, repo_url)
        response = agent_message(
            'Codex',
            'agent-codex',
            response_path,
            name,
            repo_url,
        )
        return f'{prompt}\n{response}'

    def directory_tree(self, directory: str | Path) -> str:
        root = (ROOT_DIR / directory).resolve()
        _validate_directory(root, directory)
        return _render_directory_tree(root, repository_url(self._env))


def default_prefix_table() -> str:
    """Render SPARQL prefixes from the canonical default context."""
    return _render_prefix_table(_default_prefixes())


def _default_prefixes() -> list[tuple[str, str]]:
    context_file = ROOT_DIR / 'docs' / 'contexts' / 'v1.yamlld'
    document = YAML(typ='safe').load(context_file.read_text())
    return [
        (prefix, iri)
        for prefix, iri in document['@context'].items()
        if isinstance(iri, str) and iri.endswith(('#', '/'))
    ]


def _render_prefix_table(prefixes: list[tuple[str, str]]) -> str:
    rows = ['| Prefix | Expanded value |', '| --- | --- |']
    rows.extend(_prefix_row(prefix, iri) for prefix, iri in prefixes)
    return '\n'.join(rows)


def _prefix_row(prefix: str, iri: str) -> str:
    return f'| [`{prefix}:`]({iri}) | `{iri}` |'


def _validate_directory(root: Path, directory: str | Path) -> None:
    if not root.is_relative_to(ROOT_DIR):
        raise ValueError(f'Invalid directory path: {directory}')
    if not root.is_dir():
        raise ValueError(f'Directory does not exist: {directory}')


def _render_directory_tree(root: Path, repo_url: str) -> str:
    directory_url = github_url(root.relative_to(ROOT_DIR), repo_url, directory=True)
    lines = [f':material-folder: **[`{root.name}/`]({directory_url})**  ']
    append_directory(lines, root, 0, repo_url)
    return '\n'.join(lines)
