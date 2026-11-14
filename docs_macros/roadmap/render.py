"""Render the roadmap query bindings as a Mermaid graph."""

from typing import Literal

import networkx as nx
from pydantic import BaseModel, ConfigDict

from docs_macros.roadmap.nodes import (
    Bug,
    Decision,
    Goal,
    Position,
    PrerequisiteState,
    Roadmap,
    RoadmapNode,
    Task,
)
from mkdocs_macros_sparqld.table import SelectBindings


class RoadmapRow(BaseModel):
    """One validated row from the roadmap query."""

    model_config = ConfigDict(extra='ignore')

    roadmap_item: str
    title: str
    kind: Literal['goal', 'bug', 'task', 'position', 'decision']
    status: str | None = None
    group: str | None = None
    group_title: str | None = None
    blocker: str | None = None
    blocker_title: str | None = None
    blocker_kind: str | None = None
    contribution: str | None = None


def _node(row: RoadmapRow) -> RoadmapNode:
    node_types = {'goal': Goal, 'bug': Bug, 'task': Task, 'position': Position}
    return node_types[row.kind](iri=row.roadmap_item, title=row.title,
                                status=row.status, group=row.group,
                                group_title=row.group_title)


def _edges(rows: tuple[RoadmapRow, ...]) -> frozenset[tuple[str, str]]:
    blockers = frozenset((row.blocker, row.roadmap_item) for row in rows if row.blocker)
    contributions = frozenset(
        (row.roadmap_item, row.contribution) for row in rows if row.contribution
    )
    return blockers | contributions


def _reduce_edges(edges: frozenset[tuple[str, str]]) -> frozenset[tuple[str, str]]:
    """Reduce a directed acyclic roadmap while preserving reachability."""
    graph = nx.DiGraph(sorted(edges))
    try:
        reduced = nx.transitive_reduction(graph)
    except nx.NetworkXError as error:
        cycle = nx.find_cycle(graph)
        path = ' -> '.join([cycle[0][0], *(target for _, target in cycle)])
        raise ValueError(
            f'Roadmap relationships contain a cycle: {path}. '
            + 'Remove a blocker or contribution relationship to make the roadmap acyclic.'
        ) from error
    return frozenset(reduced.edges)


def _roadmap(bindings: SelectBindings, target_iri: str | None = None) -> Roadmap:
    rows = tuple(RoadmapRow.model_validate(binding) for binding in bindings.rows)
    completed_tasks = {
        row.roadmap_item for row in rows
        if row.kind in ('task', 'bug') and row.status and row.status.lower() == 'closed'
    }
    known_items = {row.roadmap_item for row in rows}
    if target_iri is not None and target_iri not in known_items:
        raise ValueError(f'Unknown roadmap target: {target_iri}')
    rows = tuple(
        row.model_copy(update={
            'blocker': None if row.blocker in completed_tasks else row.blocker,
            'contribution': None if row.contribution in completed_tasks else row.contribution,
        })
        for row in rows if row.roadmap_item not in completed_tasks
    )
    if target_iri is not None:
        predecessors: dict[str, set[str]] = {}
        positions = {
            row.roadmap_item: row.group
            for row in rows if row.kind == 'position' and row.group
        }
        options = {
            row.group: {
                position for position, group in positions.items() if group == row.group
            }
            for row in rows if row.kind == 'position' and row.group
        }
        for row in rows:
            if row.blocker:
                predecessors.setdefault(row.roadmap_item, set()).add(row.blocker)
            if row.contribution:
                predecessors.setdefault(row.contribution, set()).add(row.roadmap_item)
        included = {target_iri}
        pending = [target_iri]
        while pending:
            item = pending.pop()
            related = set()
            if item in positions:
                related.add(positions[item])
            related.update(options.get(item, ()))
            for related_item in related:
                if related_item not in included:
                    included.add(related_item)
                    pending.append(related_item)
            for predecessor in predecessors.get(item, ()):
                if predecessor not in included:
                    included.add(predecessor)
                    pending.append(predecessor)
        rows = tuple(
            row.model_copy(update={
                'contribution': row.contribution if row.contribution in included else None,
                'blocker': row.blocker if row.blocker in included else None,
            })
            for row in rows
            if row.roadmap_item in included
            or (row.kind == 'position' and row.group in included)
        )
    labels = {row.roadmap_item: row.title for row in rows}
    if any(row.title != labels[row.roadmap_item] for row in rows):
        raise ValueError('Conflicting roadmap titles for one IRI')
    metadata = {
        row.roadmap_item: (row.kind, row.status, row.group, row.group_title)
        for row in rows
    }
    if any(
        (row.kind, row.status, row.group, row.group_title) != metadata[row.roadmap_item]
        for row in rows
    ):
        raise ValueError('Conflicting roadmap metadata for one IRI')
    decision_ids = {row.roadmap_item for row in rows if row.kind == 'decision'}
    if any(row.group not in decision_ids for row in rows if row.kind == 'position'):
        raise ValueError('Roadmap positions require a decision')
    edges = _edges(rows)
    nodes = {
        row.roadmap_item: _node(row)
        for row in rows if row.kind not in ('decision', 'position')
    }
    positions = {
        row.roadmap_item: Position(iri=row.roadmap_item, title=row.title, status=row.status)
        for row in rows if row.kind == 'position'
    }
    decisions = {
        row.roadmap_item: Decision(
            iri=row.roadmap_item,
            title=row.title,
            positions=tuple(
                positions[iri] for iri in sorted(positions)
                if any(item.roadmap_item == iri and item.group == row.roadmap_item for item in rows)
            ),
        )
        for row in rows if row.kind == 'decision'
    }
    for row in rows:
        if not row.blocker:
            continue
        if row.blocker_kind == 'state':
            if not row.blocker_title:
                raise ValueError('Prerequisite states require labels')
            existing_title = labels.get(row.blocker)
            if existing_title and existing_title != row.blocker_title:
                raise ValueError('Conflicting roadmap titles for one IRI')
            labels[row.blocker] = row.blocker_title
            nodes[row.blocker] = PrerequisiteState(
                iri=row.blocker,
                title=row.blocker_title,
            )
        elif row.blocker_kind not in (None, '', 'goal', 'bug', 'task', 'position'):
            raise ValueError(f'Unsupported blocker kind: {row.blocker_kind}')
    if any(endpoint not in labels for edge in edges for endpoint in edge):
        raise ValueError('Roadmap relationships require labeled endpoints')
    return Roadmap(
        nodes=tuple(item for _, item in sorted((nodes | decisions).items())),
        edges=_reduce_edges(edges),
    )


def render_roadmap(bindings: SelectBindings, target_iri: str | None = None) -> str:
    """Render validated SELECT bindings as deterministic Mermaid."""
    return _roadmap(bindings, target_iri).render()
