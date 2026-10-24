"""ADR and example-code macros."""

from datetime import date
from types import MappingProxyType

from docs_macros.locations import example_path, indent_block
from docs_macros.sources import EXAMPLE_SYNTAXES

ADR_STATUS = MappingProxyType(
    {
        'draft': 'Draft',
        'undecided': 'Undecided',
        'decided': 'Decided',
    }
)


_ADR_STATUS_ADMONITION = MappingProxyType(
    {
        'draft': 'note',
        'undecided': 'warning',
        'decided': 'success',
    }
)


_UNIVERSE_STATUS = MappingProxyType(
    {
        'chosen': ('success', 'Chosen'),
        'excluded': ('failure', 'Excluded'),
        'undecided': ('warning', 'Undecided'),
    }
)


def human_date(date_value: str | date | None) -> str:
    if not date_value:
        return ''
    parsed = (
        date.fromisoformat(date_value) if isinstance(date_value, str) else date_value
    )
    return f'{parsed.day} {parsed.strftime("%B %Y")}'


def adr_metadata(value_date: str | date | None, status: str) -> str:
    status_key = str(status).lower()
    kind = _ADR_STATUS_ADMONITION.get(status_key, 'note')
    parts = []

    if status:
        parts.append(
            ADR_STATUS.get(
                status_key,
                str(status).replace('_', ' ').title(),
            )
        )
    if value_date:
        parts.append(f':material-calendar-clock: {human_date(value_date)}')

    title = ' · '.join(parts)
    return f'!!! {kind} "{title}"\n'


def universe_metadata(adr_path: str, status: str) -> str:
    """Render the relationship and status of a supporting alternative page."""
    status_key = str(status).lower()
    unknown_status = ('note', str(status).title())
    kind, label = _UNIVERSE_STATUS.get(status_key, unknown_status)
    article = label.lower()
    return f'!!! {kind} "{label} alternative universe"\n\n    This is the {article} alternative universe from the [parent ADR]({adr_path}).\n'


def example_code(name: str, indent: int = 0) -> str:
    path = example_path(name)
    source = path.read_text().rstrip('\n')
    syntax = EXAMPLE_SYNTAXES.get(path.suffix.lower(), 'text')
    return indent_block(f'```{syntax}\n{source}\n```\n', indent)
