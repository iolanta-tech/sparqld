---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/sparql/index.md
"@type": schema:TechArticle
name: SPARQL cheat sheet
title: SPARQL cheat sheet
description: Practical SPARQL query patterns for exploring a sparqld endpoint.
---

# :material-database-search-outline: SPARQL cheat sheet

Every recipe is executed against the Quickstart dataset while this page is
built. Queries without `GRAPH` use the union of every named graph, including
the file catalog.

## :material-format-list-bulleted: Select values with prefixes

Declare prefixes once, then select named values from a specific set of source
graphs.

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

`CONSTRUCT` returns RDF assembled from matching bindings.

{{ source('docs/queries/construct-name.rq') }}

```turtle title="Result"
{{ stored_sparql('docs/queries/construct-name.rq') }}
```

## :material-counter: Count triples by named graph

Aggregate named graphs to inspect data and file-catalog triples separately.

{{ source('docs/queries/graph-counts.rq') }}

{{ stored_sparql('docs/queries/graph-counts.rq') | sparql_table }}

## :material-page-next-outline: Order and paginate

Always pair `LIMIT` and `OFFSET` with `ORDER BY` for stable pages.

{{ source('docs/queries/paginated-names.rq') }}

{{ stored_sparql('docs/queries/paginated-names.rq') | sparql_table }}
