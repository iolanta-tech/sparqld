---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/clients/sq/index.md
"@type": schema:TechArticle
name: Using sq with sparqld
title: sq
description: Configure the sq command-line client for a sparqld endpoint.
---

# :material-database-search-outline: `sq`

[`sq`](https://github.com/ktk/sq) accepts an endpoint URL for a one-off command:

{{ command('sq -e http://127.0.0.1:7737/ graphs') }}

## :material-file-cog-outline: Configure a project

Place `.sq.toml` in the project directory. `sq` also finds it in parent
directories, so the configuration applies throughout a repository.

{{ client_data('sq/sq.toml', title='.sq.toml') }}

The `default` value selects the endpoint when `-e` is omitted. `data`
abbreviates the `sparqld:` graph names generated from source paths. The
project-specific `schema` value matches the HTTPS IRIs in the example data.

These commands now use the configured endpoint:

```console
sq graphs
sq any
sq -f docs/queries/names.rq
```

## :material-code-braces: Use configured prefixes

`sq` injects configured prefixes together with its built-in prefix set before
sending a query. This query uses both project-specific prefixes:

{{ source('docs/queries/sq-named-graph.rq') }}

{{ command('sq -f docs/queries/sq-named-graph.rq') }}

Add `-v` to a command, as in `sq -v graphs`, to see the selected endpoint and
expanded query. Use `sq endpoints` to list configured endpoints.
