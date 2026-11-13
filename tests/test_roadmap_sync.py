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


def test_sync_writes_issue_files_with_mission_blockers(monkeypatch, tmp_path: Path) -> None:
    """Each issue has a stable IRI and explicitly blocks the mission."""
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
    first = destination / '1.yamlld'
    second = destination / '2.yamlld'
    original = first.read_text(), second.read_text()
    sync_github.main()
    assert (first.read_text(), second.read_text()) == original
    graph = YAML(typ='safe').load(first.read_text())['$graph']
    assert graph[0]['$id'] == 'urn:sparqld:roadmap:mission'
    assert graph[0]['blockedBy'] == ['https://github.com/iolanta-tech/sparqld/issues/1']
    assert graph[1]['$id'] == 'https://github.com/iolanta-tech/sparqld/issues/1'
    assert graph[1]['$type'] == 'urn:sparqld:vocabulary:GitHubIssue'
    assert graph[1]['dependsOn'] == ['https://github.com/iolanta-tech/sparqld/issues/2']
    closed_graph = YAML(typ='safe').load(second.read_text())['$graph']
    assert closed_graph[0]['blockedBy'] == ['https://github.com/iolanta-tech/sparqld/issues/2']
    assert closed_graph[1]['schema:creativeWorkStatus'] == 'closed'
    assert not (destination / '999.yamlld').exists()
    assert (destination / 'notes.txt').read_text() == 'keep this'
    assert not legacy.exists()


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
