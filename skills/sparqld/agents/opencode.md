# OpenCode

Derive a server name from the knowledge-base directory basename, such as
`sparqld-iolanta`.

List configured servers and confirm whether that name already launches
`sparqld mcp` for the same absolute directory:

```sh
opencode mcp list
```

Reuse it only when the command and directory match. If the name points
elsewhere, ask the user to choose a distinct name.

After explicit permission, add the server. Put `sparqld mcp` and its arguments
after `--`. Add flags only when the user needs them; take names and shapes from
`sparqld mcp --help`. Use `--global` when the knowledge base should be available
outside the current project:

```sh
opencode mcp add sparqld-iolanta --global -- sparqld mcp /absolute/path/to/data
```

Read the configuration back with `opencode mcp list` and confirm the command and
directory.

Start a new OpenCode session if the tools are not yet available. Use the
`query` tool from the server configured for the selected directory.
