---
name: sparqld
description: Query a local sparqld knowledge base through MCP and report evidence-backed answers with the SPARQL used.
---

# `sparqld`

Use the `query` tool from the `sparqld` MCP server for the selected knowledge
base. It runs read-only SPARQL against the watched files.

Before any query, establish the intended knowledge-base directory; ask for its
absolute path if unclear. Verify that the configuration of the server backing
the selected tool launches `sparqld mcp` for that absolute directory, even when
the tool is already available. If the configuration cannot be verified or
targets another directory, resolve that before querying.

Do not fetch sparqld documentation from the network. Use this skill, the
client-specific file below, `sparqld mcp --help`, and the live MCP tool schema.

## Set up the binary

If the `query` tool is unavailable:

1. Ask permission to check whether the installed executable supports MCP:

   ```sh
   sparqld mcp --help
   ```

   Treat that help text as the source of truth for MCP flags such as
   `--pattern` and `--query-timeout`. Pass any needed flags when registering
   the server.

   If `sparqld` is missing, ask permission to install it with:

   ```sh
   cargo install sparqld
   ```

   If `cargo` is also missing, explain that Rust and Cargo must be installed
   first and ask permission to install them with Rustup. Recheck
   `sparqld mcp --help` after installation.

   If `sparqld` does not recognize `mcp`, ask permission to update it with:

   ```sh
   cargo install sparqld --force
   ```

   Recheck the command before adding the MCP configuration.
2. Identify the MCP client and follow the matching file. Ask permission before
   any configuration change.

| Client | File |
| --- | --- |
| Claude Code | [agents/claude-code.md](agents/claude-code.md) |
| Codex | [agents/codex.md](agents/codex.md) |
| Cursor | [agents/cursor.md](agents/cursor.md) |
| OpenCode | [agents/opencode.md](agents/opencode.md) |

If the client is not listed, find its MCP add or config mechanism and register
`sparqld mcp <absolute-directory>` with any flags taken from `sparqld mcp --help`.

## Query workflow

1. Discover names, types, predicates, and relationship directions with small
   `SELECT DISTINCT` queries before making semantic assumptions.
2. Run a focused `SELECT`, `ASK`, `CONSTRUCT`, or `DESCRIBE` query.
3. State only what the result establishes. Distinguish inferred conclusions
   from recorded facts.
4. Include every SPARQL query executed, including discovery queries, in the
   answer.
