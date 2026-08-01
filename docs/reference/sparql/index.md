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
built. Queries without `GRAPH` use the union of all source graphs.

## :material-format-list-bulleted: Select values with prefixes

Declare prefixes once, then select named values from a specific set of source
graphs.

{{ live_query('names.rq') }}

## :material-link-variant-plus: Join related resources

Shared variables join statements about a star and its constellation.

{{ live_query('alpha-centauri.rq') }}

## :material-help-circle-outline: Keep optional values

`OPTIONAL` retains rows when a property is absent.

{{ live_query('optional-values.rq') }}

## :material-filter-outline: Filter values

Functions such as `LCASE`, `STR`, and `CONTAINS` narrow matching bindings.

{{ live_query('filter-names.rq') }}

## :material-check-decagram-outline: Ask whether data exists

`ASK` returns one boolean instead of a result table.

{{ live_query('ask-data.rq') }}

## :material-graph-outline: Construct a graph

`CONSTRUCT` returns RDF assembled from matching bindings.

{{ live_query('construct-name.rq') }}

## :material-counter: Count triples by source graph

Aggregate named graphs to inspect how data is distributed across files.

{{ live_query('graph-counts.rq') }}

## :material-page-next-outline: Order and paginate

Always pair `LIMIT` and `OFFSET` with `ORDER BY` for stable pages.

{{ live_query('paginated-names.rq') }}
