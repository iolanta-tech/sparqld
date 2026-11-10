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

Consider two YAML-LD documents: one describes Alpha Centauri, and the other
describes a person whose comment mentions a planet in that system:

{{ example_data('default-context/project/alpha-centauri.yamlld') }}
{{ example_data('default-context/project/guillem-anglada-escude.yamlld') }}

Query the data:

{{ source('docs/examples/default-context/queries/triples.rq') }}

{{ stored_sparql('docs/examples/default-context/queries/triples.rq') | sparql_table }}

The query returns each predicate and value for this person in the default
graph. The default context defines the
[`rdfs` prefix](http://www.w3.org/2000/01/rdf-schema#) and maps `$type` to
[`rdf:type`](http://www.w3.org/1999/02/22-rdf-syntax-ns#type). It does not
define `name`, so JSON-LD drops that property from this source. The
[discovery announcement](https://www.eso.org/public/news/eso1629/) credits the
person with leading the team that found the planet. An
[overview of Alpha Centauri](https://science.nasa.gov/exoplanets/other-stars-other-worlds/our-nearest-celestial-neighbor-an-exotic-3-star-system/)
describes the planet's orbit around a star in that system.

Keyword aliases such as `$type` come from the
[default context](default-context.md). Vocabulary terms such as `name` need a
declared or registered context:
[project](project-contexts.md) or [remote](remote-contexts.md).
