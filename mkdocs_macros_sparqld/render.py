"""Render SELECT bindings for MkDocs Markdown."""

from __future__ import annotations

import json


def parse_result(content_type: str, body: str) -> list[dict[str, str]] | bool | str:
    """Parse a SPARQL response into bindings, a boolean, or Turtle text."""
    if content_type == 'text/turtle':
        return body.strip()

    result = json.loads(body)
    if 'boolean' in result:
        return bool(result['boolean'])

    variables = result['head']['vars']
    rows: list[dict[str, str]] = []
    for binding in result['results']['bindings']:
        rows.append(
            {name: binding.get(name, {}).get('value', '') for name in variables}
        )
    return rows


def bindings_table(rows: list[dict[str, str]]) -> str:
    """Render SELECT bindings as a Markdown table."""
    if not isinstance(rows, list):
        raise TypeError(
            'sparql_table expects a list of bindings from sparql() or stored_sparql()'
        )
    if not rows:
        return ''

    columns: list[str] = list(rows[0].keys())
    for row in rows[1:]:
        if not isinstance(row, dict):
            raise TypeError(
                'sparql_table expects a list of binding dicts from sparql() '
                'or stored_sparql()'
            )
        for key in row:
            if key not in columns:
                columns.append(key)

    lines = [
        '| ' + ' | '.join(columns) + ' |',
        '| ' + ' | '.join('---' for _ in columns) + ' |',
    ]
    for row in rows:
        cells = [str(row.get(name, '')) for name in columns]
        lines.append('| ' + ' | '.join(cells) + ' |')
    return '\n'.join(lines)
