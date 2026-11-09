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


def code_fence(source: str, syntax: str) -> str:
    """Wrap source in a fence longer than any backtick run it contains."""
    longest = 0
    current = 0
    for character in source:
        if character == '`':
            current += 1
            longest = max(longest, current)
        else:
            current = 0
    fence = '`' * max(3, longest + 1)
    return f'{fence}{syntax}\n{source}\n{fence}'


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
    body = code_fence(source, syntax)
    rendered = f'!!! example "{heading}"\n\n{indent_block(body, 4)}\n'
    return indent_block(rendered, indent)
