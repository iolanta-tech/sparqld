"""Edge cases for context registration outside the startup catalog."""

import select
import shutil
import socket
from pathlib import Path

import pytest
import registering
from serving import PROJECT_DIR, SERVER_LOG_NAME, _serve


@pytest.mark.parametrize(registering.LAYOUT_NAME, registering.LAYOUTS)
def test_relative_context_uses_requested_uri(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """A source-relative import needs a canonical registration when reused remotely."""
    project = shutil.copytree(registering.SCENARIO / layout, tmp_path / PROJECT_DIR)
    body = registering.YamlDocument(project / registering.BODY)
    terms = body.context()[1]
    body.context()[1] = {'@import': 'terms.yamlld'}
    body.save()
    registering._dump(project / 'contexts/terms.yamlld', {'@context': terms})
    log = tmp_path / SERVER_LOG_NAME
    with _serve(project, sparqld_binary, log) as server:
        registering._wait_state(server, [], [registering.ALICE_FILE])
        registering._install(project, 'terms-registration.yamlld', layout=layout)
        registering._wait_state(server, [registering.ALICE], [])


def test_aliases_inside_term_definitions_fail(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """The imported dollar aliases do not replace term-definition keywords."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED,
        tmp_path / PROJECT_DIR,
    )
    body = registering.YamlDocument(project / registering.BODY)
    definition = body.context()[1]['context-file']
    definition['$id'] = definition.pop('@id')
    body.save()
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        registering._wait_state(
            server, [], [registering.ALICE_FILE, registering.BODY], False
        )


@pytest.mark.parametrize('registered', (False, True))
def test_http_context_stays_offline(
    sparqld_binary: Path,
    tmp_path: Path,
    registered: bool,
) -> None:
    """HTTP context URIs use project mappings or fail without contacting a listener."""
    project = tmp_path / PROJECT_DIR
    project.mkdir()
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        listener.listen()
        port = listener.getsockname()[1]
        registering._install(
            project,
            'offline-alice.yamlld',
            registering.ALICE_FILE,
            port=str(port),
        )
        if registered:
            registering._install(
                project,
                'offline-context.yamlld',
                'context.yamlld',
                port=str(port),
            )
        log = tmp_path / SERVER_LOG_NAME
        with _serve(project, sparqld_binary, log) as server:
            registering._wait_state(
                server,
                [registering.ALICE] if registered else [],
                [] if registered else [registering.ALICE_FILE],
                False,
            )
            assert not select.select([listener], [], [], 0)[0]
    if not registered:
        assert 'context is not registered' in log.read_text()


@pytest.mark.parametrize(registering.LAYOUT_NAME, registering.LAYOUTS)
def test_explicit_base_overrides_registration(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """An explicit document base overrides the source-directory registration base."""
    project = shutil.copytree(registering.SCENARIO / layout, tmp_path / PROJECT_DIR)
    if layout == registering.COMBINED:
        document = registering.YamlDocument(project / registering.BODY)
    else:
        document = registering.YamlDocument(project / registering.REGISTRATION)
    node = document.context()
    (node[1] if isinstance(node, list) else node)['@base'] = 'https://example.org/'
    document.save()
    log = tmp_path / SERVER_LOG_NAME
    with _serve(project, sparqld_binary, log) as server:
        registering._wait_state(server, [], [registering.ALICE_FILE], False)
    assert 'context-file is not a local file' in log.read_text()


@pytest.mark.parametrize(registering.LAYOUT_NAME, registering.LAYOUTS)
def test_registration_must_be_selected(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
) -> None:
    """Excluded bodies remain readable as contexts, but contribute no registration."""
    project = shutil.copytree(registering.SCENARIO / layout, tmp_path / PROJECT_DIR)
    if layout == registering.COMBINED:
        declaration = registering.BODY
    else:
        declaration = registering.REGISTRATION
    log = tmp_path / SERVER_LOG_NAME
    with _serve(
        project,
        sparqld_binary,
        log,
        arguments=('--pattern', f'!{declaration}'),
    ) as server:
        registering._wait_state(server, [], [registering.ALICE_FILE], False)
        document = registering.YamlDocument(project / registering.ALICE_FILE)
        document.set_context(registering.BODY)
        document.save()
        registering._wait_state(server, [registering.ALICE], [], False)
