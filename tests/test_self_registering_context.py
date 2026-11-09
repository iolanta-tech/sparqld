"""Exercise context bodies that carry their own registration."""

import shutil
from pathlib import Path

import pytest
import registering
from serving import PROJECT_DIR, SERVER_LOG_NAME, _serve, _wait_for_queries


def test_bootstrap(sparqld_binary: Path, tmp_path: Path) -> None:
    """Late dollar registration and late person registration resolve on startup."""
    with _serve(
        registering.SCENARIO / registering.COMBINED,
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as server:
        _wait_for_queries(server, registering.QUERIES, registering.SCENARIO / 'results')


@pytest.mark.parametrize(
    'change',
    ('edit', 'delete', 'invalid', 'unregister', 'dollar'),
)
def test_reload_and_restart(
    sparqld_binary: Path,
    tmp_path: Path,
    change: str,
) -> None:
    """Live changes and cold starts reach the same state; restoring files recovers."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    state = registering.ReloadExpectation(project, registering.COMBINED, change)
    live = tmp_path / 'live.log'
    with _serve(project, sparqld_binary, live) as server:
        registering._wait_state(server, [registering.ALICE], [])
        state.apply()
        with registering._recognize_panic(live):
            registering._wait_state(
                server,
                state.names,
                state.failures,
                state.registered,
            )
    with _serve(project, sparqld_binary, tmp_path / 'restart.log') as server:
        registering._wait_state(
            server,
            state.names,
            state.failures,
            state.registered,
        )
        state.target.write_text(state.original)
        registering._wait_state(server, [registering.ALICE], [])


@pytest.mark.parametrize(
    'cycle',
    ('self', 'mutual', pytest.param('recursive', marks=registering.KNOWN_PANIC)),
)
def test_cycles_fail_without_hanging(
    sparqld_binary: Path,
    tmp_path: Path,
    cycle: str,
) -> None:
    """Registration bootstrap cycles and context-reference recursion fail finitely."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    body = registering.YamlDocument(project / registering.BODY)
    if cycle == 'self':
        body.context()[0] = registering.CANONICAL
    elif cycle == 'mutual':
        dollar = registering.YamlDocument(project / registering.DOLLAR_BODY)
        dollar.set_context([registering.CANONICAL, dollar.context()])
        dollar.save()
    else:
        body.context().insert(0, 'person.yamlld')
    body.save()
    failures = [registering.ALICE_FILE, registering.BODY]
    if cycle == 'mutual':
        failures.append(registering.DOLLAR_BODY)
    with (
        _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server,
        registering._recognize_panic(tmp_path / SERVER_LOG_NAME),
    ):
        registering._wait_state(server, [], failures, False)


def test_missing_nested_dependency_on_startup(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """A missing nested context should produce catalog errors during a cold start."""
    project = shutil.copytree(
        registering.SCENARIO / registering.COMBINED, tmp_path / PROJECT_DIR
    )
    (project / registering.DOLLAR_BODY).unlink()
    with _serve(project, sparqld_binary, tmp_path / SERVER_LOG_NAME) as server:
        registering._wait_state(
            server, [], [registering.ALICE_FILE, registering.BODY], False
        )
