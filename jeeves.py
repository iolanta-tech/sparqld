"""Project automation commands."""

import datetime
import json
import sys
import tempfile
from pathlib import Path

import matplotlib.dates as matplotlib_dates
from matplotlib import pyplot
import sh
import typer


docs = typer.Typer(no_args_is_help=True)
PROJECT_ROOT = Path(__file__).parent
LANGUAGE_HISTORY_PATH = PROJECT_ROOT / 'docs/project/language-history.svg'
LANGUAGE_MEASURES = {
    'Rust': 'code',
    'Python': 'code',
    'Markdown': 'comments',
}


@docs.command(name='serve')
def _serve():
    """Serve the documentation with MkDocs."""
    sh.Command(sys.executable)('-m', 'mkdocs', 'serve', _fg=True)


def _main_history():
    """Return first-parent main commits from oldest to newest."""
    return sh.git(
        'rev-list',
        'main', first_parent=True, reverse=True, _cwd=PROJECT_ROOT
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


def lint():
    """Autoformat & lint."""
    # TODO: cargo fmt & lint
    # TODO: in parallel, python fmt & lint
