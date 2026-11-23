"""Read repository metadata from GitHub's public REST API."""

# flake8: noqa: WPS202

from datetime import UTC, datetime
from http import HTTPStatus
from typing import Annotated
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from pydantic import (
    AliasPath,
    AwareDatetime,
    BaseModel,
    Field,
    StringConstraints,
    TypeAdapter,
)

from github_repository import repository_path

_API_ROOT = 'https://api.github.com/repos'
_REQUEST_TIMEOUT_SECONDS = 15
_GitHubPage = Annotated[
    str, StringConstraints(pattern=r'^https://github[.]com/[^\s]+$')
]


class _Repository(BaseModel):
    html_url: str
    stargazers_count: int = Field(ge=0, strict=True)


class _Release(BaseModel):
    tag_name: str = Field(min_length=1)
    html_url: _GitHubPage
    published_at: AwareDatetime

    def record(self) -> dict[str, str]:
        return {
            'tag': self.tag_name,
            'url': self.html_url,
            'publishedAt': _utc_timestamp(self.published_at),
        }


class _Commit(BaseModel):
    sha: str = Field(pattern=r'^[a-f0-9]{40}$')
    html_url: _GitHubPage
    committed_at: AwareDatetime = Field(
        validation_alias=AliasPath('commit', 'committer', 'date'),
    )

    def record(self) -> dict[str, str]:
        return {
            'sha': self.sha,
            'url': self.html_url,
            'committedAt': _utc_timestamp(self.committed_at),
        }


def fetch_snapshot(repository: str) -> dict:
    metadata = _Repository.model_validate_json(_get(f'{_API_ROOT}/{repository}'))
    canonical = repository_path(metadata.html_url)
    if canonical is None:
        raise ValueError(f'Invalid GitHub repository URL: {metadata.html_url}')
    canonical_endpoint = f'{_API_ROOT}/{canonical}'
    release = _latest_release(canonical_endpoint)
    commit = _latest_commit(canonical_endpoint)
    snapshot = {
        '@id': metadata.html_url,
        '@type': 'schema:SoftwareSourceCode',
        'stars': metadata.stargazers_count,
        'observedAt': _utc_timestamp(datetime.now(UTC)),
    }
    if release is not None:
        snapshot['latestRelease'] = release.record()
    if commit is not None:
        snapshot['latestCommit'] = commit.record()
    return snapshot


def _latest_release(endpoint: str) -> _Release | None:
    try:
        return _Release.model_validate_json(_get(f'{endpoint}/releases/latest'))
    except HTTPError as error:
        if error.code != HTTPStatus.NOT_FOUND:
            raise
        return None


def _latest_commit(endpoint: str) -> _Commit | None:
    try:
        commits = TypeAdapter(list[_Commit]).validate_json(
            _get(f'{endpoint}/commits?per_page=1')
        )
    except HTTPError as error:
        if error.code != HTTPStatus.CONFLICT:
            raise
        return None
    return next(iter(commits), None)


def _get(url: str) -> bytes:
    request = Request(
        url,
        headers={
            'Accept': 'application/vnd.github+json',
            'User-Agent': 'sparqld-documentation',
            'X-GitHub-Api-Version': '2022-11-28',
        },
    )
    with urlopen(request, timeout=_REQUEST_TIMEOUT_SECONDS) as response:
        return response.read()


def _utc_timestamp(timestamp: datetime) -> str:
    return (
        timestamp.astimezone(UTC).isoformat(timespec='seconds').replace('+00:00', 'Z')
    )
