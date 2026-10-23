"""Matplotlib drawing for the language-line history chart."""

import datetime

from matplotlib import dates as matplotlib_dates
from matplotlib.axes import Axes


def chart_dates(points: list[dict[str, int]]) -> list[datetime.datetime]:
    return [
        datetime.datetime.fromtimestamp(point['timestamp'], tz=datetime.UTC)
        for point in points
    ]


def language_colors() -> dict[str, str]:
    return {
        'Rust': '#7e57c2',
        'Python': '#00897b',
        'Markdown': '#ef6c00',
    }


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
