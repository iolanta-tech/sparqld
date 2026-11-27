# Codex

Derive a server name from the knowledge-base directory basename, such as
`sparqld-data`.

Check that name first:

```sh
codex mcp get sparqld-data
```

Reuse it when its effective directory and patterns match the project.
A matching configuration under another name does not verify the selected tool.
If it points elsewhere, ask the user to choose a distinct name.

After explicit permission, add the configuration. Put the full `sparqld`
command after `--`. Put any shared `sparqld` options before `mcp`:

```sh
codex mcp add sparqld-data -- sparqld --config /absolute/path/to/project/sparqld.toml mcp
```

Read the configuration back with `codex mcp get <server-name>` and confirm its
effective directory and patterns.

Ask the user to open a new Codex thread. Use the `query` tool from the server
configured for the selected directory.
