---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/clients/curl.md
"@type": schema:TechArticle
name: Query sparqld with curl
title: curl
description: Send SPARQL Protocol requests to sparqld with curl.
---

# :material-web: `curl`

[`curl`](https://curl.se/) sends SPARQL Protocol requests directly over HTTP.
Each example is executed during the documentation build.

=== "GET"

{{ shell(
    'curl --silent --show-error --fail-with-body --get "http://127.0.0.1:$PORT/" \\\n'
    ~ '  --data-urlencode "query=ASK { ?subject ?predicate ?object }"',
    env={'PORT': sparqld_port},
    indent=4,
) }}

=== "POST query body"

{{ shell(
    'curl --silent --show-error --fail-with-body "http://127.0.0.1:$PORT/" \\\n'
    ~ '  --header "Content-Type: application/sparql-query" \\\n'
    ~ '  --data-binary "ASK { ?subject ?predicate ?object }"',
    env={'PORT': sparqld_port},
    indent=4,
) }}

=== "POST form"

{{ shell(
    'curl --silent --show-error --fail-with-body "http://127.0.0.1:$PORT/" \\\n'
    ~ '  --data-urlencode "query=ASK { ?subject ?predicate ?object }"',
    env={'PORT': sparqld_port},
    indent=4,
) }}
