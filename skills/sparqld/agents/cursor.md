# Cursor

Cursor has no `mcp add` CLI. Register the server by editing an MCP config file.

Prefer the user-scoped file `~/.cursor/mcp.json` so the knowledge base is
available across projects. Use the project file `.cursor/mcp.json` only when
the user wants the server limited to one repository.

Derive a server name from the knowledge-base directory basename, such as
`sparqld-iolanta`.

Read the chosen file first. Reuse an existing entry only when its `command` and
`args` launch `sparqld mcp` for the same absolute directory. If the name points
elsewhere, ask the user to choose a distinct name.

After explicit permission, merge a stdio entry. Keep `command` as `sparqld` and
put the subcommand, absolute directory, and any flags in `args`. Add flags only
when the user needs them; take names and shapes from `sparqld mcp --help`:

```json
{
  "mcpServers": {
    "sparqld-iolanta": {
      "command": "sparqld",
      "args": ["mcp", "/absolute/path/to/data"]
    }
  }
}
```

Preserve every unrelated server already present in the file. Write valid JSON.

Read the file back and confirm the entry. Ask the user to reload MCP servers or
restart the Agent chat so Cursor picks up the change. Use the `query` tool from
the server configured for the selected directory once it appears.
