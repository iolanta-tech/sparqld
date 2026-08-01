---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/api/index.md
"@type": schema:TechArticle
name: sparqld HTTP API
title: HTTP API
description: SPARQL Protocol request and response behavior exposed by sparqld.
---

# :material-api: HTTP API

`sparqld` exposes one read-only SPARQL query endpoint at `/`.

## :material-send: Query requests

The endpoint accepts the SPARQL Protocol GET form and both standard POST forms.
Each example below is executed during the documentation build.

{{ live_api_examples() }}

A GET request without a query returns a plain-text landing response. The root
does not currently publish a SPARQL Service Description, so clients that infer
source types may need to be told explicitly that the URL is a SPARQL endpoint.

## :material-tray-arrow-down: Responses

| Query form | Content type | Body |
| --- | --- | --- |
| `SELECT` | `application/sparql-results+json` | SPARQL Query Results JSON |
| `ASK` | `application/sparql-results+json` | SPARQL Query Results JSON |
| `CONSTRUCT` | `text/turtle` | Turtle graph |
| `DESCRIBE` | `text/turtle` | Turtle graph |

The response serialization is selected by the query form. `Accept` headers do
not currently negotiate another representation.

## :material-alert-circle-outline: Errors and limits

| Status | Condition |
| --- | --- |
| `400 Bad Request` | Missing, repeated, malformed, or invalid query |
| `404 Not Found` | Request path other than `/` |
| `405 Method Not Allowed` | SPARQL Update or another unsupported HTTP method |
| `413 Payload Too Large` | Query body exceeds 1 MiB |
| `415 Unsupported Media Type` | Unsupported POST `Content-Type` |
| `500 Internal Server Error` | Query evaluation or serialization failure |

Error bodies are `text/plain`. A `405` response includes `Allow: GET, POST`.
Change RDF by editing the served files; SPARQL Update is rejected permanently.
