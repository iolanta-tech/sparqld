# Codex

Derive a server name from the knowledge-base directory basename, such as
`sparqld-iolanta`.

Check that name first:

```sh
codex mcp get sparqld-iolanta
```

Reuse it only when it launches `sparqld mcp` for the same absolute directory.
A matching configuration under another name does not verify the selected tool.
If it points elsewhere, ask the user to choose a distinct name.

After explicit permission, add the configuration. Put `sparqld mcp` and its
arguments after `--`. Add flags only when the user needs them; take names and
shapes from `sparqld mcp --help`:

```sh
codex mcp add sparqld-iolanta -- sparqld mcp /absolute/path/to/data
```

Read the configuration back with `codex mcp get <server-name>` and confirm the
command and directory.

Ask the user to open a new Codex thread. Use the `query` tool from the server
configured for the selected directory.
