"""Changed-line lookup for `j lint --diff`."""

import re
from pathlib import Path

import sh

from jeeves.repository import PROJECT_ROOT, SH_CWD

_HUNK_HEADER = re.compile(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@')
_LOCATED_FINDING = re.compile(r'^(?P<path>.+?):(?P<line>\d+):\d+:')
_FORMAT_DIFF = re.compile(r'^Diff in (?P<path>.+?):(?P<line>\d+):')
_WOULD_REFORMAT = re.compile(r'^Would reformat: (?P<path>.+)$')


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


def diff_lines(diff):
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


def opened_path(row):
    path = row.removeprefix('+++ b/')
    if path == '/dev/null':
        return None
    return path


def record_hunk(lines, current, row):
    header = _HUNK_HEADER.match(row)
    if header is None:
        return
    start = int(header.group(1))
    count = int(header.group(2) or '1')
    if count == 0:
        return
    add_lines(lines, current, start, start + count)


def add_lines(lines, path, start, stop):
    """Record line numbers `start` through `stop` for `path`."""
    numbered = lines.setdefault(path, set())
    numbered.update(range(start, stop))


def file_line_count(relative):
    """Return the number of lines in a working-tree file."""
    return len((PROJECT_ROOT / relative).read_text().splitlines())


def relative_path(path):
    """Return a repository-relative posix path."""
    candidate = Path(path)
    if not candidate.is_absolute():
        return candidate.as_posix()
    try:
        return candidate.resolve().relative_to(PROJECT_ROOT.resolve()).as_posix()
    except ValueError:
        return candidate.as_posix()


def line_is_modified(path, line, modified):
    """Return whether `line` of `path` is in the git diff."""
    return int(line) in modified.get(relative_path(path), ())


def text_findings(output, modified):
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


def keep_finding(selected, include_following, row, modified):
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


def located_state(selected, row, modified):
    located = _LOCATED_FINDING.match(row)
    if located is None:
        return None
    if line_is_modified(located['path'], located['line'], modified):
        selected.append(row)
    return False


def format_diff_state(selected, row, modified):
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


def reformat_state(selected, row, modified):
    reformatted = _WOULD_REFORMAT.match(row)
    if reformatted is None:
        return None
    if modified.get(relative_path(reformatted['path'])):
        selected.append(row)
    return False
