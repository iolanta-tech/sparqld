"""Project automation commands."""

import datetime
import json
import sys
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
    _python_module('mkdocs')('serve', _fg=True)


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


def lint():
    """Check Rust and Python sources."""
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
    ruff = _python_module('ruff')
    flake8 = _python_module('flake8')
    ruff('check', PROJECT_ROOT, _cwd=PROJECT_ROOT, _fg=True)
    flake8(PROJECT_ROOT, _cwd=PROJECT_ROOT, _fg=True)


def _cargo_fmt(*, check):
    """Run `cargo fmt`, or `cargo fmt -- --check` when `check` is true."""
    arguments = ['fmt']
    if check:
        arguments.extend(('--', '--check'))
    sh.cargo(*arguments, _cwd=PROJECT_ROOT, _fg=True)


def _python_module(name):
    """Return a command for `python -m <name>`."""
    return sh.Command(sys.executable).bake('-m', name)


def _ruff_format(*, check):
    """Run Ruff format. Writing exits 1 when Ruff changes a file."""
    ruff = _python_module('ruff')
    if check:
        ruff('format', '--check', PROJECT_ROOT, _cwd=PROJECT_ROOT, _fg=True)
        return
    ruff(
        'format',
        '--exit-non-zero-on-format',
        PROJECT_ROOT,
        _cwd=PROJECT_ROOT,
        _fg=True,
    )
