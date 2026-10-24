"""Changed-line lookup for `j lint --diff`."""

import re

import sh

from jeeves.repository import PROJECT_ROOT, SH_CWD

LineIndex = dict[str, set[int]]
_HUNK_HEADER = re.compile(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@')


def modified_lines():
    """Return changed line numbers for each path in the working tree."""
    lines = diff_lines(
        str(
            sh.git(
                'diff',
                'HEAD',
                '-U0',
                '--no-color',
                _cwd=SH_CWD,
                _tty_out=False,
            )
        )
    )
    untracked = str(
        sh.git(
            'ls-files',
            '--others',
            '--exclude-standard',
            _cwd=SH_CWD,
            _tty_out=False,
        )
    )
    for relative in untracked.splitlines():
        counted = file_line_count(relative)
        add_lines(lines, relative, 1, counted + 1)
    return lines


def diff_lines(diff: str) -> dict[str, set[int]]:
    """Collect new-file line numbers from a zero-context git diff."""
    lines = {}
    current = None
    for row in diff.splitlines():
        if row.startswith('+++ b/'):
            current = opened_path(row)
            continue
        if current is None:
            continue
        record_hunk(lines, current, row)
    return lines


def opened_path(row: str) -> str | None:
    path = row.removeprefix('+++ b/')
    if path == '/dev/null':
        return None
    return path


def record_hunk(lines: dict[str, set[int]], current: str, row: str) -> None:
    header = _HUNK_HEADER.match(row)
    if header is None:
        return
    start = int(header.group(1))
    count = int(header.group(2) or '1')
    if count == 0:
        return
    add_lines(lines, current, start, start + count)


def add_lines(lines: LineIndex, path: str, start: int, stop: int) -> None:
    """Record line numbers `start` through `stop` for `path`."""
    numbered = lines.setdefault(path, set())
    numbered.update(range(start, stop))


def file_line_count(relative: str) -> int:
    """Return the number of lines in a working-tree file."""
    return len((PROJECT_ROOT / relative).read_text().splitlines())
