"""Helpers for the self-registering context examples."""

import time
from collections.abc import Generator
from contextlib import contextmanager, suppress
from pathlib import Path
from typing import Any

import pytest
from ruamel.yaml import YAML
from serving import WAIT_SECONDS, Server, _query

from mkdocs_macros_sparqld.render import SelectResult

SCENARIO = (
    Path(__file__).resolve().parents[1] / 'docs/examples/self-registering-context'
)
CANONICAL = 'https://example.org/contexts/combined/person'
BODY = 'contexts/person.yamlld'
DOLLAR_BODY = 'contexts/z-dollar.yamlld'
REGISTRATION = 'contexts/z-registration.yamlld'
COMBINED = 'combined'
SEPARATE = 'separate'
LAYOUT_NAME = 'layout'
ALICE = 'Alice'
ALICE_FILE = 'alice.yamlld'
LAYOUTS = (COMBINED, SEPARATE)
VARIANTS = SCENARIO / 'variants'
YAML_CODEC = YAML()
PANIC = 'The remote context stack is invalid'


def _dump(path: Path, document: dict) -> None:
    with path.open('w') as stream:
        YAML_CODEC.dump(document, stream)


QUERIES = tuple(
    (name, SCENARIO / 'queries' / f'{name}.rq')
    for name in ('alice', 'failures', 'registration')
)


class ContextStackPanic(RuntimeError):
    """A verified oxjsonld failure rather than a generic endpoint timeout."""


class YamlDocument:
    """A YAML file the tests load, edit, and write back."""

    def __init__(self, path: Path) -> None:
        """Load one YAML document from `path`."""
        self.path = path
        self.body = YAML_CODEC.load(path.read_text())

    def context(self) -> Any:
        """Return the document's `@context` value."""
        return self.body['@context']

    def set_context(self, context_value: Any) -> None:
        """Replace the document's `@context` value."""
        self.body['@context'] = context_value

    def save(self) -> None:
        """Write the document back to its path."""
        _dump(self.path, self.body)


class ReloadExpectation:
    """Files and catalog state for one live edit followed by a restart."""

    def __init__(self, project: Path, layout: str, change: str) -> None:
        """Record the file that `change` will edit and the initial catalog."""
        declaration = BODY if layout == COMBINED else REGISTRATION
        if change == 'dollar':
            target = project / DOLLAR_BODY
        else:
            target = project / BODY
        if change == 'unregister':
            target = project / declaration
        self.target = target
        self.original = target.read_text()
        self.failures: list[str] = []
        self.names: list[str] = []
        self.registered = layout == SEPARATE
        self.change = change

    def apply(self) -> None:
        """Perform the edit and record the catalog state it should reach."""
        if self.change == 'edit':
            document = YamlDocument(self.target)
            document.context()[1]['name'] = 'https://schema.org/alternateName'
            document.save()
            self.registered = True
            return
        if self.change == 'unregister':
            document = YamlDocument(self.target)
            document.body.pop('context-file')
            document.save()
            self.failures = [ALICE_FILE]
            self.registered = False
            return
        if self.change == 'delete':
            self.target.unlink()
            self.failures = [ALICE_FILE]
            return
        if self.change == 'invalid':
            self.target.write_text('"@context": [')
            self.failures = [ALICE_FILE, BODY]
            return
        self.target.unlink()
        self.failures = [ALICE_FILE, BODY]


KNOWN_PANIC = pytest.mark.xfail(
    strict=True,
    raises=ContextStackPanic,
    reason='oxjsonld 0.2.5 context stack assertion',
)


def _install(
    project: Path,
    template: str,
    destination: str | None = None,
    **replacements: str,
) -> None:
    text = (VARIANTS / template).read_text()
    for token, replacement in replacements.items():
        text = text.replace(f'{{{token}}}', replacement)
    (project / (destination or template)).write_text(text)


def _wait_state(
    server: Server,
    names: list[str],
    failures: list[str],
    registered: bool = True,
) -> None:
    deadline = time.monotonic() + WAIT_SECONDS
    actual = []
    expected = [
        [{'name': name} for name in names],
        [{'file': f'sparqld:{path}'} for path in sorted(failures)],
        registered,
    ]
    while time.monotonic() < deadline:
        assert server.process.poll() is None
        with suppress(OSError):
            responses = [_query(server.endpoint, query) for _, query in QUERIES]
            actual = [
                response.rows()
                if isinstance(response, SelectResult)
                else response.boolean
                for response in responses
            ]
            if actual == expected:
                return
        time.sleep(0.1)
    pytest.fail(f'Expected {expected}; got {actual}')


@contextmanager
def _recognize_panic(log: Path) -> Generator[None]:
    try:
        yield
    except pytest.fail.Exception:
        if PANIC in log.read_text():
            raise ContextStackPanic(log.read_text()) from None
        raise
