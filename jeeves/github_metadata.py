"""Refresh checked-in GitHub metadata from public REST API responses."""

# flake8: noqa: WPS201, WPS202

from datetime import UTC, datetime
from http import HTTPStatus
from http.client import HTTPException
from pathlib import Path
from shutil import which
from sys import maxsize
from tempfile import NamedTemporaryFile, TemporaryDirectory
from urllib.error import HTTPError

import typer
from pydantic import AwareDatetime, BaseModel, Field
from ruamel.yaml import YAML, YAMLError
from ruamel.yaml.emitter import RoundTripEmitter

from github_repository import repository_path
from jeeves.github_api import fetch_snapshot
from jeeves.repository import PROJECT_ROOT
from jeeves.sparqld_serve import query, serve, wait_until_ready
from mkdocs_macros_sparqld.render import SelectResult

_DOCUMENTATION = PROJECT_ROOT / 'docs'
_DISCOVERY = _DOCUMENTATION / 'queries/github-repositories.rq'
_SNAPSHOTS = _DOCUMENTATION / 'results/github'
_REFRESH_ERRORS = (OSError, ValueError, YAMLError, HTTPException)


def _snapshot_context() -> dict:
    return {
        'schema': 'https://schema.org/',
        'xsd': 'http://www.w3.org/2001/XMLSchema#',
        'github': 'urn:sparqld:vocabulary:github:',
        'stars': 'github:stars',
        'observedAt': _date_time_term('github:observedAt'),
        'latestRelease': 'github:latestRelease',
        'tag': 'schema:version',
        'publishedAt': _date_time_term('schema:datePublished'),
        'latestCommit': 'github:latestCommit',
        'sha': 'schema:identifier',
        'committedAt': _date_time_term('github:committedAt'),
        'url': {'@id': 'schema:url', '@type': '@id'},
    }


def _date_time_term(predicate: str) -> dict[str, str]:
    return {'@id': predicate, '@type': 'xsd:dateTime'}


class _Observation(BaseModel):
    observed_at: AwareDatetime = Field(alias='observedAt')


class _SnapshotEmitter(RoundTripEmitter):
    def choose_scalar_style(self) -> str:
        style = super().choose_scalar_style()
        return '"' if style == "'" else style


def update_github() -> None:
    """Update snapshots for GitHub repositories referenced in the docs graph."""
    try:
        failed = _refresh(_discover_repositories(), _SNAPSHOTS)
    except (OSError, RuntimeError, ValueError) as error:
        typer.echo(str(error), err=True)
        raise typer.Exit(1) from error
    if failed:
        raise typer.Exit(1)


def _discover_repositories() -> set[str]:
    binary = _sparqld_binary()
    with TemporaryDirectory(prefix='sparqld-github-') as temporary:
        log = Path(temporary) / 'server.log'
        with serve(_DOCUMENTATION, binary, log) as server:
            wait_until_ready(server, (('github-repositories', _DISCOVERY),), log)
            return _parse_repositories(query(server.endpoint, _DISCOVERY))


def _sparqld_binary() -> Path:
    binary = PROJECT_ROOT / 'target/debug/sparqld'
    if not binary.is_file():
        installed = which('sparqld')
        if installed is None:
            raise RuntimeError('Build sparqld with cargo build or install it on PATH.')
        binary = Path(installed)
    return binary


def _parse_repositories(response: bytes) -> set[str]:
    solutions = SelectResult.model_validate_json(response)
    paths = (repository_path(row['repository']) for row in solutions.rows())
    return {path for path in paths if path is not None}


def _refresh(repositories: set[str], snapshots: Path) -> bool:
    failed = False
    targets = {repo: snapshots / f'{repo}.yamlld' for repo in sorted(repositories)}
    pending = sorted(targets, key=lambda repo: _last_observation(targets[repo]))
    for repository in pending:
        try:
            _write_snapshot(targets[repository], fetch_snapshot(repository))
        except HTTPError as error:
            typer.echo(f'{repository}: {error}', err=True)
            if error.code in {HTTPStatus.FORBIDDEN, HTTPStatus.TOO_MANY_REQUESTS}:
                return True
            failed = True
        except _REFRESH_ERRORS as error:
            typer.echo(f'{repository}: {error}', err=True)
            failed = True
        else:
            typer.echo(f'Updated {repository}')
    return failed


def _last_observation(path: Path) -> datetime:
    try:
        with path.open(encoding='utf-8') as snapshot:
            return _Observation.model_validate(
                YAML(typ='safe').load(snapshot)
            ).observed_at
    except (FileNotFoundError, ValueError, YAMLError):
        return datetime.min.replace(tzinfo=UTC)


def _write_snapshot(path: Path, snapshot: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    yaml = YAML()
    yaml.Emitter = _SnapshotEmitter
    yaml.default_flow_style = False
    yaml.width = maxsize
    with NamedTemporaryFile(
        mode='w',
        encoding='utf-8',
        dir=path.parent,
        delete_on_close=False,
    ) as stream:
        yaml.dump({'@context': _snapshot_context(), **snapshot}, stream)
        stream.close()
        Path(stream.name).replace(path)
