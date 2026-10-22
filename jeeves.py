"""Project automation commands."""

import datetime
import json
import re
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from types import MappingProxyType

import sh
import typer
from matplotlib import dates as matplotlib_dates
from matplotlib import pyplot

docs = typer.Typer(no_args_is_help=True)
PROJECT_ROOT = Path(__file__).parent
LANGUAGE_HISTORY_PATH = PROJECT_ROOT / 'docs/project/language-history.svg'
LANGUAGE_MEASURES = MappingProxyType(
    {
        'Rust': 'code',
        'Python': 'code',
        'Markdown': 'comments',
    }
)


@docs.command(name='serve')
def _serve():
    """Serve the documentation with MkDocs."""
    sh.mkdocs.serve(_fg=True)


def _main_history():
    """Return first-parent main commits from oldest to newest."""
    return sh.git(
        'rev-list', 'main', first_parent=True, reverse=True, _cwd=PROJECT_ROOT
    ).splitlines()


def _commit_timestamp(commit):
    """Return a commit's author timestamp."""
    return int(
        sh.git.show(
            commit,
            no_patch=True,
            format='%at',
            _cwd=PROJECT_ROOT,
            _tty_out=False,
        ).strip()
    )


def _language_line_counts(directory):
    """Return the selected Tokei line count for each charted language."""
    report = json.loads(sh.tokei(directory, output='json'))
    return {
        language: report.get(language, {}).get(measure, 0)
        for language, measure in LANGUAGE_MEASURES.items()
    }


def _render_language_history(points):
    """Render the language-line history chart with Matplotlib."""
    if not points:
        raise RuntimeError('main has no commits to chart.')

    dates = [
        datetime.datetime.fromtimestamp(point['timestamp'], tz=datetime.UTC)
        for point in points
    ]
    colors = {
        'Rust': '#7e57c2',
        'Python': '#00897b',
        'Markdown': '#ef6c00',
    }
    figure, axes = pyplot.subplots(figsize=(12, 5), layout='constrained')
    axes.stackplot(
        dates,
        *([point[language] for point in points] for language in colors),
        colors=colors.values(),
        labels=colors,
    )
    locator = matplotlib_dates.AutoDateLocator()
    axes.xaxis.set_major_locator(locator)
    axes.xaxis.set_major_formatter(matplotlib_dates.ConciseDateFormatter(locator))
    axes.set(
        title='Lines by language',
        xlabel='First-parent commits on main',
        ylabel='Lines',
    )
    axes.grid(axis='y', color='#d7d7d7')
    axes.legend(frameon=False, ncols=2)
    axes.spines[['top', 'right']].set_visible(False)
    figure.savefig(
        LANGUAGE_HISTORY_PATH,
        format='svg',
        metadata={'Title': 'Rust, Python, and Markdown lines on main'},
    )
    pyplot.close(figure)


@docs.command(name='language-history')
def _language_history():
    """Generate the language-line chart for the Project page."""
    commits = _main_history()
    points = []
    with tempfile.TemporaryDirectory(prefix='sparqld-language-history-') as temporary:
        archive = Path(temporary) / 'snapshot.tar'
        for index, commit in enumerate(commits):
            snapshot = Path(temporary) / str(index)
            snapshot.mkdir()
            sh.git.archive(commit, format='tar', _cwd=PROJECT_ROOT, _out=archive)
            sh.tar(extract=True, file=archive, directory=snapshot)
            points.append(
                {
                    'timestamp': _commit_timestamp(commit),
                    **_language_line_counts(snapshot),
                }
            )

    _render_language_history(points)
    typer.echo(f'Wrote {LANGUAGE_HISTORY_PATH.relative_to(PROJECT_ROOT)}')


def fmt():
    """Format Rust and Python sources."""
    if _in_parallel(_fmt_rust, _fmt_python):
        raise typer.Exit(code=1)


def lint(diff: bool = False):
    """Check Rust and Python sources."""
    if diff:
        _lint_diff()
        return
    _in_parallel(_lint_rust, _lint_python)


def evolve():
    """Format a clean tree, then lint it."""
    status = sh.git(
        'status',
        '--porcelain',
        _cwd=PROJECT_ROOT,
        _tty_out=False,
    )
    if str(status).strip():
        typer.echo('Working tree has uncommitted changes.')
        typer.echo(status, nl=False)
        raise typer.Exit(code=1)
    if _in_parallel(_fmt_rust, _fmt_python):
        typer.echo('Formatting changed files.')
        raise typer.Exit(code=1)
    lint()


def publish_pluglet():
    """Build a clean pluglet distribution and publish it."""
    poetry = sh.Command(PROJECT_ROOT / '.venv/bin/poetry')
    poetry('build', '--clean', _cwd=PROJECT_ROOT, _fg=True)
    poetry('publish', _cwd=PROJECT_ROOT, _fg=True)


def _in_parallel(rust_task, python_task):
    """Run the Rust and Python tasks. Return whether formatting changed files."""
    with ThreadPoolExecutor(max_workers=2) as executor:
        tasks = (
            executor.submit(rust_task),
            executor.submit(python_task),
        )
        changed = False
        for task in tasks:
            changed = task.result() or changed
        return changed


def _fmt_rust():
    """Format Rust sources. Return whether `cargo fmt` changed files."""
    try:
        _cargo_fmt(check=True)
    except sh.ErrorReturnCode_1:
        _cargo_fmt(check=False)
        return True
    return False


def _fmt_python():
    """Format Python sources. Return whether Ruff changed files."""
    try:
        _ruff_format(check=False)
    except sh.ErrorReturnCode_1:
        return True
    return False


def _lint_rust():
    """Check Rust formatting and run Clippy."""
    _cargo_fmt(check=True)
    sh.cargo('clippy', _cwd=PROJECT_ROOT, _fg=True)


def _lint_python():
    """Check Python formatting, Ruff, and Flake8."""
    _ruff_format(check=True)
    sh.ruff.check(PROJECT_ROOT, _cwd=PROJECT_ROOT, _fg=True)
    sh.flake8(PROJECT_ROOT, _cwd=PROJECT_ROOT, _fg=True)


def _cargo_fmt(*, check):
    """Run `cargo fmt`, or `cargo fmt -- --check` when `check` is true."""
    _cargo_fmt_command(check=check)(_cwd=PROJECT_ROOT, _fg=True)


def _cargo_fmt_command(*, check):
    """Return `cargo fmt`, baking rustfmt's `--check` when requested."""
    command = sh.cargo.fmt
    if check:
        return command.bake('--', '--check')
    return command


def _ruff_format(*, check):
    """Run Ruff format. Writing exits 1 when Ruff changes a file."""
    _ruff_format_command(check=check)(
        PROJECT_ROOT,
        _cwd=PROJECT_ROOT,
        _fg=True,
    )


def _ruff_format_command(*, check):
    """Return `ruff format`, baking `--check` or the write-and-exit flag."""
    command = sh.ruff.format
    if check:
        return command.bake('--check')
    return command.bake('--exit-non-zero-on-format')


_HUNK_HEADER = re.compile(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@')
_LOCATED_FINDING = re.compile(r'^(?P<path>.+?):(?P<line>\d+):\d+:')
_FORMAT_DIFF = re.compile(r'^Diff in (?P<path>.+?):(?P<line>\d+):')
_WOULD_REFORMAT = re.compile(r'^Would reformat: (?P<path>.+)$')


def _lint_diff():
    """Report lint findings that sit on lines changed relative to HEAD."""
    modified = _modified_lines()
    findings = _text_findings(_check_output(), modified)
    findings.extend(_clippy_findings(modified))
    if findings:
        typer.echo('\n'.join(findings))
        raise typer.Exit(code=1)


def _modified_lines():
    """Return changed line numbers for each path in the working tree."""
    lines = _diff_lines(
        str(
            sh.git(
                'diff',
                'HEAD',
                '-U0',
                '--no-color',
                _cwd=PROJECT_ROOT,
                _tty_out=False,
            )
        )
    )
    untracked = str(
        sh.git(
            'ls-files',
            '--others',
            '--exclude-standard',
            _cwd=PROJECT_ROOT,
            _tty_out=False,
        )
    )
    for relative in untracked.splitlines():
        counted = _file_line_count(relative)
        lines.setdefault(relative, set()).update(range(1, counted + 1))
    return lines


def _diff_lines(diff):
    """Collect new-file line numbers from a zero-context git diff."""
    lines = {}
    current = None
    for row in diff.splitlines():
        if row.startswith('+++ b/'):
            current = row.removeprefix('+++ b/')
            if current == '/dev/null':
                current = None
            continue
        if current is None:
            continue
        header = _HUNK_HEADER.match(row)
        if header is None:
            continue
        start = int(header.group(1))
        count = int(header.group(2) or '1')
        if count == 0:
            continue
        lines.setdefault(current, set()).update(range(start, start + count))
    return lines


def _file_line_count(relative):
    """Return the number of lines in a working-tree file."""
    return len((PROJECT_ROOT / relative).read_text().splitlines())


def _relative_path(path):
    """Return a repository-relative posix path."""
    candidate = Path(path)
    if not candidate.is_absolute():
        return candidate.as_posix()
    try:
        return candidate.resolve().relative_to(PROJECT_ROOT.resolve()).as_posix()
    except ValueError:
        return candidate.as_posix()


def _line_is_modified(path, line, modified):
    """Return whether `line` of `path` is in the git diff."""
    return int(line) in modified.get(_relative_path(path), ())


def _output_text(stream):
    """Return a command stream as text."""
    if not stream:
        return ''
    if isinstance(stream, bytes):
        return stream.decode()
    return stream


def _command_output(command, *arguments):
    """Run a command and return its combined output, including failures."""
    try:
        completed = command(*arguments, _cwd=PROJECT_ROOT, _tty_out=False)
    except sh.ErrorReturnCode as error:
        return _output_text(error.stdout) + _output_text(error.stderr)
    return str(completed)


def _check_output():
    """Return formatter and Python linter output for the whole tree."""
    chunks = (
        _command_output(_ruff_format_command(check=True), str(PROJECT_ROOT)),
        _command_output(
            sh.ruff.check.bake('--output-format', 'concise'),
            str(PROJECT_ROOT),
        ),
        _command_output(sh.flake8, str(PROJECT_ROOT)),
        _command_output(_cargo_fmt_command(check=True)),
    )
    return '\n'.join(chunk for chunk in chunks if str(chunk).strip())


def _text_findings(output, modified):
    """Keep text findings whose path and line are in `modified`."""
    selected = []
    include_following = False
    for row in output.splitlines():
        located = _LOCATED_FINDING.match(row)
        formatted = _FORMAT_DIFF.match(row)
        reformatted = _WOULD_REFORMAT.match(row)
        if located:
            include_following = False
            if _line_is_modified(located['path'], located['line'], modified):
                selected.append(row)
            continue
        if formatted:
            include_following = _line_is_modified(
                formatted['path'],
                formatted['line'],
                modified,
            )
            if include_following:
                selected.append(row)
            continue
        if reformatted:
            include_following = False
            if modified.get(_relative_path(reformatted['path'])):
                selected.append(row)
            continue
        if include_following:
            selected.append(row)
    return selected


def _clippy_findings(modified):
    """Return Clippy diagnostics whose primary span is on a changed line."""
    output = _command_output(
        sh.cargo,
        'clippy',
        '--quiet',
        '--message-format=json',
    )
    selected = []
    for row in str(output).splitlines():
        if not row.startswith('{'):
            continue
        rendered = _clippy_row(json.loads(row), modified)
        if rendered:
            selected.append(rendered)
    return selected


def _clippy_row(message, modified):
    """Format one Clippy JSON message when it touches the diff."""
    if message.get('reason') != 'compiler-message':
        return None
    diagnostic = message.get('message') or {}
    if diagnostic.get('level') not in {'warning', 'error'}:
        return None
    spans = diagnostic.get('spans') or []
    primary = next((span for span in spans if span.get('is_primary')), None)
    if primary is None:
        return None
    path = primary.get('file_name') or ''
    line = primary.get('line_start')
    if not _line_is_modified(path, line, modified):
        return None
    column = primary.get('column_start')
    text = diagnostic.get('message')
    return f'{_relative_path(path)}:{line}:{column}: {text}'
