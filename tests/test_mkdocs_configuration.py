"""Exercise the MkDocs macro package with a live build-time endpoint."""

from pathlib import Path
from types import SimpleNamespace

import pytest
import tomllib

from mkdocs_macros_sparqld.configuration import configure
from mkdocs_macros_sparqld.render import SelectResult
from mkdocs_macros_sparqld.server import (
    ensure_endpoint,
    run_query,
    run_query_in,
    stop_server,
)

_ENCODING = 'utf-8'


def test_mkdocs_creates_missing_configuration(tmp_path: Path) -> None:
    """Leave a config file that later invocations can use."""
    configure(SimpleNamespace(project_dir=tmp_path, conf={}))

    settings = (tmp_path / 'sparqld.toml').read_text(encoding=_ENCODING)
    assert tomllib.loads(settings) == {'directory': 'docs'}


@pytest.mark.parametrize(
    ('directory_line', 'directory'),
    [('', 'docs'), ('directory = "dataset"\n', 'dataset')],
)
def test_mkdocs_uses_existing_configuration(
    sparqld_binary: Path,
    tmp_path: Path,
    request: pytest.FixtureRequest,
    directory_line: str,
    directory: str,
) -> None:
    """Read the shared configuration and allow queries over a separate directory."""
    settings = tmp_path / 'sparqld.toml'
    configuration = f'{directory_line}patterns = ["included.ttl"]\n'
    settings.write_text(configuration, encoding=_ENCODING)
    _write_sources(tmp_path / directory, 'configured')
    _write_sources(tmp_path / 'alternate', 'separate')
    query = 'SELECT ?value WHERE { <urn:subject> <urn:predicate> ?value }'
    env = SimpleNamespace(
        project_dir=tmp_path,
        conf={'extra': {'sparqld': {'binary': sparqld_binary}}},
    )
    request.addfinalizer(stop_server)
    configure(env)
    assert settings.read_text(encoding=_ENCODING) == configuration
    assert _bindings(run_query(query)[1]) == ['configured']
    assert _bindings(run_query_in(tmp_path / 'alternate', query)[1]) == ['separate']
    assert _bindings(run_query(query)[1]) == ['configured']


def test_mkdocs_rejects_empty_directory(
    sparqld_binary: Path,
    tmp_path: Path,
    request: pytest.FixtureRequest,
) -> None:
    """An explicitly empty path is invalid even if the docs directory exists."""
    (tmp_path / 'docs').mkdir()
    (tmp_path / 'sparqld.toml').write_text('directory = ""\n', encoding=_ENCODING)
    request.addfinalizer(stop_server)
    configure(
        SimpleNamespace(
            project_dir=tmp_path,
            conf={'extra': {'sparqld': {'binary': sparqld_binary}}},
        ),
    )
    with pytest.raises(RuntimeError, match='directory must not be empty'):
        ensure_endpoint()


def _write_sources(directory: Path, literal: str) -> None:
    directory.mkdir()
    (directory / 'included.ttl').write_text(
        f'<urn:subject> <urn:predicate> "{literal}" .',
        encoding=_ENCODING,
    )
    (directory / 'excluded.ttl').write_text(
        '<urn:subject> <urn:predicate> "excluded" .',
        encoding=_ENCODING,
    )


def _bindings(response: str) -> list[str]:
    return [row['value'] for row in SelectResult.model_validate_json(response).rows()]
