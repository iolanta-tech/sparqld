---
name: sparqld
description: Query a local sparqld knowledge base through MCP and report evidence-backed answers with the SPARQL used.
---

# sparqld

Use the `sparqld.query` MCP tool whenever it is available. It runs read-only
SPARQL against the user's watched local knowledge base.

## Set up MCP

If `sparqld.query` is unavailable, do not start `sparqld mcp` from a shell: the
Codex MCP client owns the stdio process and its tool connection.

1. If the knowledge-base directory is not explicit, ask the user for its
   absolute path.
2. Check whether the installed executable supports MCP:

   ```sh
   sparqld mcp --help
   ```

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
3. Ask permission before modifying Codex's persistent MCP configuration. After
   permission, run:

   ```sh
   codex mcp add sparqld -- sparqld mcp /absolute/path/to/data
   ```

4. Ask the user to open a new Codex thread. The new thread starts the stdio
   server and makes `sparqld.query` available.

Use HTTP or `sq` only when the user explicitly provides an endpoint or chooses
that fallback.

## Query workflow

1. Discover names, types, predicates, and relationship directions with small
   `SELECT DISTINCT` queries before making semantic assumptions.
2. Run a focused `SELECT`, `ASK`, `CONSTRUCT`, or `DESCRIBE` query.
3. State only what the result establishes. Distinguish inferred conclusions
   from recorded facts.
4. Include every SPARQL query used in the answer.
