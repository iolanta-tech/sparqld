"""Roadmap graph rendering from SPARQL results."""

from collections.abc import Callable
from unittest.mock import Mock

import pytest

from docs_macros.roadmap import (
    ROADMAP_QUERY,
    RoadmapMacro,
    render_roadmap,
    roadmap,
)
from mkdocs_macros_sparqld.table import SelectBindings

NODE = 'roadmap_item'
TITLE = 'title'
BLOCKER = 'blocker'
FIRST = 'a'
KIND = 'kind'
GOAL = 'goal'


@pytest.fixture
def bindings() -> SelectBindings:
    """A graph with two blockers and a transitive dependency."""
    return SelectBindings(
        columns=[],
        rows=[
            {NODE: FIRST, TITLE: 'A', KIND: GOAL},
            {NODE: 'b', TITLE: 'B', BLOCKER: FIRST, KIND: GOAL},
            {NODE: 'c', TITLE: 'C', BLOCKER: 'b', KIND: GOAL},
            {NODE: 'c', TITLE: 'C', BLOCKER: FIRST, KIND: GOAL},
            {
                NODE: 'urn:roadmap:mission',
                TITLE: 'The mission outcome',
                KIND: GOAL,
            },
            {NODE: 'c', TITLE: 'C', 'contribution': 'urn:roadmap:mission', KIND: GOAL},
        ],
    )


def test_graph_is_deterministic(bindings: SelectBindings) -> None:
    """Ordering and duplicate query rows do not change the diagram."""
    original = render_roadmap(bindings)
    bindings.rows = list(reversed(bindings.rows)) + bindings.rows
    assert render_roadmap(bindings) == original
    assert original.count(' --> ') == 4
    assert 'The mission outcome' in original


@pytest.mark.parametrize(
    'rows',
    [
        [{NODE: FIRST}],
        [{NODE: FIRST, TITLE: 'A'}, {NODE: FIRST, TITLE: 'B'}],
        [{NODE: FIRST, TITLE: 'A', BLOCKER: 'missing'}],
    ],
)
def test_invalid_graph_fails(rows: list[dict[str, str]]) -> None:
    """Incomplete graphs produce a build error."""
    with pytest.raises(ValueError):
        render_roadmap(SelectBindings(columns=[], rows=rows))


def test_labels_cannot_inject_mermaid() -> None:
    """Quotes, brackets, HTML, and fences remain label text."""
    rendered = render_roadmap(
        SelectBindings(
            columns=[],
            rows=[
                {NODE: FIRST, TITLE: '"[x]<script>```\n next', KIND: GOAL},
            ],
        )
    )
    assert '#34;#91;x#93;#60;script#62;#96;#96;#96; next' in rendered
    assert rendered.count('```') == 2


def test_bugs_get_red_mermaid_class() -> None:
    """Bug nodes use the diagram's red bug style."""
    rendered = render_roadmap(
        SelectBindings(
            columns=[],
            rows=[{NODE: FIRST, TITLE: 'Bug', KIND: 'bug'}],
        )
    )
    assert ':::bug' in rendered
    assert 'classDef bug fill:#fee2e2,stroke:#dc2626,color:#7f1d1d' in rendered


@pytest.mark.parametrize(
    'render',
    [
        roadmap,
        lambda query: RoadmapMacro(Mock(macros={'stored_sparql': query})).render(),
    ],
    ids=['helper', 'macro'],
)
def test_queries_before_rendering(
    bindings: SelectBindings,
    render: Callable[[Callable[[str], SelectBindings]], str],
) -> None:
    """Roadmap entry points execute their stored query internally."""

    query = Mock(return_value=bindings)
    assert render(query) == render_roadmap(bindings)
    query.assert_called_once_with(ROADMAP_QUERY)


def test_helper_requires_select_bindings() -> None:
    """Non-SELECT query results produce a build error."""

    with pytest.raises(TypeError):
        roadmap(Mock(return_value='ASK bindings'))
