---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/sparql/prefixes.md
"@type": schema:TechArticle
name: SPARQL prefixes
title: SPARQL prefixes
description: Prefix mappings available in sparqld SPARQL queries.
---

# :material-pound: SPARQL prefixes

SPARQL queries can use the prefixes from
[sparqld's default context](../contexts/default-context.md) without declaring
them in each query.

{{ default_prefix_table() }}

For example, this query uses `rdfs:` without a `PREFIX` declaration:

```sparql
SELECT ?comment
WHERE {
  ?subject rdfs:comment ?comment .
}
```

Declare other prefixes with `PREFIX` in the query. A query's own declaration
takes precedence over a default mapping with the same prefix.
