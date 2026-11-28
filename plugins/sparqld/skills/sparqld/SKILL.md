---
name: sparqld
description: Configure or query a local sparqld knowledge base through MCP and report evidence-backed answers with the SPARQL used.
---

# `sparqld`

Use the `query` tool from the `sparqld` MCP server for the selected knowledge
base. It runs read-only SPARQL against the watched files.

Before any query, identify the intended knowledge-base directory and source
patterns. Ask for the absolute directory if unclear. Inspect the MCP entry for
the selected tool, even when the tool is already available. Resolve the config
path, served directory, and effective patterns as described below. Resolve any
mismatch before querying.

Do not fetch sparqld documentation from the network. Use this skill,
`sparqld --help`, `sparqld mcp --help`, and the live MCP tool schema.

## Configure a knowledge base

1. Look for an existing `sparqld.toml` in the intended project. Reuse it if it
   selects the intended directory and patterns. When asked to set up a project
   with no file, create one at its root. Preserve unrelated settings when
   updating an existing file.

   For example, to serve Turtle files under `data/` and skip `data/archive/`:

   ```toml
   directory = "data"
   patterns = ["**/*.ttl", "!archive/**"]
   ```

   Use the project's actual directory and requested patterns. Omit `patterns`
   when no filtering is needed. Paths in `patterns` are relative to the served
   directory. Prefix exclusions with `!`. An exclusion wins over an inclusion.

2. If the client lets you set the server's working directory, set it to the
   directory containing `sparqld.toml` and launch `sparqld mcp`. For a file
   with a different name or a client without that setting, launch
   `sparqld --config /absolute/path/to/project/sparqld.toml mcp`. A relative
   `--config` path resolves from the server's working directory. A relative
   `directory` in TOML resolves from the config file's parent.

3. Read back the selected MCP entry. Check its `cwd`, command, and arguments
   against the TOML file when present. An explicit `--directory` overrides the file and
   resolves from the server's working directory when relative. Any
   `--pattern` flags replace the entire TOML pattern list. Omit those flags
   when the file supplies the intended settings. Reload or restart the server
   after changing its settings. Query a known source graph to confirm it
   loads, and check a known excluded source if filtering matters.

## Set up the binary

If the `query` tool is unavailable:

1. Ask permission to check whether the installed executable supports MCP:

   ```sh
   sparqld mcp --help
   ```

   Check that `sparqld --help` lists `--config`. Place it before `mcp`.

   If `sparqld` is missing, ask permission to install it with:

   ```sh
   cargo install sparqld
   ```

   If `cargo` is also missing, explain that Rust and Cargo must be installed
   first and ask permission to install them with Rustup. Recheck
   `sparqld mcp --help` after installation.

   If `sparqld` lacks `mcp` or `--config`, ask permission to update it with:

   ```sh
   cargo install sparqld --force
   ```

   Recheck the command before adding the MCP configuration.
2. Prepare the project config as described above. Ask permission before
   changing the Codex registration. Derive a server name from the
   knowledge-base directory basename, such as `sparqld-data`.

   Check that name first:

   ```sh
   codex mcp get sparqld-data
   ```

   Reuse it when its effective directory and patterns match.
   A matching configuration under another name does not verify the selected
   tool. If it points elsewhere, ask the user to choose a distinct name.

   After explicit permission, add the configuration. Put `sparqld mcp` and its
   arguments after `--`:

   ```sh
   codex mcp add sparqld-data -- sparqld --config /absolute/path/to/project/sparqld.toml mcp
   ```

   Read the configuration back with `codex mcp get <server-name>` and confirm
   its effective directory and patterns.

   Ask the user to open a new Codex thread. Use the `query` tool from the
   server configured for the selected directory.

## Query workflow

1. Discover names, types, predicates, and relationship directions with small
   `SELECT DISTINCT` queries before making semantic assumptions.
2. Run a focused `SELECT`, `ASK`, `CONSTRUCT`, or `DESCRIBE` query.
3. State only what the result establishes. Distinguish inferred conclusions
   from recorded facts.
4. Include every SPARQL query executed, including discovery queries, in the
   answer.
