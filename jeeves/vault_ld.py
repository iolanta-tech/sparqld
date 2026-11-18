"""Compare a pinned Vault-LD exporter with RDFLib JSON-LD on shared fixtures."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Annotated, Any
from urllib.parse import urlsplit

import rdflib
import typer
from pydantic import BaseModel, Field
from ruamel.yaml import YAML

from jeeves.repository import PROJECT_ROOT

DEFAULT_VAULT_LD = Path.home() / 'projects/vault-ld'

REVISION = '025e71be8d810e387dc451c920e05472d1c47170'
ROOT = (
    PROJECT_ROOT
    / 'docs/project/decisions/implement-live-file-backed-sparql-access-with/examples'
)
SCENARIOS = ('wiki-link', 'wiki-literal', 'vocabulary', 'explicit-terms')


class ContextDocument(BaseModel):
    context: dict[str, Any] = Field(alias='@context')


class Result(BaseModel):
    vault_ld_revision: str
    rdflib_version: str
    vault_ld: list[list[dict[str, str]]]
    json_ld: list[list[dict[str, str]]]
    same_triples: bool
    vault_ld_stderr: str


def validate_identifier(scenario: str, identifier: str):
    """Reject reserved example HTTP namespaces in the comparison fixtures."""
    parsed = urlsplit(identifier)
    hostname = (parsed.hostname or '').lower().rstrip('.')
    reserved = ('example.com', 'example.net', 'example.org', 'example')
    if parsed.scheme.lower() in ('http', 'https') and any(
        hostname == domain or hostname.endswith('.' + domain) for domain in reserved
    ):
        raise ValueError(f'{scenario}: reserved example identifier {identifier}')


def validate_context(scenario: str, value: Any):
    """Check context mappings, including namespace keys and nested definitions."""
    if isinstance(value, str):
        validate_identifier(scenario, value)
    elif isinstance(value, dict):
        for key, item in value.items():
            validate_identifier(scenario, key)
            validate_context(scenario, item)
    elif isinstance(value, list):
        for item in value:
            validate_context(scenario, item)


def validate_graph(scenario: str, graph: rdflib.Graph):
    """Check URI terms and literal datatype identifiers before saving results."""
    for triple in graph:
        for term in triple:
            if isinstance(term, rdflib.URIRef):
                validate_identifier(scenario, str(term))
            elif isinstance(term, rdflib.Literal) and term.datatype:
                validate_identifier(scenario, str(term.datatype))


def rows(graph: rdflib.Graph):
    """Keep IRI and literal kinds visible, including wiki-shaped IRI values."""
    triples = [
        [
            {
                'value': str(term),
                'kind': 'iri' if isinstance(term, rdflib.URIRef) else 'literal',
            }
            for term in triple
        ]
        for triple in graph
    ]
    return sorted(triples, key=lambda row: json.dumps(row, sort_keys=True))


def summary(result: Result):
    prefixes = {
        'https://dbpedia.org/resource/': 'dbr:',
        'https://dbpedia.org/ontology/': 'dbo:',
        'http://dbpedia.org/property/': 'dbp:',
        'urn:centauri:': 'centauri:',
        'https://schema.org/': 'schema:',
        'http://www.w3.org/1999/02/22-rdf-syntax-ns#type': 'rdf:type',
    }
    lines = []
    for key, heading in [('vault_ld', 'Vault-LD'), ('json_ld', 'JSON-LD (RDFLib)')]:
        lines.append(heading)
        for row in getattr(result, key):
            terms = []
            for term in row:
                value = term['value']
                if term['kind'] == 'literal':
                    value = json.dumps(value)
                else:
                    for namespace, prefix in prefixes.items():
                        if value.startswith(namespace):
                            value = prefix + value[len(namespace) :]
                            break
                terms.append(value)
            lines.append('  ' + ' '.join(terms))
    if result.vault_ld_stderr:
        lines.extend(['', result.vault_ld_stderr])
    return '\n'.join(lines) + '\n'


def vault_ld(
    vault_ld: Annotated[
        Path,
        typer.Option(
            '--vault-ld',
            help='Local Vault-LD checkout at the revision pinned by the verifier.',
            file_okay=False,
            resolve_path=True,
        ),
    ] = DEFAULT_VAULT_LD,
    check: Annotated[
        bool,
        typer.Option('--check', help='Compare generated results with saved outputs.'),
    ] = False,
):
    """Generate Vault-LD example outputs; use --check to verify saved outputs."""
    exporter = vault_ld / 'scripts/vault_to_rdf.py'
    if not exporter.is_file():
        raise typer.BadParameter(
            'Expected a Vault-LD checkout containing scripts/vault_to_rdf.py.',
            param_hint='--vault-ld',
        )
    revision = subprocess.check_output(
        ['git', '-C', str(vault_ld), 'rev-parse', 'HEAD'],
        text=True,
    ).strip()
    if revision != REVISION:
        raise typer.BadParameter(
            f'Expected Vault-LD revision {REVISION}, got {revision}.',
            param_hint='--vault-ld',
        )
    prepared = []
    for scenario in SCENARIOS:
        project = ROOT / scenario
        context = ContextDocument.model_validate_json(
            (project / 'context.jsonld').read_text()
        ).context
        validate_context(scenario, context)
        baseline = rdflib.Graph()
        for note in sorted(project.glob('*.md')):
            frontmatter = YAML(typ='safe').load(note.read_text().split('---', 2)[1])
            baseline.parse(
                data=json.dumps({'@context': context, **frontmatter}),
                format='json-ld',
                publicID=context['@base'],
            )
        validate_graph(scenario, baseline)
        prepared.append((scenario, project, baseline))
    results = []
    for scenario, project, baseline in prepared:
        with tempfile.TemporaryDirectory() as output:
            process = subprocess.run(
                [
                    sys.executable,
                    str(exporter),
                    str(project),
                    '--out-dir',
                    output,
                ],
                check=True,
                capture_output=True,
                text=True,
            )
            exported = rdflib.Graph()
            for filename in ('schema.ttl', 'data.ttl'):
                exported.parse(Path(output) / filename, format='turtle')
        validate_graph(scenario, exported)
        result = Result.model_validate(
            {
                'vault_ld_revision': revision,
                'rdflib_version': rdflib.__version__,
                'vault_ld': rows(exported),
                'json_ld': rows(baseline),
                'same_triples': set(exported) == set(baseline),
                'vault_ld_stderr': process.stderr.strip(),
            }
        )
        results.append((scenario, project, result))
    for scenario, project, result in results:
        expected = project / 'result.json'
        if not check:
            expected.write_text(result.model_dump_json(indent=2) + '\n')
            (project / 'result.txt').write_text(summary(result))
        else:
            assert result == Result.model_validate_json(expected.read_text()), scenario
            assert summary(result) == (project / 'result.txt').read_text(), scenario
        print(f'{scenario}: same_triples={result.same_triples}')
