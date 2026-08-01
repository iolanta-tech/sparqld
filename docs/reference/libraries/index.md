---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/libraries/index.md
"@type": schema:TechArticle
hide: [toc]
name: SPARQL client libraries
title: Libraries
description: Live Python and JavaScript client-library examples for sparqld.
---

# :material-code-block-tags: Libraries

Use a SPARQL client library to keep HTTP serialization and result parsing out
of application code. These examples run `SELECT`, `ASK`, and `CONSTRUCT`
against a temporary `sparqld` endpoint during every documentation build.

{{ live_library_examples() }}

The Python example uses
[:simple-python: SPARQLWrapper](https://sparqlwrapper.readthedocs.io/en/latest/main.html).
The JavaScript example uses
[:simple-javascript: Comunica](https://comunica.dev/docs/query/getting_started/query_app/)
with an explicit `sparql` source type because `sparqld` does not publish a
Service Description.
