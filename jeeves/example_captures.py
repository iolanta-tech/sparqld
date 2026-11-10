"""Capture recipes for documented example SPARQL results."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from jeeves.example_mutations import (
    drop_content_url,
    register_ambiguous,
    register_new,
    register_old,
)
from jeeves.example_projects import CopiedProject, UnchangedProject
from jeeves.repository import PROJECT_ROOT

EXAMPLES = PROJECT_ROOT / 'docs/examples'
GOLDENS = PROJECT_ROOT / 'tests/examples'


@dataclass(frozen=True)
class Capture:
    """One project state and the queries to materialize under a results directory."""

    label: str
    crate_parent: Path
    queries: tuple[tuple[str, Path], ...]
    prepare: Callable[[Path], Path]


def captures() -> tuple[Capture, ...]:
    """Return every example state that should be materialized with vitrify."""
    return (*_static_captures(), *_recursive_captures(), *_reload_captures())


def _static_captures() -> tuple[Capture, ...]:
    default_context = EXAMPLES / 'default-context'
    self_registering = EXAMPLES / 'self-registering-context'
    failures = 'failures'
    return (
        Capture(
            label='default-context',
            crate_parent=GOLDENS / 'default-context/results',
            queries=_named_queries(default_context, ('triples',)),
            prepare=UnchangedProject(default_context / 'project'),
        ),
        Capture(
            label='self-registering-context',
            crate_parent=GOLDENS / 'self-registering-context/results',
            queries=_named_queries(
                self_registering,
                ('alpha-centauri', failures, 'registration'),
            ),
            prepare=UnchangedProject(self_registering / 'combined'),
        ),
    )


def _recursive_captures() -> tuple[Capture, ...]:
    recursive = EXAMPLES / 'recursive-context'
    failures = 'failures'
    names = ('declaration', failures, 'alpha-centauri')
    queries = _named_queries(recursive, names, dependencies=True)
    source = recursive / 'project'
    return (
        Capture(
            label='recursive-context/initial',
            crate_parent=GOLDENS / 'recursive-context/results/initial',
            queries=queries,
            prepare=UnchangedProject(source),
        ),
        Capture(
            label='recursive-context/unregistered',
            crate_parent=GOLDENS / 'recursive-context/results/unregistered',
            queries=queries,
            prepare=CopiedProject(source, drop_content_url),
        ),
    )


def _reload_captures() -> tuple[Capture, ...]:
    reload = EXAMPLES / 'context-reload'
    names = ('alpha-centauri', 'failures')
    queries = _named_queries(reload, names)
    source = reload / 'project'
    return (
        Capture(
            label='context-reload/unregistered',
            crate_parent=GOLDENS / 'context-reload/results/unregistered',
            queries=queries,
            prepare=UnchangedProject(source),
        ),
        Capture(
            label='context-reload/old',
            crate_parent=GOLDENS / 'context-reload/results/old',
            queries=queries,
            prepare=CopiedProject(source, register_old),
        ),
        Capture(
            label='context-reload/new',
            crate_parent=GOLDENS / 'context-reload/results/new',
            queries=queries,
            prepare=CopiedProject(source, register_new),
        ),
        Capture(
            label='context-reload/ambiguous',
            crate_parent=GOLDENS / 'context-reload/results/ambiguous',
            queries=_named_queries(reload, names, dependencies=True),
            prepare=CopiedProject(source, register_ambiguous),
        ),
    )


def _named_queries(
    scenario: Path,
    names: tuple[str, ...],
    *,
    dependencies: bool = False,
) -> tuple[tuple[str, Path], ...]:
    queries = tuple(
        (name, scenario.joinpath('queries', f'{name}.rq')) for name in names
    )
    if not dependencies:
        return queries
    shared = PROJECT_ROOT / 'docs/queries/context-dependencies.rq'
    return (*queries, ('dependencies', shared))
