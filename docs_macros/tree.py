"""Directory tree macro helpers."""

from pathlib import Path
from types import MappingProxyType

from docs_macros.locations import ROOT_DIR, github_url

EXAMPLE_ICONS = MappingProxyType(
    {
        '.json': ':material-code-json:',
        '.jsonld': ':material-code-json:',
        '.md': ':material-language-markdown:',
        '.rq': ':material-database-search-outline:',
        '.sh': ':material-console:',
        '.yaml': ':simple-yaml:',
        '.yamlld': ':simple-yaml:',
        '.yml': ':simple-yaml:',
    }
)


def directory_entries(directory: Path) -> list[Path]:
    return sorted(
        directory.iterdir(),
        key=lambda entry: (not entry.is_dir(), entry.name.lower()),
    )


def tree_connector(index: int, count: int) -> str:
    if index == count - 1:
        return '└──'
    return '├──'


def folder_line(
    entry: Path,
    relative: Path,
    connector: str,
    padding: str,
    repo_url: str,
) -> str:
    url = github_url(relative, repo_url, directory=True)
    name = entry.name
    return f'{padding}{connector} :material-folder: **[`{name}/`]({url})**  '


def file_line(
    entry: Path,
    relative: Path,
    connector: str,
    padding: str,
    repo_url: str,
) -> str:
    url = github_url(relative, repo_url)
    icon = EXAMPLE_ICONS.get(entry.suffix.lower(), ':material-file-outline:')
    name = entry.name
    return f'{padding}{connector} {icon} [`{name}`]({url})  '


def append_entry(
    lines: list[str],
    entry: Path,
    connector: str,
    depth: int,
    repo_url: str,
) -> None:
    relative = entry.relative_to(ROOT_DIR)
    padding = '&nbsp;&nbsp;&nbsp;&nbsp;' * depth
    if entry.is_dir():
        lines.append(folder_line(entry, relative, connector, padding, repo_url))
        append_directory(lines, entry, depth + 1, repo_url)
        return
    lines.append(file_line(entry, relative, connector, padding, repo_url))


def append_directory(
    lines: list[str],
    directory: Path,
    depth: int,
    repo_url: str,
) -> None:
    entries = directory_entries(directory)
    for index, entry in enumerate(entries):
        connector = tree_connector(index, len(entries))
        append_entry(lines, entry, connector, depth, repo_url)
