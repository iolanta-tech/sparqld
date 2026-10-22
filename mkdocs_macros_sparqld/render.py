"""Render SELECT bindings for MkDocs Markdown."""

from __future__ import annotations

import json
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, StrictBool, TypeAdapter

_JSON_CONFIG = ConfigDict(extra='ignore')


class Term(BaseModel):
    """One RDF term in a SPARQL JSON binding."""

    model_config = _JSON_CONFIG

    term_type: Literal['uri', 'literal', 'bnode'] = Field(alias='type')
    lexical_form: str = Field(alias='value')


class AskResult(BaseModel):
    """SPARQL Query Results JSON for an ASK query."""

    model_config = _JSON_CONFIG

    boolean: StrictBool


class QueryVariables(BaseModel):
    """Variable names projected by a SELECT query."""

    model_config = _JSON_CONFIG

    variables: list[str] = Field(alias='vars')


class QuerySolutions(BaseModel):
    """Solution bindings for a SELECT query."""

    model_config = _JSON_CONFIG

    bindings: list[dict[str, Term]]


class SelectResult(BaseModel):
    """SPARQL Query Results JSON for a SELECT query."""

    model_config = _JSON_CONFIG

    head: QueryVariables
    solutions: QuerySolutions = Field(alias='results')

    def rows(self) -> list[dict[str, str]]:
        """Return each binding as variable names mapped to term values."""
        table: list[dict[str, str]] = []
        for binding in self.solutions.bindings:
            row: dict[str, str] = {}
            for name in self.head.variables:
                term = binding.get(name)
                if term is None:
                    row[name] = ''
                else:
                    row[name] = term.lexical_form
            table.append(row)
        return table


_JSON_RESULT: TypeAdapter[AskResult | SelectResult] = TypeAdapter(
    AskResult | SelectResult,
)
QueryOutput = list[dict[str, str]] | bool | str


def parse_result(content_type: str, body: str) -> QueryOutput:
    """Parse a SPARQL response into bindings, a boolean, or Turtle text."""
    if content_type == 'text/turtle':
        return body.strip()

    parsed = _JSON_RESULT.validate_python(json.loads(body))
    if isinstance(parsed, AskResult):
        return parsed.boolean
    return parsed.rows()


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
        '| {values} |'.format(values=' | '.join(columns)),
        '| {values} |'.format(values=' | '.join('---' for _ in columns)),
    ]
    for row in rows:
        cells = [str(row.get(name, '')) for name in columns]
        lines.append('| {values} |'.format(values=' | '.join(cells)))
    return '\n'.join(lines)
