# OpenCode

Derive a server name from the knowledge-base directory basename, such as
`sparqld-data`.

List configured servers and confirm the matching server name:

```sh
opencode mcp list
```

Reuse it when its effective directory and patterns match the project.
If the name points elsewhere, ask the user to choose a distinct name.

After explicit permission, add the server. Put the full `sparqld` command after
`--`. Put any shared `sparqld` options before `mcp`.
Use `--global` when the knowledge base should be available
outside the current project:

```sh
opencode mcp add sparqld-data --global -- sparqld --config /absolute/path/to/project/sparqld.toml mcp
```

Read the saved MCP entry in the OpenCode config file. Confirm its effective
directory and patterns. Use `opencode mcp list` to check the server status.

Start a new OpenCode session if the tools are not yet available. Use the
`query` tool from the server configured for the selected directory.
