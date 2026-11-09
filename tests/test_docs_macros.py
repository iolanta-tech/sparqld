"""Decision-log models loaded through the MkDocs macro loader."""

import sys
from pathlib import Path

import pytest
from mkdocs_macros.util import import_local_module

from docs_macros.sources import code_fence


def _literal(lexical_form: str) -> dict[str, str]:
    return {'type': 'literal', 'value': lexical_form}


def test_code_fence_outlasts_nested_markdown_fences() -> None:
    source = '# Title\n\n```sh\necho hi\n```\n'
    rendered = code_fence(source, 'markdown')
    assert rendered.startswith('````markdown\n')
    assert rendered.endswith('\n````')
    assert '```sh\necho hi\n```' in rendered


def test_decision_model_from_local_module(monkeypatch: pytest.MonkeyPatch) -> None:
    """Resolve RDF term annotations when MkDocs loads the local module."""
    monkeypatch.delitem(sys.modules, 'docs_macros', raising=False)
    module = import_local_module(
        str(Path(__file__).resolve().parents[1]), 'docs_macros'
    )
    binding = {
        'graph': {'type': 'uri', 'value': 'https://example.org/decision'},
        'title': _literal('Load decision models'),
        'status': _literal('decided'),
        'date': _literal('2026-10-24'),
        'author': _literal('Example author'),
    }

    decision = module.Decision.model_validate(binding)

    assert decision.title.lexical_form == 'Load decision models'
    assert decision.graph.term_type == 'uri'


def test_decision_page_keeps_a_directory_index() -> None:
    """Link a nested decision to its directory, and a flat decision to its file."""
    from docs_macros.decisions import decision_page

    nested = (
        'sparqld:project/decisions/'
        'implement-yaml-ld-and-markdown-ld-lsp-support-with/index.md'
    )
    flat = 'sparqld:project/decisions/implement-sparqld-in.md'

    assert decision_page(nested) == (
        'implement-yaml-ld-and-markdown-ld-lsp-support-with/index.md'
    )
    assert decision_page(flat) == 'implement-sparqld-in.md'
    assert decision_page('https://example.org/decision') == 'decision'
