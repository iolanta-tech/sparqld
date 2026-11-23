"""Exercise graph discovery and offline GitHub snapshot refreshes."""

# flake8: noqa: WPS202

import json
from email.message import Message
from http import HTTPStatus
from http.client import IncompleteRead
from pathlib import Path
from unittest.mock import Mock, call
from urllib.error import HTTPError

import pytest
import typer
from ruamel.yaml import YAML

from jeeves import github_api, github_metadata

_API_ROOT = 'https://api.github.com/repos'
_IDENTIFIER = '@id'
_HTML_URL = 'html_url'
_LATEST_COMMIT = 'latestCommit'
_URL = 'url'
_RELEASE_PATH = '/releases/latest'
_COMMITS_PATH = '/commits?per_page=1'
_EMPTY_RELEASE = HTTPError('', HTTPStatus.NOT_FOUND, 'No releases', Message(), None)
_EMPTY_HISTORY = HTTPError('', HTTPStatus.CONFLICT, 'Empty repository', Message(), None)
_OLD_RECORD = 'observedAt: "2026-11-17T08:00:00Z"\nstars: 1\n'
_OLDER_REPO = 'owner/older'
_NEWER_REPO = 'owner/newer'
_RECENT_RECORD = 'observedAt: "2026-11-18T08:00:00Z"\nstars: 9\n'


@pytest.mark.integration
def test_discovery_uses_non_generated_sources(
    tmp_path: Path,
    sparqld_binary: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    yaml = YAML()
    yaml.dump(
        {
            '@graph': [
                {
                    _IDENTIFIER: 'urn:project',
                    'urn:issue': {
                        _IDENTIFIER: 'https://github.com/Owner/Repo/issues/3'
                    },
                    'urn:file': {
                        _IDENTIFIER: 'https://github.com/owner/repo/blob/main/index.md'
                    },
                    'urn:topic': {_IDENTIFIER: 'https://github.com/topics/python'},
                    'urn:description': 'https://github.com/literal/repo',
                },
                {
                    _IDENTIFIER: 'http://www.github.com/Other/Project',
                    'urn:relationship': {_IDENTIFIER: 'urn:project'},
                },
            ],
        },
        tmp_path / 'external.yamlld',
    )
    generated = tmp_path / 'results/github/removed'
    generated.mkdir(parents=True)
    yaml.dump(
        {_IDENTIFIER: 'https://github.com/removed/entry', 'urn:stars': 1},
        generated / 'entry.yamlld',
    )
    monkeypatch.setattr(github_metadata, '_DOCUMENTATION', tmp_path)
    monkeypatch.setattr(github_metadata, 'PROJECT_ROOT', sparqld_binary.parents[2])

    assert github_metadata._discover_repositories() == {'owner/repo', 'other/project'}


def _empty_repository(name: str) -> list:
    repo = {_HTML_URL: f'https://github.com/{name}', 'stargazers_count': 0}
    return [json.dumps(repo).encode(), _EMPTY_RELEASE, _EMPTY_HISTORY]


def _repository_calls(name: str) -> list:
    url = f'{_API_ROOT}/{name}'
    return [call(url), call(f'{url}{_RELEASE_PATH}'), call(f'{url}{_COMMITS_PATH}')]


@pytest.fixture
def old_snapshots(tmp_path: Path) -> Path:
    owner = tmp_path / 'owner'
    owner.mkdir()
    (owner / 'older.yamlld').write_text(_OLD_RECORD)
    (owner / 'newer.yamlld').write_text(_RECENT_RECORD)
    return owner


@pytest.mark.parametrize(
    'limit_status', [HTTPStatus.FORBIDDEN, HTTPStatus.TOO_MANY_REQUESTS]
)
def test_refresh_stops_on_rate_limit(
    old_snapshots: Path,
    monkeypatch: pytest.MonkeyPatch,
    limit_status: HTTPStatus,
) -> None:
    failure = HTTPError('', limit_status, 'Rate limit', Message(), None)
    fetch = Mock(side_effect=[*_empty_repository('owner/missing'), failure])
    monkeypatch.setattr(github_api, '_get', fetch)
    monkeypatch.setattr(
        github_metadata,
        '_discover_repositories',
        lambda: {_NEWER_REPO, _OLDER_REPO, 'owner/missing'},
    )
    monkeypatch.setattr(github_metadata, '_SNAPSHOTS', old_snapshots.parent)

    with pytest.raises(typer.Exit) as error:
        github_metadata.update_github()

    assert error.value.exit_code == 1
    assert fetch.call_args_list == [
        *_repository_calls('owner/missing'),
        call(f'{_API_ROOT}/owner/older'),
    ]
    assert (old_snapshots / 'older.yamlld').read_text() == _OLD_RECORD
    assert (old_snapshots / 'newer.yamlld').read_text() == _RECENT_RECORD


def test_refresh_continues_after_incomplete_read(
    old_snapshots: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fetch = Mock(
        side_effect=[
            *_empty_repository(_OLDER_REPO)[:2],
            IncompleteRead(b''),
            *_empty_repository(_NEWER_REPO),
        ]
    )
    monkeypatch.setattr(github_api, '_get', fetch)

    assert (
        github_metadata._refresh(
            {_OLDER_REPO, _NEWER_REPO},
            old_snapshots.parent,
        )
        is True
    )

    assert fetch.call_args_list == [
        *_repository_calls(_OLDER_REPO),
        *_repository_calls(_NEWER_REPO),
    ]
    assert (old_snapshots / 'older.yamlld').read_text() == _OLD_RECORD
    new = YAML(typ='safe').load(old_snapshots / 'newer.yamlld')
    assert (new[_IDENTIFIER], new['stars']) == ('https://github.com/owner/newer', 0)
    assert new.keys().isdisjoint({'latestRelease', _LATEST_COMMIT})


_CANONICAL_REPOSITORY = 'https://github.com/Owner/Repo'


def test_snapshot_keeps_long_url_on_one_line(tmp_path: Path) -> None:
    path = tmp_path / 'repo.yamlld'
    url = f'{_CANONICAL_REPOSITORY}/commit/0123456789abcdef0123456789abcdef01234567'
    url = url.replace('/Repo/', '/long-repository-name-for-line-wrapping/')

    github_metadata._write_snapshot(path, {_LATEST_COMMIT: {_URL: url}})

    assert f'  url: {url}' in path.read_text().splitlines()
    assert YAML(typ='safe').load(path)[_LATEST_COMMIT][_URL] == url


def test_capture_keeps_release_and_commit_data(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    release = f'{_CANONICAL_REPOSITORY}/releases/tag/1.2'
    sha = '0123456789abcdef0123456789abcdef01234567'
    commit = f'{_CANONICAL_REPOSITORY}/commit/{sha}'
    fetch = Mock(
        side_effect=[
            json.dumps(
                {_HTML_URL: _CANONICAL_REPOSITORY, 'stargazers_count': 4}
            ).encode(),
            json.dumps(
                {
                    'tag_name': '1.2',
                    _HTML_URL: release,
                    'published_at': '2026-11-17T10:00:00-04:00',
                }
            ).encode(),
            json.dumps(
                [
                    {
                        'sha': sha,
                        _HTML_URL: commit,
                        'commit': {'committer': {'date': '2026-11-18T12:00:00Z'}},
                    }
                ]
            ).encode(),
        ]
    )
    monkeypatch.setattr(github_api, '_get', fetch)

    assert github_metadata._refresh({'old-owner/old-repo'}, tmp_path) is False
    assert fetch.call_args_list == [
        call(f'{_API_ROOT}/old-owner/old-repo'),
        *_repository_calls('owner/repo')[1:],
    ]
    snapshot = YAML(typ='safe').load(tmp_path / 'old-owner/old-repo.yamlld')

    assert (snapshot[_IDENTIFIER], snapshot['stars']) == (_CANONICAL_REPOSITORY, 4)
    assert snapshot['latestRelease'] == {
        'tag': '1.2',
        _URL: release,
        'publishedAt': '2026-11-17T14:00:00Z',
    }
    assert snapshot[_LATEST_COMMIT] == {
        'sha': sha,
        _URL: commit,
        'committedAt': '2026-11-18T12:00:00Z',
    }
