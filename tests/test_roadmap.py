"""Roadmap graph rendering from SPARQL results."""

import json
import time
from collections.abc import Callable
from pathlib import Path
from unittest.mock import Mock

import pytest
from serving import ROOT, SERVER_LOG_NAME, WAIT_SECONDS, _query, _serve

from docs_macros.roadmap import (
    ROADMAP_QUERY,
    RoadmapMacro,
    render_roadmap,
    roadmap,
)
from docs_macros.roadmap.nodes import Bug, Goal, Position, PrerequisiteState, Task
from docs_macros.roadmap.render import _roadmap
from mkdocs_macros_sparqld.render import SelectResult
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
    assert original.count(' --> ') == 3
    assert 'The mission outcome' in original


def test_target_renders_only_transitive_blockers(bindings: SelectBindings) -> None:
    """A target graph contains its full blocker chain and excludes unrelated outcomes."""
    rendered = render_roadmap(bindings, 'c')
    assert all(label in rendered for label in ('A', 'B', 'C'))
    assert 'The mission outcome' not in rendered
    assert rendered.count(' --> ') == 2


def test_target_includes_options_for_a_blocking_decision() -> None:
    """A selected decision retains its position nodes as its options."""
    rows = [
        {NODE: 'decision', TITLE: 'Choose', KIND: 'decision'},
        {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'group': 'decision', 'status': 'undecided'},
        {NODE: 'datalog', TITLE: 'Datalog', KIND: 'position', 'group': 'decision', 'status': 'undecided'},
        {NODE: 'goal', TITLE: 'Inference', KIND: GOAL, BLOCKER: 'decision'},
        {NODE: 'unrelated', TITLE: 'Unrelated', KIND: GOAL},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'goal')
    assert 'Choose' in rendered and 'OWL' in rendered and 'Datalog' in rendered
    assert 'Unrelated' not in rendered


def test_decision_options_retain_transitive_prerequisites() -> None:
    """Options of included decisions retain their full blocker chains."""
    rows = [
        {NODE: 'decision', TITLE: 'Choose', KIND: 'decision'},
        {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'group': 'decision',
         'status': 'undecided', BLOCKER: 'setup'},
        {NODE: 'datalog', TITLE: 'Datalog', KIND: 'position', 'group': 'decision',
         'status': 'undecided', BLOCKER: 'review'},
        {NODE: 'goal', TITLE: 'Inference', KIND: GOAL, BLOCKER: 'decision'},
        {NODE: 'setup', TITLE: 'Initial setup', KIND: GOAL, BLOCKER: 'base'},
        {NODE: 'base', TITLE: 'Base work', KIND: GOAL},
        {NODE: 'review', TITLE: 'Review', KIND: 'task'},
        {NODE: 'unrelated', TITLE: 'Unrelated', KIND: GOAL},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'goal')
    assert all(label in rendered for label in (
        'Choose', 'OWL', 'Datalog', 'Initial setup', 'Base work', 'Review',
    ))
    assert 'Unrelated' not in rendered


def test_position_prerequisite_includes_its_decision_and_options() -> None:
    """A blocking decision position keeps its group and sibling options in scope."""
    rows = [
        {NODE: 'decision', TITLE: 'Choose', KIND: 'decision'},
        {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'group': 'decision', 'status': 'undecided',
         'contribution': 'outside'},
        {NODE: 'datalog', TITLE: 'Datalog', KIND: 'position', 'group': 'decision', 'status': 'undecided'},
        {NODE: 'goal', TITLE: 'Inference', KIND: GOAL, BLOCKER: 'owl', 'blocker_kind': 'position'},
        {NODE: 'outside', TITLE: 'Outside scope', KIND: GOAL},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'goal')
    assert 'Choose' in rendered and 'OWL' in rendered and 'Datalog' in rendered
    assert 'Outside scope' not in rendered
    assert rendered.count(' --> ') == 1


def test_unknown_target_fails(bindings: SelectBindings) -> None:
    """A misspelled target does not silently render an empty graph."""
    with pytest.raises(ValueError, match='Unknown roadmap target'):
        render_roadmap(bindings, 'missing')


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


@pytest.mark.parametrize('kind', ['goal', 'bug', 'task'])
def test_work_items_share_a_shape(kind: str) -> None:
    """Goals, bugs, and GitHub issues use the same shape."""
    rendered = render_roadmap(
        SelectBindings(
            columns=[],
            rows=[{NODE: FIRST, TITLE: 'Work item', KIND: kind}],
        )
    )
    assert '("Work item")' in rendered


@pytest.mark.parametrize('node_type', [Goal, Task])
def test_work_items_default_to_open(node_type: type) -> None:
    """A missing status has the same color as an explicitly open task."""
    assert node_type(iri='item', title='Item').status_class() == 'status_open'
    assert node_type(iri='item', title='Item', status='closed').status_class() == 'status_closed'



@pytest.mark.parametrize('kind', ['goal', 'task', 'bug'])
@pytest.mark.parametrize('status', [None, '', 'open', 'closed', 'undecided', 'unknown'])
def test_query_result_status_reaches_roadmap(kind: str, status: str | None) -> None:
    """Unbound SELECT status values receive the work item's default style."""
    terms = {
        NODE: {'type': 'uri', 'value': 'urn:roadmap:item'},
        TITLE: {'type': 'literal', 'value': 'Work item'},
        KIND: {'type': 'literal', 'value': kind},
    }
    if status is not None:
        terms['status'] = {'type': 'literal', 'value': status}
    bindings = SelectResult.model_validate({
        'head': {'vars': [NODE, TITLE, KIND, 'status']},
        'results': {'bindings': [terms]},
    }).bindings()
    rendered = render_roadmap(bindings)

    if status == 'closed' and kind in ('task', 'bug'):
        assert 'Work item' not in rendered
        return
    expected = status or 'open'
    if expected == 'open' and kind == 'bug':
        expected = 'open_bug'
    elif expected == 'unknown':
        expected = 'unrecorded'
    assert 'Work item' in rendered
    assert f' status_{expected}\n' in rendered


def test_open_bugs_are_red_and_closed_bugs_are_hidden() -> None:
    assert Bug(iri='bug', title='Bug').status_class() == 'status_open_bug'
    assert Bug(iri='bug', title='Bug', status='open').status_class() == 'status_open_bug'
    rendered = render_roadmap(SelectBindings(columns=[], rows=[
        {NODE: 'mission', TITLE: 'Mission', KIND: 'goal', 'blocker': 'bug', 'blocker_kind': 'bug'},
        {NODE: 'bug', TITLE: 'Open bug', KIND: 'bug', 'status': 'open'},
        {NODE: 'done', TITLE: 'Closed bug', KIND: 'bug', 'status': 'closed', 'contribution': 'mission'},
        {NODE: 'task', TITLE: 'Open task', KIND: 'task', 'contribution': 'mission'},
    ]), 'mission')
    assert 'Open bug' in rendered and 'Open task' in rendered
    assert 'Closed bug' not in rendered
    assert 'classDef status_open_bug fill:#f4cccc' in rendered
    assert 'classDef status_open fill:#d9eaf7' in rendered


def test_default_task_status_does_not_change_other_entities() -> None:
    """States and decision options keep their own status semantics."""
    assert PrerequisiteState(iri='state', title='State').status_class() == 'status_unrecorded'
    assert PrerequisiteState(iri='state', title='State', status='').status_class() == 'status_unrecorded'
    assert Position(iri='option', title='Option', status='').status_class() == 'status_unrecorded'
    assert Position(iri='option', title='Option', status='undecided').status_class() == 'status_undecided'


def test_task_can_block_a_goal() -> None:
    """A roadmap task can appear as a prerequisite for an outcome."""
    rendered = render_roadmap(SelectBindings(columns=[], rows=[
        {NODE: 'task', TITLE: 'GitHub issue', KIND: 'task'},
        {NODE: 'goal', TITLE: 'Outcome', KIND: GOAL, BLOCKER: 'task', 'blocker_kind': 'task'},
    ]))
    assert 'GitHub issue' in rendered and 'Outcome' in rendered
    assert rendered.count(' --> ') == 1


def test_external_issue_can_block_a_mission_prerequisite() -> None:
    """An external blocker remains visible on the path to the mission."""
    rows = [
        {NODE: 'mission', TITLE: 'Mission', KIND: GOAL, BLOCKER: 'local', 'blocker_kind': 'task'},
        {NODE: 'local', TITLE: 'Local issue', KIND: 'task', BLOCKER: 'external', 'blocker_kind': 'task'},
        {NODE: 'external', TITLE: 'External issue', KIND: 'task', 'status': 'open'},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'mission')
    assert 'External issue' in rendered
    assert rendered.count(' --> ') == 2


def test_target_scope_omits_completed_task_blockers_and_their_edges() -> None:
    """Closed tasks and their blocker chains stay out of a scoped diagram."""
    rows = [
        {NODE: 'mission', TITLE: 'Mission', KIND: GOAL, BLOCKER: 'done',
         'blocker_kind': 'task'},
        {NODE: 'done', TITLE: 'Completed task', KIND: 'task', 'status': 'closed',
         BLOCKER: 'done-prerequisite', 'blocker_kind': 'task'},
        {NODE: 'done-prerequisite', TITLE: 'Old prerequisite', KIND: 'task', 'status': 'open'},
        {NODE: 'active', TITLE: 'Active task', KIND: 'task', 'status': 'open',
         BLOCKER: 'cancelled', 'blocker_kind': 'task', 'contribution': 'mission'},
        {NODE: 'cancelled', TITLE: 'Cancelled task', KIND: 'task', 'status': 'cancelled'},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'mission')

    assert 'Mission' in rendered and 'Active task' in rendered
    assert 'Completed task' not in rendered
    assert 'Old prerequisite' not in rendered
    assert 'Cancelled task' in rendered
    assert rendered.count(' --> ') == 2


def test_unscoped_roadmap_omits_completed_tasks_and_incident_edges() -> None:
    """Unscoped diagrams also omit completed tasks and their relationships."""
    rendered = render_roadmap(SelectBindings(columns=[], rows=[
        {NODE: 'done', TITLE: 'Completed task', KIND: 'task', 'status': 'closed'},
        {NODE: 'goal', TITLE: 'Outcome', KIND: GOAL, BLOCKER: 'done',
         'blocker_kind': 'task'},
    ]))
    assert 'Completed task' not in rendered
    assert rendered.count(' --> ') == 0


def test_mission_target_retains_contributors() -> None:
    """A mission target includes contributors and their blocker prerequisites."""
    rows = [
        {NODE: 'mission', TITLE: 'Mission', KIND: GOAL},
        {NODE: 'feature', TITLE: 'Feature', KIND: GOAL, 'contribution': 'mission'},
        {NODE: 'prerequisite', TITLE: 'Prerequisite', KIND: GOAL},
        {NODE: 'feature', TITLE: 'Feature', KIND: GOAL, BLOCKER: 'prerequisite',
         'contribution': 'mission'},
        {NODE: 'other', TITLE: 'Other', KIND: GOAL},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows), 'mission')
    assert all(label in rendered for label in ('Mission', 'Feature', 'Prerequisite'))
    assert 'Other' not in rendered


def test_decision_renders_positions_and_dependency() -> None:
    """A decision owns its positions and can block a goal."""
    rows = [
        {NODE: 'decision', TITLE: 'Choose', KIND: 'decision'},
        {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'group': 'decision', 'status': 'undecided'},
        {NODE: 'datalog', TITLE: 'Datalog', KIND: 'position', 'group': 'decision', 'status': 'undecided'},
        {NODE: 'goal', TITLE: 'Inference', KIND: GOAL, BLOCKER: 'decision'},
    ]
    rendered = render_roadmap(SelectBindings(columns=[], rows=rows))
    assert rendered.count('subgraph ') == 1
    assert 'Choose' in rendered and 'OWL' in rendered and 'Datalog' in rendered
    assert rendered.count(' --> ') == 1
    assert rendered == render_roadmap(SelectBindings(columns=[], rows=list(reversed(rows))))


@pytest.mark.parametrize(
    'rows',
    [
        [{NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'group': 'missing'}],
        [
            {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'status': 'undecided'},
            {NODE: 'owl', TITLE: 'OWL', KIND: 'position', 'status': 'accepted'},
        ],
        [{NODE: 'same', TITLE: 'A', KIND: GOAL}, {NODE: 'same', TITLE: 'A', KIND: 'bug'}],
    ],
)
def test_inconsistent_decisions_fail(rows: list[dict[str, str]]) -> None:
    """Missing decisions and conflicting metadata fail in either row order."""
    for ordered in (rows, list(reversed(rows))):
        with pytest.raises(ValueError):
            render_roadmap(SelectBindings(columns=[], rows=ordered))


def test_unknown_status_cannot_inject_mermaid() -> None:
    """Unrecognized status text uses the neutral style."""
    rendered = render_roadmap(SelectBindings(columns=[], rows=[
        {NODE: FIRST, TITLE: 'A', KIND: GOAL, 'status': 'x\nmalicious --> node'},
    ]))
    assert 'malicious' not in rendered
    assert 'status_unrecorded' in rendered


def test_prerequisite_states_use_stadium_shape() -> None:
    """Prerequisite states use a stadium to distinguish their entity type."""
    rendered = render_roadmap(
        SelectBindings(
            columns=[],
            rows=[
                {
                    NODE: 'urn:roadmap:bug',
                    TITLE: 'Bug',
                    KIND: 'bug',
                    BLOCKER: 'urn:roadmap:release',
                    'blocker_title': 'Release fixes the issue',
                    'blocker_kind': 'state',
                },
            ],
        )
    )
    assert '(["Release fixes the issue"])' in rendered


@pytest.mark.parametrize(
    'render',
    [
        lambda query: roadmap(query, 'urn:roadmap:mission'),
        lambda query: RoadmapMacro(Mock(macros={'stored_sparql': query})).render('urn:roadmap:mission'),
    ],
    ids=['helper', 'macro'],
)
def test_queries_before_rendering(
    bindings: SelectBindings,
    render: Callable[[Callable[[str], SelectBindings]], str],
) -> None:
    """Roadmap entry points execute their stored query internally."""

    query = Mock(return_value=bindings)
    assert render(query) == render_roadmap(bindings, 'urn:roadmap:mission')
    query.assert_called_once_with(ROADMAP_QUERY)


def test_helper_requires_select_bindings() -> None:
    """Non-SELECT query results produce a build error."""

    with pytest.raises(TypeError):
        roadmap(Mock(return_value='ASK bindings'))


def test_macro_accepts_target_iri(bindings: SelectBindings) -> None:
    """The MkDocs macro forwards its target IRI to the scoped renderer."""
    query = Mock(return_value=bindings)
    macro = RoadmapMacro(Mock(macros={'stored_sparql': query}))
    rendered = macro.render('c')
    assert 'The mission outcome' not in rendered
    query.assert_called_once_with(ROADMAP_QUERY)


@pytest.mark.parametrize(
    ('edges', 'expected'),
    [
        ({('a', 'b'), ('b', 'c'), ('a', 'c')}, {('a', 'b'), ('b', 'c')}),
        (
            {('a', 'b'), ('a', 'c'), ('b', 'd'), ('c', 'd'), ('a', 'd')},
            {('a', 'b'), ('a', 'c'), ('b', 'd'), ('c', 'd')},
        ),
    ],
    ids=['chain', 'diamond'],
)
def test_redundant_edges_preserve_reachability(
    edges: set[tuple[str, str]], expected: set[tuple[str, str]],
) -> None:
    """Reduction retains necessary paths and is independent of input order."""
    rows = [
        {NODE: node, TITLE: node.upper(), KIND: GOAL}
        for node in sorted({endpoint for edge in edges for endpoint in edge})
    ]
    rows.extend(
        {NODE: target, TITLE: target.upper(), KIND: GOAL, BLOCKER: source}
        for source, target in sorted(edges)
    )
    for ordered in (rows, list(reversed(rows)) + rows):
        diagram = _roadmap(SelectBindings(columns=[], rows=ordered))
        assert diagram.edges == expected


def test_closed_intermediate_keeps_direct_edge() -> None:
    """Completed intermediates cannot make a visible direct relationship redundant."""
    rows = [
        {NODE: 'a', TITLE: 'A', KIND: GOAL},
        {NODE: 'b', TITLE: 'B', KIND: 'task', 'status': 'closed', BLOCKER: 'a'},
        {NODE: 'c', TITLE: 'C', KIND: GOAL, BLOCKER: 'b'},
        {NODE: 'c', TITLE: 'C', KIND: GOAL, BLOCKER: 'a'},
    ]
    diagram = _roadmap(SelectBindings(columns=[], rows=rows), 'c')
    assert diagram.edges == {('a', 'c')}


@pytest.mark.parametrize(
    'edges',
    [
        [('a', 'b'), ('b', 'a')],
        [('a', 'b'), ('b', 'c'), ('c', 'a')],
        [('a', 'b'), ('b', 'a'), ('a', 'c'), ('b', 'c')],
        [('a', 'a')],
    ],
    ids=['two-node-cycle', 'long-cycle', 'cycle-with-shared-target', 'self-loop'],
)
@pytest.mark.parametrize('scoped', [False, True])
def test_roadmap_cycles_fail(edges: list[tuple[str, str]], scoped: bool) -> None:
    """Cycles fail with a stable path and an actionable relationship error."""
    rows = [
        {NODE: node, TITLE: node.upper(), KIND: GOAL}
        for node in sorted({endpoint for edge in edges for endpoint in edge})
    ]
    rows.extend(
        {NODE: target, TITLE: target.upper(), KIND: GOAL, BLOCKER: source}
        for source, target in edges
    )
    messages = []
    for ordered in (rows, list(reversed(rows)) + rows):
        with pytest.raises(ValueError, match='Roadmap relationships contain a cycle:') as error:
            render_roadmap(SelectBindings(columns=[], rows=ordered), 'a' if scoped else None)
        messages.append(str(error.value))
    assert messages[0] == messages[1]
    assert 'a -> ' in messages[0]
    assert 'Remove a blocker or contribution relationship' in messages[0]


def test_contribution_and_blocker_cycle_fails() -> None:
    """Contribution and blocker relationships participate in the same cycle check."""
    rows = [
        {NODE: 'a', TITLE: 'A', KIND: GOAL, 'contribution': 'b', BLOCKER: 'b'},
        {NODE: 'b', TITLE: 'B', KIND: GOAL},
    ]
    with pytest.raises(ValueError, match='a -> b -> a'):
        render_roadmap(SelectBindings(columns=[], rows=rows))


@pytest.mark.parametrize('completed_kind', [
    'https://vocab.methodandstructure.com/process-model#Task',
    'urn:sparqld:vocabulary:Bug',
])
@pytest.mark.parametrize(('status_property', 'closed_status'), [
    ('https://vocab.methodandstructure.com/process-model#status',
     {'@id': 'urn:roadmap:status:closed'}),
    ('https://schema.org/creativeWorkStatus', 'CLOSED'),
])
def test_query_omits_completed_contribution_targets(
    sparqld_binary: Path,
    tmp_path: Path,
    completed_kind: str,
    status_property: str,
    closed_status: dict[str, str] | str,
) -> None:
    """The stored query keeps live contributions and drops completed endpoints."""
    goal_type = 'https://vocab.methodandstructure.com/process-model#Goal'
    label_property = 'http://www.w3.org/2000/01/rdf-schema#label'
    project = tmp_path / 'project'
    project.mkdir()
    (project / 'roadmap.jsonld').write_text(json.dumps({'@graph': [
        {'@id': 'urn:live', '@type': goal_type, label_property: 'Live contributor',
         'urn:sparqld:vocabulary:contributesTo': [
             {'@id': 'urn:completed'}, {'@id': 'urn:open'},
         ]},
        {'@id': 'urn:completed', '@type': completed_kind,
         label_property: 'Completed target', status_property: closed_status},
        {'@id': 'urn:open', '@type': completed_kind, label_property: 'Open target'},
    ]}))
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        deadline = time.monotonic() + WAIT_SECONDS
        while True:
            try:
                result = _query(server.endpoint, ROOT / ROADMAP_QUERY)
                assert isinstance(result, SelectResult)
                rows = result.rows()
                assert {row[NODE] for row in rows} == {'urn:live', 'urn:open'}
            except (OSError, AssertionError):
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.1)
            else:
                break
        contributions = {row['contribution'] for row in rows if row['contribution']}
        assert contributions == {'urn:open'}
        diagram = render_roadmap(result.bindings())
        assert 'Live contributor' in diagram
        assert 'Open target' in diagram
        assert 'Completed target' not in diagram
