"""Project mutations for example result capture states."""

from __future__ import annotations

import json
from pathlib import Path

from ruamel.yaml import YAML

YAML_CODEC = YAML()


def drop_content_url(project: Path) -> None:
    """Remove the recursive-context registration triple from example.jsonld."""
    body = project / 'example.jsonld'
    document = json.loads(body.read_text())
    document.pop('http://schema.org/contentUrl')
    body.write_text(json.dumps(document))


def register_old(project: Path) -> None:
    """Register urn:context:second on body.yamlld with the old name term."""
    _dump_yaml(project / 'body.yamlld', _registered_body('urn:old'))


def register_new(project: Path) -> None:
    """Point registration at alternate.yamlld with the new name term."""
    _dump_yaml(project / 'body.yamlld', {'@context': {'name': 'urn:old'}})
    _dump_yaml(project / 'alternate.yamlld', _registered_body('urn:new'))


def register_ambiguous(project: Path) -> None:
    """Register the same context URL from both body and alternate files."""
    _dump_yaml(project / 'body.yamlld', _registered_body('urn:old'))
    _dump_yaml(project / 'alternate.yamlld', _registered_body('urn:new'))


def _dump_yaml(path: Path, document: dict) -> None:
    with path.open('w') as stream:
        YAML_CODEC.dump(document, stream)


def _registered_body(name_term: str) -> dict:
    return {
        '@context': {'name': name_term},
        '@type': 'http://www.w3.org/ns/json-ld#Context',
        'http://schema.org/contentUrl': {'@id': 'urn:context:second'},
    }
