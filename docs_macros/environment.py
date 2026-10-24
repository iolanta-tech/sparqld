"""Macros that read the repository URL from the MkDocs environment."""

from pathlib import Path
from typing import Any

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
    """Macros that read the repository URL from the MkDocs environment."""

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
        if not root.is_relative_to(ROOT_DIR):
            raise ValueError(f'Invalid directory path: {directory}')
        if not root.is_dir():
            raise ValueError(f'Directory does not exist: {directory}')

        repo_url = repository_url(self._env)
        root_relative = root.relative_to(ROOT_DIR)
        root_url = github_url(root_relative, repo_url, directory=True)
        lines = [
            f':material-folder: **[`{root.name}/`]({root_url})**  ',
        ]

        append_directory(lines, root, 0, repo_url)
        return '\n'.join(lines)
