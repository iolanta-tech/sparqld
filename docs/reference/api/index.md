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

| Method | Query encoding | Content type |
| --- | --- | --- |
| `GET` | URL-encoded `query` parameter | |
| `POST` | SPARQL query in the request body | `application/sparql-query` |
| `POST` | URL-encoded `query` parameter in the request body | `application/x-www-form-urlencoded` |

A plain `GET /` returns a text landing response.

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
