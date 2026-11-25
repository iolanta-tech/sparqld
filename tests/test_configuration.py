"""Exercise configuration through the HTTP endpoint."""

import json
from pathlib import Path

import pytest
from serving import ROOT, _available_port, _launch, _wait_for_queries


def _write_result(path: Path, value: str) -> None:
    path.write_text(json.dumps({
        'head': {'vars': ['value']},
        'results': {'bindings': [{'value': {'type': 'literal', 'value': value}}]},
    }))


@pytest.mark.parametrize('custom_config', [False, True])
def test_http_configuration_and_cli_overrides(
    sparqld_binary: Path,
    tmp_path: Path,
    custom_config: bool,
) -> None:
    """Load the documented settings, then override the directory and HTTP options."""
    configured_port = _available_port()
    template = ROOT / 'docs/examples/configuration/sparqld.toml'
    config = template.read_text().replace('port = 8080', f'port = {configured_port}')
    config_path = tmp_path / 'sparqld.toml'
    config_arguments = ()
    if custom_config:
        config_path.write_text('broken discovery file')
        config_arguments = ('-c', 'settings/selected.toml')
        config_path = tmp_path / config_arguments[1]
        config_path.parent.mkdir()
    config_path.write_text(config)

    data = config_path.parent / 'data'
    data.mkdir()
    (data / 'included.ttl').write_text('<urn:subject> <urn:predicate> "config" .')
    (data / 'skipped.nt').write_text('<urn:subject> <urn:predicate> "skipped" .')
    explicit = tmp_path / 'explicit'
    explicit.mkdir()
    (explicit / 'included.nt').write_text('<urn:subject> <urn:predicate> "explicit" .')
    (explicit / 'skipped.ttl').write_text('<urn:subject> <urn:predicate> "skipped" .')

    query = tmp_path / 'values.rq'
    query.write_text('SELECT ?value WHERE { <urn:subject> <urn:predicate> ?value }')
    queries = (('values', query),)
    results = tmp_path / 'expected'
    result = results / 'values/results.json'
    result.parent.mkdir(parents=True)
    _write_result(result, 'config')

    with _launch(
        sparqld_binary, tmp_path / 'config.log', configured_port, (*config_arguments, 'http'), tmp_path,
    ) as server:
        _wait_for_queries(server, queries, results)

    override_port = _available_port()
    while override_port == configured_port:
        override_port = _available_port()
    config_path.write_text(config.replace('127.0.0.1', 'invalid address'))
    arguments = (
        *config_arguments,
        '--directory', 'explicit', '--pattern', '*.nt',
        'http', '--host', '127.0.0.1', '--port', str(override_port), '--watch',
    )
    _write_result(result, 'explicit')
    with _launch(
        sparqld_binary, tmp_path / 'override.log', override_port, arguments, tmp_path,
    ) as server:
        _wait_for_queries(server, queries, results)
        (explicit / 'included.nt').write_text('<urn:subject> <urn:predicate> "reloaded" .')
        _write_result(result, 'reloaded')
        _wait_for_queries(server, queries, results)
