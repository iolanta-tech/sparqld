"""GitHub roadmap projection behavior."""

from pathlib import Path

import pytest
from ruamel.yaml import YAML

from skills.roadmap import sync_github


def issue(number: int, state: str = 'open') -> sync_github.Issue:
    return sync_github.Issue(
        number=number,
        html_url=f'https://github.com/iolanta-tech/sparqld/issues/{number}',
        title=f'Issue {number}',
        body='Details',
        state=state,
    )


def metadata(path: Path) -> dict:
    """Read the YAML-LD frontmatter independently of Markdown contents."""
    return YAML(typ='safe').load(path.read_text().split('---\n', 2)[1])


def test_sync_writes_issue_files_with_mission_blockers(monkeypatch, tmp_path: Path) -> None:
    """Dependencies replace redundant mission links and preserve issue metadata."""
    destination = tmp_path / 'github-issues'
    legacy = tmp_path / 'github-issues.yamlld'
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', legacy)
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    monkeypatch.setattr(
        sync_github, 'fetch',
        lambda endpoint: [issue(1), issue(2, 'closed')]
        if 'state=all' in endpoint else ([issue(2, 'closed')] if '/1/' in endpoint else []),
    )
    destination.mkdir()
    (destination / '999.yamlld').write_text(
        '$graph:\n- $id: https://github.com/iolanta-tech/sparqld/issues/999\n'
        '  $type: urn:sparqld:vocabulary:GitHubIssue\n'
    )
    (destination / 'notes.txt').write_text('keep this')
    legacy.write_text(
        '$graph:\n- $id: https://github.com/iolanta-tech/sparqld/issues/999\n'
        '  $type: urn:sparqld:vocabulary:GitHubIssue\n'
    )
    sync_github.main()
    first = destination / '1.md'
    second = destination / '2.md'
    original = first.read_text(), second.read_text()
    sync_github.main()
    assert (first.read_text(), second.read_text()) == original
    record = metadata(first)
    assert '$graph' not in record
    assert record['blocks'] == sync_github.MISSION
    assert record['$id'] == issue(1).html_url
    assert record['$type'] == 'pm:Task'
    assert record['title'] == issue(1).title
    assert 'rdfs:label' not in record
    assert record['blockedBy'] == issue(2).html_url
    closed_record = metadata(second)
    assert 'blocks' not in closed_record
    assert closed_record['status'] == 'closed'
    assert not (destination / '999.yamlld').exists()
    assert (destination / 'notes.txt').read_text() == 'keep this'
    assert not legacy.exists()


def test_external_blocker_is_described_without_blocking_mission(monkeypatch, tmp_path: Path) -> None:
    """External dependencies retain their details without becoming mission links."""
    destination = tmp_path / 'github-issues'
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', tmp_path / 'legacy.yamlld')
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    external = sync_github.Issue(
        number=7,
        html_url='https://github.com/example/project/issues/7',
        title='External prerequisite',
        state='closed',
    )
    monkeypatch.setattr(
        sync_github, 'fetch',
        lambda endpoint: [issue(1)] if 'state=all' in endpoint else [external],
    )

    sync_github.main()

    record = metadata(destination / '1.md')
    assert '$graph' not in record
    assert record['blocks'] == sync_github.MISSION
    assert record['blockedBy'] == {
        '$id': 'https://github.com/example/project/issues/7',
        '$type': 'pm:Task',
        'title': 'External prerequisite',
        'status': 'closed',
    }


def test_sync_failure_preserves_projection(monkeypatch, tmp_path: Path) -> None:
    """A failed dependency fetch leaves all prior projection files unchanged."""
    destination = tmp_path / 'github-issues'
    destination.mkdir()
    issue_file = destination / '1.yamlld'
    issue_file.write_text('previous issue')
    legacy = tmp_path / 'github-issues.yamlld'
    legacy.write_text('previous aggregate')
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', legacy)
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)

    def fetch(endpoint: str) -> list[sync_github.Issue]:
        if 'state=all' in endpoint:
            return [issue(1)]
        raise RuntimeError('GitHub unavailable')

    monkeypatch.setattr(sync_github, 'fetch', fetch)
    with pytest.raises(RuntimeError):
        sync_github.main()
    assert issue_file.read_text() == 'previous issue'
    assert legacy.read_text() == 'previous aggregate'


def test_sync_refuses_to_overwrite_unrelated_file(monkeypatch, tmp_path: Path) -> None:
    """A numeric filename alone does not authorize replacing a file."""
    destination = tmp_path / 'github-issues'
    destination.mkdir()
    issue_file = destination / '1.yamlld'
    issue_file.write_text('hand-authored content')
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    monkeypatch.setattr(
        sync_github, 'fetch', lambda endpoint: [issue(1)] if 'state=all' in endpoint else [],
    )
    with pytest.raises(ValueError, match='non-generated'):
        sync_github.main()
    assert issue_file.read_text() == 'hand-authored content'


def test_bug_label_survives_repeated_sync_and_stale_cleanup(monkeypatch, tmp_path: Path) -> None:
    """Bug classification retains identity for refresh and stale-file cleanup."""
    destination = tmp_path / 'github-issues'
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', tmp_path / 'legacy.yamlld')
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    bug = issue(26).model_copy(update={'labels': [sync_github.Label(name='bug')]})
    issues = [bug]
    monkeypatch.setattr(
        sync_github, 'fetch', lambda endpoint: issues if 'state=all' in endpoint else [],
    )

    sync_github.main()
    sync_github.main()

    issue_file = destination / '26.md'
    record = metadata(issue_file)
    assert record['$type'] == [
        'pm:Task', 'urn:sparqld:vocabulary:Bug',
    ]
    assert record['status'] == 'open'
    issues.clear()
    sync_github.main()
    assert not issue_file.exists()


def test_external_bug_blocker_retains_classification() -> None:
    """External blockers use the same label classification as local issues."""
    external = issue(7).model_copy(update={
        'html_url': 'https://github.com/example/project/issues/7',
        'labels': [sync_github.Label(name='bug')],
    })
    record = sync_github.issue_document(issue(1), [external])['blockedBy']
    assert record['$type'] == [
        'pm:Task', 'urn:sparqld:vocabulary:Bug',
    ]


@pytest.mark.parametrize(('state', 'reason', 'status'), [
    ('open', None, 'open'),
    ('closed', 'completed', 'closed'),
    ('closed', 'not_planned', 'cancelled'),
])
@pytest.mark.parametrize('external', [False, True])
def test_issue_status_resources(state: str, reason: str | None, status: str, external: bool) -> None:
    """Local tasks and external prerequisites share GitHub status mapping."""
    task = issue(1, state).model_copy(update={'state_reason': reason})
    if external:
        task = task.model_copy(update={
            'html_url': 'https://github.com/example/project/issues/1',
        })
        record = sync_github.issue_document(issue(2), [task])['blockedBy']
    else:
        record = sync_github.issue_document(task, [])
    assert record['status'] == status
    assert 'schema:creativeWorkStatus' not in record


@pytest.mark.parametrize('record', [
    {'$id': 'https://github.com/iolanta-tech/sparqld/issues/2',
     '$type': 'urn:sparqld:vocabulary:GitHubIssue'},
    {'$id': 'https://github.com/iolanta-tech/sparqld/issues/2', '$type': 'pm:Task'},
    {'$id': 'https://github.com/example/project/issues/1', '$type': 'pm:Task'},
    {'$id': 'https://github.com/iolanta-tech/sparqld/issues/1',
     '$type': 'urn:sparqld:vocabulary:Bug'},
])
def test_generated_recognition_requires_issue_identity_and_type(tmp_path: Path, record: dict) -> None:
    """Preserve unrelated records even when their filename matches an issue."""
    path = tmp_path / '1.yamlld'
    with path.open('w') as stream:
        YAML().dump(record, stream)
    assert not sync_github.is_generated_issue_file(path)


def test_sync_migrates_previous_issue_graph(monkeypatch, tmp_path: Path) -> None:
    """The previous generated shape migrates and then remains byte stable."""
    destination = tmp_path / 'github-issues'
    destination.mkdir()
    path = destination / '1.yamlld'
    path.write_text(
        '$graph:\n- $id: https://github.com/iolanta-tech/sparqld/issues/1\n'
        '  $type: urn:sparqld:vocabulary:GitHubIssue\n'
        '  blocks:\n  - urn:sparqld:roadmap:mission\n'
    )
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', tmp_path / 'legacy.yamlld')
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    monkeypatch.setattr(
        sync_github, 'fetch', lambda endpoint: [issue(1)] if 'state=all' in endpoint else [],
    )
    sync_github.main()
    assert not path.exists()
    path = destination / '1.md'
    original = path.read_text()
    record = metadata(path)
    assert '$graph' not in record
    assert record['$type'] == 'pm:Task'
    assert record['blocks'] == sync_github.MISSION
    sync_github.main()
    assert path.read_text() == original


@pytest.mark.parametrize(('edges', 'expected'), [
    ({1: [2], 2: [3]}, {1}),
    ({1: [2, 3], 2: [4], 3: [4]}, {1}),
    ({1: [2], 2: [1]}, {1}),
    ({1: [2], 2: [1], 3: [2]}, {3}),
    ({1: [1], 2: []}, {1, 2}),
])
def test_mission_anchors_preserve_dependency_paths(edges, expected) -> None:
    """Chains, branches and cycles retain stable terminal mission anchors."""
    numbers = set(edges) | {number for blockers in edges.values() for number in blockers}
    issues = [issue(number) for number in sorted(numbers)]
    dependencies = {
        item.html_url: [issue(number) for number in edges.get(item.number, [])]
        for item in issues
    }
    expected_urls = {issue(number).html_url for number in expected}
    assert sync_github.mission_blockers(issues, dependencies) == expected_urls
    assert sync_github.mission_blockers(list(reversed(issues)), dependencies) == expected_urls


@pytest.mark.parametrize(('state', 'reason', 'expected'), [
    ('closed', None, {1, 2}),
    ('closed', 'not_planned', {1}),
    ('open', None, {1}),
])
def test_downstream_visibility_controls_mission_shortcuts(state, reason, expected) -> None:
    """A hidden completed dependent cannot carry its prerequisite's mission path."""
    dependent = issue(1, state).model_copy(update={'state_reason': reason})
    prerequisite = issue(2)
    dependencies = {dependent.html_url: [prerequisite], prerequisite.html_url: []}
    assert sync_github.mission_blockers([dependent, prerequisite], dependencies) == {
        issue(number).html_url for number in expected
    }


def test_issue_document_omits_only_mission_shortcut() -> None:
    """Removing a shortcut preserves dependencies, body, identity and status."""
    original = sync_github.issue_document(issue(1), [issue(2)])
    reduced = sync_github.issue_document(issue(1), [issue(2)], blocks_mission=False)
    assert reduced == {key: value for key, value in original.items() if key != 'blocks'}


@pytest.mark.parametrize('types', [
    'pm:Task',
    ['pm:Task', 'urn:sparqld:vocabulary:Bug'],
    'urn:sparqld:vocabulary:GitHubIssue',
    ['urn:sparqld:vocabulary:GitHubIssue', 'urn:sparqld:vocabulary:Bug'],
])
def test_generated_recognition_accepts_current_and_legacy_types(tmp_path: Path, types) -> None:
    """Current tasks and legacy issues require an exact local issue URL."""
    path = tmp_path / '1.yamlld'
    with path.open('w') as stream:
        YAML().dump({'$id': issue(1).html_url, '$type': types}, stream)
    assert sync_github.is_generated_issue_file(path)


def test_multiple_blockers_use_sorted_list() -> None:
    """Several blockers retain local references and embedded external records."""
    external = issue(7).model_copy(update={
        'html_url': 'https://github.com/example/project/issues/7',
    })
    record = sync_github.issue_document(issue(1), [issue(3), external, issue(2)])
    assert record['blockedBy'] == [
        {
            '$id': external.html_url,
            '$type': 'pm:Task',
            'title': external.title,
            'status': 'open',
        },
        issue(2).html_url,
        issue(3).html_url,
    ]
    assert 'dependsOn' not in record


def test_unblocked_issue_omits_blocker_property() -> None:
    """An issue without blockers retains its mission anchor."""
    record = sync_github.issue_document(issue(1), [])
    assert 'blockedBy' not in record
    assert 'dependsOn' not in record
    assert record['blocks'] == sync_github.MISSION


@pytest.mark.parametrize('body', [None, '', '# Details\n\n---\n\n{{ unsafe() }}\n', 'Line\nlast'])
def test_markdown_body_is_separate_from_metadata(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, body: str | None,
) -> None:
    """Issue Markdown remains exact text after safely serialized frontmatter."""
    task = issue(1).model_copy(update={'body': body, 'title': 'Title: "quoted" <script>alert(1)</script>\nnext'})
    destination = tmp_path / 'github-issues'
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', tmp_path / 'legacy.yamlld')
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    monkeypatch.setattr(
        sync_github, 'fetch', lambda endpoint: [task] if 'state=all' in endpoint else [],
    )
    sync_github.main()
    path = destination / '1.md'
    original = path.read_text()
    record = metadata(path)
    assert record['title'] == task.title
    assert record['render_macros'] is False
    assert 'schema:description' not in record
    assert original.split('---\n', 2)[2] == '\n' + (body or '')
    sync_github.main()
    assert path.read_text() == original


@pytest.mark.parametrize('collision_suffix', ['.md', '.yamlld'])
def test_migration_collision_preserves_all_files(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, collision_suffix: str,
) -> None:
    """Preflight all destinations before replacing or deleting generated records."""
    destination = tmp_path / 'github-issues'
    destination.mkdir()
    previous = destination / '1.yamlld'
    previous.write_text(
        '$id: https://github.com/iolanta-tech/sparqld/issues/1\n$type: pm:Task\n'
    )
    collision = destination / f'2{collision_suffix}'
    collision.write_text('User notes')
    unrelated = destination / '3.md'
    unrelated.write_text('Keep stale user notes')
    monkeypatch.setattr(sync_github, 'DESTINATION_DIR', destination)
    monkeypatch.setattr(sync_github, 'LEGACY_DESTINATION', tmp_path / 'legacy.yamlld')
    monkeypatch.setattr(sync_github, 'ROOT', tmp_path)
    monkeypatch.setattr(
        sync_github, 'fetch', lambda endpoint: [issue(1), issue(2)] if 'state=all' in endpoint else [],
    )
    before = {path.name: path.read_bytes() for path in destination.iterdir()}
    with pytest.raises(ValueError, match='non-generated'):
        sync_github.main()
    assert {path.name: path.read_bytes() for path in destination.iterdir()} == before


def test_generated_recognition_preserves_mixed_graph(tmp_path: Path) -> None:
    """A matching issue does not authorize deleting additional authored records."""
    path = tmp_path / '1.yamlld'
    with path.open('w') as stream:
        YAML().dump({'$graph': [
            {'$id': issue(1).html_url, '$type': 'pm:Task'},
            {'$id': 'urn:authored:goal', '$type': 'pm:Goal'},
        ]}, stream)
    assert not sync_github.is_generated_issue_file(path)
