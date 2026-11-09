# Claude Code

Derive a server name from the knowledge-base directory basename, such as
`sparqld-iolanta`.

List configured servers and confirm whether that name already launches
`sparqld mcp` for the same absolute directory:

```sh
claude mcp list
```

Reuse it only when the command and directory match. If the name points
elsewhere, ask the user to choose a distinct name. Inspect one server with
`claude mcp get <server-name>` when more detail is needed.

After explicit permission, add a stdio server. Put Claude Code options before
`--` and `sparqld mcp` with its arguments after `--`. Add sparqld flags only
when the user needs them; take names and shapes from `sparqld mcp --help`.
Choose `--scope user` when the knowledge base should be available outside the
current project:

```sh
claude mcp add --transport stdio --scope user sparqld-iolanta -- sparqld mcp /absolute/path/to/data
```

Read the configuration back with `claude mcp get <server-name>` or
`claude mcp list` and confirm the command and directory.

Start a new Claude Code session if the tools are not yet available. Use the
`query` tool from the server configured for the selected directory.
