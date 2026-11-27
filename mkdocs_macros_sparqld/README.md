# mkdocs-macros-sparqld

[mkdocs-macros](https://mkdocs-macros-plugin.readthedocs.io/) pluglet that runs
[sparqld](https://github.com/iolanta-tech/sparqld) during `mkdocs build` /
`mkdocs serve` and expands SPARQL macros into rendered pages.

Full documentation:
[MkDocs integration](https://sparqld.iolanta.tech/reference/integrations/mkdocs/)

## Install

```console
pip install mkdocs-macros-sparqld
```

The pluglet requires a POSIX environment because it uses `sh` to start the
server. Install [`sparqld`](https://crates.io/crates/sparqld) 0.1.9 or later on
`PATH`, or point `extra.sparqld.binary` at the executable. Until 0.1.9 is
released, build the binary from the sparqld source checkout.

## Configuration

Register the pluglet with mkdocs-macros:

```yaml
plugins:
  - macros:
      modules: [mkdocs_macros_sparqld]
```

The pluglet reads `sparqld.toml` next to `mkdocs.yml`. If the file is missing,
the pluglet creates it with `directory = "docs"`. It uses the file's source
settings and uses `docs/` if the file omits `directory`. It starts a temporary
HTTP server on a free port on localhost.
The server loads the dataset once.

Optional overrides under `extra.sparqld`:

```yaml
extra:
  sparqld:
    directory: data                    # overrides sparqld.toml for MkDocs
    binary: target/debug/sparqld       # default: sparqld on PATH
```

If you set `extra.sparqld.directory`, the pluglet uses it to seed a new
`sparqld.toml`. An existing file keeps its contents.

## Macros and filters

- Use `sparql('SELECT …')` to run a SPARQL query.
- Use `stored_sparql('path/from/mkdocs/root.rq')` to run a query file relative
  to the MkDocs project root.
- Apply the `sparql_table` filter to render `SELECT` bindings as a Markdown table.
- Read the local endpoint port from the `sparqld_port` variable.

```markdown
{{ stored_sparql('docs/queries/names.rq') | sparql_table }}
```

## License

Apache-2.0 OR MIT
