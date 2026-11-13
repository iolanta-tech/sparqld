"""MkDocs macros for project documentation."""

from typing import Any

from docs_macros.adr import adr_metadata, example_code, universe_metadata
from docs_macros.commands import command, shell
from docs_macros.compatibility import client_compatibility
from docs_macros.decisions import Decision, decision_log
from docs_macros.environment import EnvironmentMacros, default_prefix_table
from docs_macros.locations import CLIENT_COMPATIBILITY_PATH
from docs_macros.roadmap import RoadmapMacro

__all__ = ['CLIENT_COMPATIBILITY_PATH', 'Decision']


# @todo(roadmap-extracted-contributions): Include code-owned roadmap outcomes.
# description: >
#   Extend docs/queries/roadmap.rq to include Riddles contributions from an
#   externally managed sparqld endpoint over the repository root. Preserve
#   native roadmap records, mission contributions, and blocker direction.
#   Configure extra.sparqld.endpoint so the pluglet queries that endpoint
#   without owning its process lifecycle. Adapt Riddles headline and
#   urn:riddle:blockedBy predicates into the roadmap result columns.
# acceptance:
#   - Render native and extracted outcomes together from real endpoint results.
#   - Test cross-file dependencies between code annotations and native records.
# blocked-by:
#   - riddles-integration
#   - existing-sparqld-endpoint


def _register_source_macros(env: Any, macros: EnvironmentMacros) -> None:
    env.macro(macros.source)
    env.macro(default_prefix_table)
    env.macro(macros.example_data)
    env.macro(example_code)
    env.macro(macros.result_data)
    env.macro(macros.client_data)


def _register_page_macros(env: Any, macros: EnvironmentMacros) -> None:
    env.macro(RoadmapMacro(env).render, name='roadmap_mermaid')
    env.macro(adr_metadata)
    env.macro(universe_metadata)
    env.macro(decision_log)
    env.macro(shell)
    env.macro(client_compatibility)
    env.macro(command)
    env.macro(macros.agent_conversation)
    env.macro(macros.directory_tree)


def define_env(env: Any) -> None:
    """Register documentation macros."""
    macros = EnvironmentMacros(env)
    _register_source_macros(env, macros)
    _register_page_macros(env, macros)
