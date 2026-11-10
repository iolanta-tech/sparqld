---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/sparql/prefixes.md
"@type": schema:TechArticle
hide: [toc]
name: Prefixes
title: Prefixes
description: Prefix mappings available in sparqld SPARQL queries.
---

# :material-pound: Prefixes

SPARQL queries can use the prefixes from
[sparqld's default context](../contexts/default-context.md) without declaring
them in each query.

{{ default_prefix_table() }}

This query uses the default [`rdfs:`](http://www.w3.org/2000/01/rdf-schema#)
mapping without a `PREFIX` declaration:

{{ source('docs/examples/default-context/queries/comment.rq') }}

Result:

{{ stored_sparql('docs/examples/default-context/queries/comment.rq') | sparql_table }}

Declare other prefixes with `PREFIX` in the query. A query's own declaration
takes precedence over a default mapping with the same prefix.
