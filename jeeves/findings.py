"""Text diagnostics limited to lines changed relative to HEAD."""

import re
from pathlib import Path

from jeeves.diff import LineIndex
from jeeves.repository import PROJECT_ROOT

_LOCATED_FINDING = re.compile(r'^(?P<path>.+?):(?P<line>\d+):\d+:')
_FORMAT_DIFF = re.compile(r'^Diff in (?P<path>.+?):(?P<line>\d+):')
_WOULD_REFORMAT = re.compile(r'^Would reformat: (?P<path>.+)$')


def relative_path(path: str) -> str:
    """Return a repository-relative posix path."""
    candidate = Path(path)
    if not candidate.is_absolute():
        return candidate.as_posix()
    try:
        return candidate.resolve().relative_to(PROJECT_ROOT.resolve()).as_posix()
    except ValueError:
        return candidate.as_posix()


def line_is_modified(path: str, line: str | int, modified: LineIndex) -> bool:
    """Return whether `line` of `path` is in the git diff."""
    return int(line) in modified.get(relative_path(path), ())


def text_findings(output: str, modified: LineIndex) -> list[str]:
    """Keep text findings whose path and line are in `modified`."""
    selected = []
    include_following = False
    for row in output.splitlines():
        include_following = keep_finding(
            selected,
            include_following,
            row,
            modified,
        )
    return selected


def keep_finding(
    selected: list[str],
    include_following: bool,
    row: str,
    modified: LineIndex,
) -> bool:
    located = located_state(selected, row, modified)
    if located is not None:
        return located
    formatted = format_diff_state(selected, row, modified)
    if formatted is not None:
        return formatted
    reformatted = reformat_state(selected, row, modified)
    if reformatted is not None:
        return reformatted
    if include_following:
        selected.append(row)
    return include_following


def located_state(
    selected: list[str],
    row: str,
    modified: LineIndex,
) -> bool | None:
    located = _LOCATED_FINDING.match(row)
    if located is None:
        return None
    if line_is_modified(located['path'], located['line'], modified):
        selected.append(row)
    return False


def format_diff_state(
    selected: list[str],
    row: str,
    modified: LineIndex,
) -> bool | None:
    formatted = _FORMAT_DIFF.match(row)
    if formatted is None:
        return None
    include_following = line_is_modified(
        formatted['path'],
        formatted['line'],
        modified,
    )
    if include_following:
        selected.append(row)
    return include_following


def reformat_state(
    selected: list[str],
    row: str,
    modified: LineIndex,
) -> bool | None:
    reformatted = _WOULD_REFORMAT.match(row)
    if reformatted is None:
        return None
    if modified.get(relative_path(reformatted['path'])):
        selected.append(row)
    return False
