"""Compare context bodies that carry their registration with separate declarations."""

import shutil
from pathlib import Path

import pytest
import registering
from serving import PROJECT_DIR, SERVER_LOG_NAME, _serve, _wait_for_queries


@pytest.mark.parametrize(registering.LAYOUT_NAME, registering.LAYOUTS)
def test_bootstrap(sparqld_binary: Path, tmp_path: Path, layout: str) -> None:
    """Late dollar registration and late person registration resolve on startup."""
    with _serve(
        registering.SCENARIO / layout,
        sparqld_binary,
        tmp_path / SERVER_LOG_NAME,
    ) as server:
        _wait_for_queries(server, registering.QUERIES, registering.SCENARIO / 'results')


@pytest.mark.parametrize(
    (registering.LAYOUT_NAME, 'change'),
    [
        pytest.param(layout, change, marks=registering.KNOWN_PANIC)
        if (layout, change) == (registering.SEPARATE, 'dollar')
        else (layout, change)
        for layout in registering.LAYOUTS
        for change in ('edit', 'delete', 'invalid', 'unregister', 'dollar')
    ],
)
def test_reload_and_restart(
    sparqld_binary: Path,
    tmp_path: Path,
    layout: str,
    change: str,
) -> None:
    """Live changes and cold starts reach the same state; restoring files recovers."""
    project = shutil.copytree(registering.SCENARIO / layout, tmp_path / PROJECT_DIR)
    state = registering.ReloadExpectation(project, layout, change)
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


@registering.KNOWN_PANIC
def test_missing_nested_dependency_on_startup(
    sparqld_binary: Path,
    tmp_path: Path,
) -> None:
    """A missing nested context should produce catalog errors during a cold start."""
    project = shutil.copytree(
        registering.SCENARIO / registering.SEPARATE, tmp_path / PROJECT_DIR
    )
    (project / registering.DOLLAR_BODY).unlink()
    log = tmp_path / SERVER_LOG_NAME
    with (
        _serve(project, sparqld_binary, log) as server,
        registering._recognize_panic(log),
    ):
        registering._wait_state(server, [], [registering.ALICE_FILE, registering.BODY])
