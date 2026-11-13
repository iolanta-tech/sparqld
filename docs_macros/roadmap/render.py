"""Render the roadmap query bindings as a Mermaid graph."""

from typing import Literal

from pydantic import BaseModel, ConfigDict

from docs_macros.roadmap.nodes import (
    Bug,
    Decision,
    Goal,
    PrerequisiteState,
    Position,
    Roadmap,
    RoadmapNode,
)
from mkdocs_macros_sparqld.table import SelectBindings


class RoadmapRow(BaseModel):
    """One validated row from the roadmap query."""

    model_config = ConfigDict(extra='ignore')

    roadmap_item: str
    title: str
    kind: Literal['goal', 'bug', 'position', 'decision']
    status: str | None = None
    group: str | None = None
    group_title: str | None = None
    blocker: str | None = None
    blocker_title: str | None = None
    blocker_kind: str | None = None
    contribution: str | None = None


def _node(row: RoadmapRow) -> RoadmapNode:
    node_types = {'goal': Goal, 'bug': Bug, 'position': Position}
    return node_types[row.kind](iri=row.roadmap_item, title=row.title,
                                status=row.status, group=row.group,
                                group_title=row.group_title)


def _edges(rows: tuple[RoadmapRow, ...]) -> frozenset[tuple[str, str]]:
    blockers = frozenset((row.blocker, row.roadmap_item) for row in rows if row.blocker)
    contributions = frozenset(
        (row.roadmap_item, row.contribution) for row in rows if row.contribution
    )
    return blockers | contributions


def _roadmap(bindings: SelectBindings) -> Roadmap:
    rows = tuple(RoadmapRow.model_validate(binding) for binding in bindings.rows)
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
        elif row.blocker_kind not in (None, '', 'goal'):
            raise ValueError(f'Unsupported blocker kind: {row.blocker_kind}')
    if any(endpoint not in labels for edge in edges for endpoint in edge):
        raise ValueError('Roadmap relationships require labeled endpoints')
    return Roadmap(
        nodes=tuple(item for _, item in sorted((nodes | decisions).items())),
        edges=edges,
    )


def render_roadmap(bindings: SelectBindings) -> str:
    """Render validated SELECT bindings as deterministic Mermaid."""
    return _roadmap(bindings).render()
