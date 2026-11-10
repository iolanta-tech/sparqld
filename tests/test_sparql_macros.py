"""Row limits for SPARQL query examples rendered by MkDocs."""

import json
import re
from pathlib import Path
from types import SimpleNamespace

import pytest

from mkdocs_macros_sparqld import macros

DOCS = Path(__file__).resolve().parents[1] / 'docs'


def _select_result(row_count: int) -> str:
    return json.dumps(
        {
            'head': {'vars': ['name']},
            'results': {
                'bindings': [
                    {'name': {'type': 'literal', 'value': f'name-{index}'}}
                    for index in range(row_count)
                ]
            },
        }
    )


@pytest.fixture
def configured_macros(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> macros.SparqldMacros:
    monkeypatch.setattr(macros, 'configure', lambda _env: tmp_path)
    monkeypatch.setattr(macros, 'ensure_endpoint', lambda: 'http://localhost:1234/')
    env = SimpleNamespace(variables={})
    return macros.SparqldMacros(env)


@pytest.mark.parametrize('row_count', [10, 11])
@pytest.mark.parametrize('query_method', ['sparql', 'stored_sparql'])
def test_query_macros_enforce_select_row_limit(
    configured_macros: macros.SparqldMacros,
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    row_count: int,
    query_method: str,
) -> None:
    query = 'SELECT ?name WHERE { ?subject <urn:name> ?name }'
    query_file = tmp_path / 'query.rq'
    query_file.write_text(query)
    monkeypatch.setattr(
        macros,
        'run_query',
        lambda _query: (
            'application/sparql-results+json',
            _select_result(row_count),
        ),
    )

    query_operation = getattr(configured_macros, query_method)
    query_argument = query if query_method == 'sparql' else 'query.rq'

    if row_count == macros.MAX_SELECT_ROWS:
        assert len(query_operation(query_argument).rows) == macros.MAX_SELECT_ROWS
    else:
        with pytest.raises(ValueError, match='the maximum is 10'):
            query_operation(query_argument)


def test_documented_limit_only_for_pagination() -> None:
    limit_queries = {
        path.relative_to(DOCS).as_posix()
        for path in DOCS.rglob('*.rq')
        if re.search(r'\bLIMIT\b', path.read_text(), re.IGNORECASE)
    }
    assert limit_queries == {'queries/paginated-names.rq'}

    inline_query_pattern = re.compile(
        r"\{\{\s*sparql\(\s*(['\"])(.*?)\1\s*\)(?:\s*\|\s*[\w]+)*\s*\}\}",
        re.DOTALL,
    )
    inline_limit_queries = [
        path.relative_to(DOCS).as_posix()
        for path in DOCS.rglob('*.md')
        if any(
            re.search(r'\bLIMIT\b', match.group(2), re.IGNORECASE)
            for match in inline_query_pattern.finditer(path.read_text())
        )
    ]
    assert inline_limit_queries == []
