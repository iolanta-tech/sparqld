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

## Set up MCP

If that tool is unavailable:

1. Ask permission to check whether the installed executable supports MCP:

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
2. Ask permission before configuring the user's MCP client to launch:

   ```sh
   sparqld mcp /absolute/path/to/data
   ```

   The MCP client owns this stdio process and its tool connection.

## Codex

For an available tool, inspect its server's configuration with
`codex mcp get <server-name>` and verify the command and directory before using
that tool. A matching configuration under another name does not verify it.

When setting up a server, derive a name from the directory basename, such as
`sparqld-iolanta`. Check that name first:

```sh
codex mcp get sparqld-iolanta
```

Reuse it only when it launches `sparqld mcp` for the same absolute directory.
If it points elsewhere, ask the user to choose a distinct name. After explicit
permission, add the configuration:

```sh
codex mcp add sparqld-iolanta -- sparqld mcp /absolute/path/to/data
```

Ask the user to open a new Codex thread. Use the `query` tool from the server
configured for the selected directory.

## Query workflow

1. Discover names, types, predicates, and relationship directions with small
   `SELECT DISTINCT` queries before making semantic assumptions.
2. Run a focused `SELECT`, `ASK`, `CONSTRUCT`, or `DESCRIBE` query.
3. State only what the result establishes. Distinguish inferred conclusions
   from recorded facts.
4. Include every SPARQL query executed, including discovery queries, in the
   answer.
