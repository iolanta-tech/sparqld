---
name: sparqld
description: Query a local sparqld knowledge base or write or review its YAML-LD data using sparqld MCP.
---

# `sparqld`

Use the `query` tool from the `sparqld` MCP server for the selected knowledge
base. It runs read-only SPARQL against the watched files.

Use the query workflow for questions about a knowledge base. Use the review
workflow to assess YAML-LD, including Markdown frontmatter. Use the write
workflow for `sparqld write` or requests to create or edit YAML-LD for a
sparqld knowledge base. MCP queries are read-only; write files locally.

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

## Write workflow

Read and apply the shared [YAML-LD rules](yaml-ld-rules.md).

1. Identify the target file and its served directory. Verify the MCP server
   watches that directory and includes the file in its source patterns. Inspect
   existing data and contexts through MCP and local files before editing.
2. Write YAML-LD that expresses the requested facts using the shared rules.
3. Check the effective context and intended consumers as described there.
4. Save the file locally and run the YAML-LD quote-style check described below.
5. Validate the loaded data only through the configured sparqld MCP
   query tool: inspect the catalog for load errors for that source and query
   its named graph for the expected subjects, types, predicates, literals, and
   IRI-valued objects. Verify removals as well as additions when editing. For
   context registration or reload changes, query the affected dependents too.
   A successful query or absence of an error alone does not prove the expected
   data loaded. Account for asynchronous reload; if MCP cannot establish the
   result, report that limitation rather than claiming validation.
6. Report the changed files, observed results, and SPARQL queries used. Do not
   substitute a parser, linter, external expansion tool, or test suite for MCP
   validation.

## Review workflow

1. Read the target source and the shared [YAML-LD rules](yaml-ld-rules.md).
   Identify its intended facts, consumers, served directory, and source graph.
2. Verify the configured MCP watches the directory and includes the source.
   Inspect catalog load errors and query the source graph for actual subjects,
   types, predicates, literals, and IRI-valued objects. Compare the results with
   the source and intended meaning; check relationships in both directions
   where needed. Account for asynchronous reload.
3. Review the source against the shared rules, including stable entity identity,
   vocabulary, context, and YAML syntax. For Markdown, inspect its YAML-LD
   frontmatter without treating the rest of the document as YAML.
4. Report numbered, actionable findings with source locations and evidence.
   Distinguish confirmed defects from uncertainty and report validation gaps.
   Include every SPARQL query executed. Do not edit the reviewed content unless
   asked to fix it.

### Quote-style check

Run yamllint on every file written by this workflow with the bundled
[quote configuration](yamllint.yaml). Resolve its absolute path from the
installed skill directory; do not resolve it from the knowledge-base directory.

```sh
yamllint --strict -c <absolute-skill-directory>/yamllint.yaml <written-files>
```

The configuration checks keys and values, requires double quotes when quoting
is needed, and includes `*.yamlld` in directory discovery. When passing a
directory, verify coverage with `yamllint -c <configuration> --list-files
<directory>` first.

If yamllint is unavailable, use `uvx --from yamllint yamllint` with the same
arguments. If neither executable is available, report the prerequisite and
resolve installation through the environment's package-management workflow
before completing the write. Fix quote alerts before reporting completion.
This source-style check supplements the mandatory MCP validation of loaded data.
