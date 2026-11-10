"""Live example_sparql macro against isolated scenario projects."""

from collections.abc import Iterator
from pathlib import Path

import pytest

from docs_macros.examples import clear_example_projects, example_sparql
from mkdocs_macros_sparqld.server import configure, stop_server
from mkdocs_macros_sparqld.table import SelectBindings


@pytest.fixture
def configured_macros(sparqld_binary: Path) -> Iterator[None]:
    """Point the pluglet at the repository and the project sparqld binary."""
    root = Path(__file__).resolve().parents[1]
    configure(
        project_dir=root,
        directory=root / 'docs',
        binary=sparqld_binary,
    )
    yield
    clear_example_projects()
    stop_server()


def test_example_sparql_select(configured_macros: None) -> None:
    bindings = example_sparql('default-context', 'triples')
    assert isinstance(bindings, SelectBindings)
    assert bindings.columns == ['subject', 'predicate']
    assert bindings.rows == [
        {
            'subject': 'http://dbpedia.org/resource/Alpha_Centauri',
            'predicate': 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type',
        },
        {
            'subject': 'http://dbpedia.org/resource/Alpha_Centauri',
            'predicate': 'https://schema.org/name',
        },
    ]


def test_example_sparql_ask(configured_macros: None) -> None:
    assert example_sparql('recursive-context/initial', 'declaration') is True
    assert example_sparql('recursive-context/unregistered', 'alpha-centauri') is False
