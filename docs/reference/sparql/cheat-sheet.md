---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/sparql/cheat-sheet.md
"@type": schema:TechArticle
name: Cheat sheet
title: Cheat sheet
description: Practical SPARQL query patterns for exploring a sparqld endpoint.
---

# :material-database-search-outline: Cheat sheet

The build runs every recipe against the Quickstart dataset. Queries without
`GRAPH` use the union of every named graph, including the file catalog.

## :material-format-list-bulleted: Select values with prefixes

Declare a prefix once, then select names for three related subjects from the
shared default graph.

{{ source('docs/queries/names.rq') }}

{{ stored_sparql('docs/queries/names.rq') | sparql_table }}

## :material-link-variant-plus: Join related resources

Shared variables join statements about a star and its constellation.

{{ source('docs/queries/alpha-centauri.rq') }}

{{ stored_sparql('docs/queries/alpha-centauri.rq') | sparql_table }}

## :material-help-circle-outline: Keep optional values

`OPTIONAL` retains rows when a property is absent.

{{ source('docs/queries/optional-values.rq') }}

{{ stored_sparql('docs/queries/optional-values.rq') | sparql_table }}

## :material-filter-outline: Filter values

Functions such as `LCASE`, `STR`, and `CONTAINS` narrow matching bindings.

{{ source('docs/queries/filter-names.rq') }}

{{ stored_sparql('docs/queries/filter-names.rq') | sparql_table }}

## :material-check-decagram-outline: Ask whether data exists

`ASK` returns one boolean instead of a result table.

{{ source('docs/queries/ask-data.rq') }}

```text title="Result"
{{ stored_sparql('docs/queries/ask-data.rq') }}
```

## :material-graph-outline: Construct a graph

`CONSTRUCT` returns a graph in the Resource Description Framework (RDF)
format, assembled from matching bindings.

{{ source('docs/queries/construct-name.rq') }}

```turtle title="Result"
{{ stored_sparql('docs/queries/construct-name.rq') }}
```

## :material-counter: Count triples by named graph

Compare the triple counts for each example source file.

{{ source('docs/queries/graph-counts.rq') }}

{{ stored_sparql('docs/queries/graph-counts.rq') | sparql_table }}

## :material-page-next-outline: Order and paginate

Always pair `LIMIT` and `OFFSET` with `ORDER BY` for stable pages.

{{ source('docs/queries/paginated-names.rq') }}

{{ stored_sparql('docs/queries/paginated-names.rq') | sparql_table }}
