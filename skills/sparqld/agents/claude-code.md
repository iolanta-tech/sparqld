# Claude Code

Derive a server name from the knowledge-base directory basename, such as
`sparqld-data`.

List configured servers and confirm the matching server name:

```sh
claude mcp list
```

Reuse it when its effective directory and patterns match the project.
If the name points elsewhere, ask the user to choose a distinct name. Inspect one server with
`claude mcp get <server-name>` when more detail is needed.

After explicit permission, add a stdio server. Put Claude Code options before
`--` and the full `sparqld` command after `--`. Put any shared `sparqld`
options before `mcp`.
Choose `--scope user` when the knowledge base should be available outside the
current project:

```sh
claude mcp add --transport stdio --scope user sparqld-data -- sparqld --config /absolute/path/to/project/sparqld.toml mcp
```

Read the configuration back with `claude mcp get <server-name>` and confirm
its effective directory and patterns.

Start a new Claude Code session if the tools are not yet available. Use the
`query` tool from the server configured for the selected directory.
