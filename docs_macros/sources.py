"""Source-file admonitions."""

from pathlib import Path
from types import MappingProxyType

from docs_macros.locations import github_url, indent_block

EXAMPLE_SYNTAXES = MappingProxyType(
    {
        '.json': 'json',
        '.jsonld': 'json',
        '.md': 'markdown',
        '.mjs': 'javascript',
        '.py': 'python',
        '.rq': 'sparql',
        '.sh': 'console',
        '.toml': 'toml',
        '.yaml': 'yaml',
        '.yamlld': 'yaml',
        '.yml': 'yaml',
    }
)


def source_heading(
    path: Path,
    repository_path: str | Path,
    title: str,
    repo_url: str,
) -> str:
    github = github_url(repository_path, repo_url)
    name = path.name
    return f'{title}<span class="example-source-link" markdown>:fontawesome-brands-github: [`{name}`]({github})</span>'


def source_data(
    path: Path,
    repository_path: str | Path,
    indent: int,
    title: str,
    repo_url: str,
) -> str:
    if not path.is_file():
        raise ValueError(f'Documentation source does not exist: {path}')
    source = path.read_text().rstrip('\n')
    syntax = EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    heading = source_heading(path, repository_path, title, repo_url)
    body = f'```{syntax}\n{source}\n```'
    rendered = f'!!! example "{heading}"\n\n{indent_block(body, 4)}\n'
    return indent_block(rendered, indent)
