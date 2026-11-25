# OpenCode

Derive a server name from the knowledge-base directory basename, such as
`sparqld-data`.

List configured servers and confirm whether that name already launches
`sparqld mcp` for the same absolute directory:

```sh
opencode mcp list
```

Reuse it only when the command and directory match. If the name points
elsewhere, ask the user to choose a distinct name.

After explicit permission, add the server. Put the full `sparqld` command after
`--`. Put any shared `sparqld` options before `mcp`.
Use `--global` when the knowledge base should be available
outside the current project:

```sh
opencode mcp add sparqld-data --global -- sparqld --directory /absolute/path/to/data mcp
```

Read the configuration back with `opencode mcp list` and confirm the command and
directory.

Start a new OpenCode session if the tools are not yet available. Use the
`query` tool from the server configured for the selected directory.
