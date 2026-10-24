"""Decision-log models loaded through the MkDocs macro loader."""

import sys
from pathlib import Path

import pytest
from mkdocs_macros.util import import_local_module


def _literal(lexical_form: str) -> dict[str, str]:
    return {'type': 'literal', 'value': lexical_form}


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
