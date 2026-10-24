"""Markdown table for SELECT bindings."""

from __future__ import annotations

from pydantic import BaseModel, TypeAdapter, field_validator

_BINDING_ROWS = TypeAdapter(list[dict[str, str]])


class BindingTable(BaseModel):
    """SELECT bindings passed to `sparql_table`."""

    rows: list[dict[str, str]]

    @field_validator('rows', mode='plain')
    @classmethod
    def require_binding_rows(cls, rows: object) -> list[dict[str, str]]:
        """Require a list of string-keyed binding rows."""
        return _BINDING_ROWS.validate_python(rows)


def _table_columns(rows: list[dict[str, str]]) -> list[str]:
    columns = list(rows[0].keys())
    for row in rows[1:]:
        for key in row:
            if key not in columns:
                columns.append(key)
    return columns


def _table_lines(
    columns: list[str],
    rows: list[dict[str, str]],
) -> list[str]:
    lines = [
        '| {values} |'.format(values=' | '.join(columns)),
        '| {values} |'.format(values=' | '.join('---' for _ in columns)),
    ]
    for row in rows:
        cells = [str(row.get(name, '')) for name in columns]
        lines.append('| {values} |'.format(values=' | '.join(cells)))
    return lines
