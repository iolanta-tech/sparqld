---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/libraries/comunica.md
"@type": schema:TechArticle
name: Query sparqld with Comunica
title: Comunica
description: Run a SELECT query against sparqld with Node.js and Comunica.
---

# :simple-javascript: Comunica [:material-open-in-new:](https://comunica.dev/docs/query/getting_started/query_app/)

{{ source('docs/reference/libraries/comunica.mjs', title='Example') }}

{{ shell('node docs/reference/libraries/comunica.mjs', env={'PORT': sparqld_port}) }}
