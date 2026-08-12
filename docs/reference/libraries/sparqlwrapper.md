---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/libraries/sparqlwrapper.md
"@type": schema:TechArticle
name: Query sparqld with SPARQLWrapper
title: SPARQLWrapper
description: Run a SELECT query against sparqld with Python and SPARQLWrapper.
---

# :simple-python: SPARQLWrapper [:material-open-in-new:](https://sparqlwrapper.readthedocs.io/en/latest/main.html)

{{ source('docs/reference/libraries/sparqlwrapper.py', title='Example') }}

{{ shell('python docs/reference/libraries/sparqlwrapper.py', env={'PORT': sparqld_port}) }}
