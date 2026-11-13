"""Render the roadmap query bindings as a Mermaid graph."""

from typing import Literal

from pydantic import BaseModel, ConfigDict

from docs_macros.roadmap.nodes import Bug, Goal, Roadmap, RoadmapNode
from mkdocs_macros_sparqld.table import SelectBindings


class RoadmapRow(BaseModel):
    """One validated row from the roadmap query."""

    model_config = ConfigDict(extra='ignore')

    roadmap_item: str
    title: str
    kind: Literal['goal', 'bug']
    blocker: str | None = None
    contribution: str | None = None


def _node(row: RoadmapRow) -> RoadmapNode:
    node_types = {'goal': Goal, 'bug': Bug}
    return node_types[row.kind](iri=row.roadmap_item, title=row.title)


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
    edges = _edges(rows)
    if any(endpoint not in labels for edge in edges for endpoint in edge):
        raise ValueError('Roadmap relationships require labeled endpoints')
    nodes = tuple(_node(row) for row in rows)
    unique_nodes = {node.iri: node for node in nodes}
    return Roadmap(
        nodes=tuple(unique_nodes[iri] for iri in sorted(unique_nodes)),
        edges=edges,
    )


def render_roadmap(bindings: SelectBindings) -> str:
    """Render validated SELECT bindings as deterministic Mermaid."""
    return _roadmap(bindings).render()
