"""Markdown rendering for SELECT bindings."""

from mkdocs_macros_sparqld.render import bindings_table, parse_result
from mkdocs_macros_sparqld.table import SelectBindings


def test_bindings_table_renders_rows() -> None:
    table = bindings_table(
        SelectBindings(
            columns=['name'],
            rows=[{'name': 'Alpha Centauri'}],
        )
    )
    assert table == ('| name |\n| --- |\n| Alpha Centauri |')


def test_bindings_table_keeps_headers_when_empty() -> None:
    table = bindings_table(SelectBindings(columns=['file'], rows=[]))
    assert table == '| file |\n| --- |'


def test_bindings_table_accepts_plain_rows() -> None:
    table = bindings_table([{'subject': 'a', 'predicate': 'b'}])
    assert '| subject | predicate |' in table
    assert '| a | b |' in table


def test_parse_keeps_empty_select_vars() -> None:
    body = '{"head":{"vars":["file"]},"results":{"bindings":[]}}'
    parsed = parse_result('application/sparql-results+json', body)
    assert isinstance(parsed, SelectBindings)
    assert parsed.columns == ['file']
    assert parsed.rows == []
    assert bindings_table(parsed) == '| file |\n| --- |'


def test_parse_result_ask_boolean() -> None:
    body = '{"head":{},"boolean":false}'
    assert parse_result('application/sparql-results+json', body) is False
