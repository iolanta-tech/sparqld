"""Language-line history chart for the Project page."""

import datetime
import json
import tempfile
from pathlib import Path
from types import MappingProxyType

import sh
from matplotlib import dates as matplotlib_dates
from matplotlib import pyplot
from matplotlib.axes import Axes

from jeeves.repository import PROJECT_ROOT, SH_CWD

LANGUAGE_HISTORY_PATH = PROJECT_ROOT / 'docs/project/language-history.svg'
LANGUAGE_MEASURES = MappingProxyType(
    {
        'Rust': 'code',
        'Python': 'code',
        'Markdown': 'comments',
    }
)


def main_history():
    """Return first-parent main commits from oldest to newest."""
    return sh.git(
        'rev-list', 'main', first_parent=True, reverse=True, _cwd=SH_CWD
    ).splitlines()


def commit_timestamp(commit: str) -> int:
    """Return a commit's author timestamp."""
    return int(
        sh.git.show(
            commit,
            no_patch=True,
            format='%at',
            _cwd=SH_CWD,
            _tty_out=False,
        ).strip()
    )


def language_line_counts(directory: Path) -> dict[str, int]:
    """Return the selected Tokei line count for each charted language."""
    report = json.loads(sh.tokei(directory, output='json'))
    return {
        language: report.get(language, {}).get(measure, 0)
        for language, measure in LANGUAGE_MEASURES.items()
    }


def chart_dates(points: list[dict[str, int]]) -> list[datetime.datetime]:
    return [
        datetime.datetime.fromtimestamp(point['timestamp'], tz=datetime.UTC)
        for point in points
    ]


def language_colors():
    return {
        'Rust': '#7e57c2',
        'Python': '#00897b',
        'Markdown': '#ef6c00',
    }


def draw_history(axes: Axes, points: list[dict[str, int]]) -> None:
    dates = chart_dates(points)
    colors = language_colors()
    axes.stackplot(
        matplotlib_dates.date2num(dates),
        *([point[language] for point in points] for language in colors),
        colors=colors.values(),
        labels=colors,
    )
    format_history_axes(axes)


def format_history_axes(axes: Axes) -> None:
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


def render_language_history(points: list[dict[str, int]]) -> None:
    """Render the language-line history chart with Matplotlib."""
    if not points:
        raise RuntimeError('main has no commits to chart.')
    figure, axes = pyplot.subplots(figsize=(12, 5), layout='constrained')
    draw_history(axes, points)
    figure.savefig(
        LANGUAGE_HISTORY_PATH,
        format='svg',
        metadata={'Title': 'Rust, Python, and Markdown lines on main'},
    )
    pyplot.close(figure)


def history_point(archive: Path, index: int, commit: str) -> dict[str, int]:
    snapshot = archive.parent / str(index)
    snapshot.mkdir()
    sh.git.archive(commit, format='tar', _cwd=SH_CWD, _out=str(archive))
    sh.tar(extract=True, file=str(archive), directory=str(snapshot))
    return {
        'timestamp': commit_timestamp(commit),
        **language_line_counts(snapshot),
    }


def history_points():
    with tempfile.TemporaryDirectory(prefix='sparqld-language-history-') as temporary:
        archive = Path(temporary) / 'snapshot.tar'
        return [
            history_point(archive, index, commit)
            for index, commit in enumerate(main_history())
        ]
