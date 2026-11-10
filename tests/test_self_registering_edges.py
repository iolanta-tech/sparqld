"""Edge cases for context registration outside the startup catalog."""

import select
import shutil
import socket
from pathlib import Path

import pytest
import registering
from serving import PROJECT_DIR, SERVER_LOG_NAME, _serve


def test_relative_context_uses_requested_uri(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """A source-relative import needs a canonical registration when reused remotely."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    body = registering.YamlDocument(project / registering.BODY)
    terms = body.context()[-1]
    body.context()[-1] = {'@import': 'terms.yamlld'}
    body.save()
    registering._dump(project / 'contexts/terms.yamlld', {'@context': terms})
    log = tmp_path / SERVER_LOG_NAME
    with _serve(project, sparqld_binary, log) as server:
        registering._wait_state(server, [], [registering.STAR_FILE])
        registering._install(
            project,
            'terms-registration.yamlld',
            'contexts/terms.yamlld',
            layout=registering.COMBINED,
        )
        registering._wait_state(server, [registering.STAR], [])


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
    definition = body.context()[-1]['name']
    body.context()[-1]['name'] = {'$id': definition}
    body.save()
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        registering._wait_state(
            server, [], [registering.STAR_FILE, registering.BODY], False
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
            'offline-alpha-centauri.yamlld',
            registering.STAR_FILE,
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
                [registering.STAR] if registered else [],
                [] if registered else [registering.STAR_FILE],
                False,
            )
            assert not select.select([listener], [], [], 0)[0]
    if not registered:
        assert 'context is not registered' in log.read_text()


def test_explicit_base_keeps_saved_context(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """An explicit @base on a saved context does not select a different body file."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    document = registering.YamlDocument(project / registering.BODY)
    node = document.body.get('@context')
    if isinstance(node, list):
        node[-1]['@base'] = 'https://example.org/'
    else:
        document.set_context(
            [*([node] if node else []), {'@base': 'https://example.org/'}]
        )
    document.save()
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        registering._wait_state(server, [registering.STAR], [])


def test_registration_must_be_selected(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """Excluded bodies remain readable as contexts, but contribute no registration."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    log = tmp_path / SERVER_LOG_NAME
    with _serve(
        project,
        sparqld_binary,
        log,
        arguments=('--pattern', f'!{registering.BODY}'),
    ) as server:
        registering._wait_state(server, [], [registering.STAR_FILE], False)
        document = registering.YamlDocument(project / registering.STAR_FILE)
        document.set_context(registering.BODY)
        document.save()
        registering._wait_state(server, [registering.STAR], [], False)
