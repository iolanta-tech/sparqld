"""Render GitHub metadata from saved repository captures."""

import re
from datetime import UTC
from urllib.parse import quote

from pydantic import AwareDatetime, BaseModel, Field
from ruamel.yaml import YAML

from docs_macros.locations import RESULTS_DIR
from github_repository import repository_path

GITHUB_URL = 'https://github.com/'
GITHUB_RESULTS_DIR = RESULTS_DIR / 'github'
_LINK_LABEL_SPECIAL = re.compile(r'[^a-zA-Z0-9 .-]')


class _Release(BaseModel):
    tag: str
    url: str
    published_at: AwareDatetime | None = Field(default=None, alias='publishedAt')


class _Repository(BaseModel):
    stars: int | None = Field(default=None, ge=0, strict=True)
    observed_at: AwareDatetime | None = Field(default=None, alias='observedAt')
    latest_release: _Release | None = Field(default=None, alias='latestRelease')


def github_stars(repository: str) -> str:
    """Link the captured star count to the repository's stargazers."""
    name = _repository_name(repository)
    snapshot = _read_capture(name)
    if snapshot.stars is None:
        return ''
    observed = _utc_date(snapshot.observed_at)
    title = f' "Observed {observed}"' if observed else ''
    return f'[{snapshot.stars:,}]({GITHUB_URL}{name}/stargazers{title})'


def github_latest_release(repository: str) -> str:
    """Link the captured release tag to its release page."""
    release = _read_capture(_repository_name(repository)).latest_release
    if release is None:
        return ''
    if not release.url.startswith(GITHUB_URL):
        raise ValueError(f'Expected a GitHub release URL, got {release.url!r}')
    label = _LINK_LABEL_SPECIAL.sub(
        lambda match: f'&#{ord(match[0])};',
        release.tag,
    )
    url = quote(release.url, safe=':/%?=&#+@')
    published = _utc_date(release.published_at)
    if published:
        return f'[{label}<br/>{published}]({url} "Released {published}")'
    return f'[{label}]({url})'


def _repository_name(repository: str) -> str:
    url = repository if '://' in repository else f'{GITHUB_URL}{repository}'
    name = repository_path(url)
    if name is None:
        raise ValueError(f'Expected a GitHub repository, got {repository!r}')
    return name


def _read_capture(name: str) -> _Repository:
    path = GITHUB_RESULTS_DIR / f'{name}.yamlld'
    try:
        capture = YAML(typ='safe').load(path.read_text(encoding='utf-8'))
    except FileNotFoundError:
        return _Repository()
    return _Repository.model_validate(capture)


def _utc_date(timestamp: AwareDatetime | None) -> str:
    if timestamp is None:
        return ''
    return timestamp.astimezone(UTC).date().isoformat()
