---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/contexts/index.md
"@type": schema:TechArticle
icon: material/code-braces-box
hide: [toc]
name: Contexts
title: Contexts
description: How sparqld resolves JSON-LD contexts for sources in the served directory.
---

# :material-code-braces-box: Contexts

{{ example_data('default-context/project/alpha-centauri.yamlld') }}

{{ source('docs/queries/contexts/default-context-triples.rq') }}

{{ stored_sparql('docs/queries/contexts/default-context-triples.rq') | sparql_table }}

This document lacks what's called a Context. Or does it?
