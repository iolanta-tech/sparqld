---
name: sparqld
description: Query a local sparqld knowledge base and report evidence-backed answers with the SPARQL used. Use when a user asks to find, inspect, or answer questions from data served by sparqld, especially when the endpoint, RDF vocabulary, or relationship path must be discovered.
---

# Sparqld

Query the local read-only endpoint, beginning with its explicit URL when given;
otherwise use the default `http://127.0.0.1:7737/`.

## Start the endpoint when needed

Test the endpoint with `sq graphs`. Act on a connection failure only; do not
mistake a SPARQL, authentication, or serialization error for a stopped server.
When sparqld is not running:

1. Determine the directory to serve from the user's request or available
   project context. Ask for it when it cannot be established safely; do not
   assume that an arbitrary working directory is a knowledge base.
2. Confirm that `sparqld --version` works. When it is missing and Cargo is
   available, install it with `cargo install sparqld`.
3. Start `sparqld <directory>` in the execution environment's background
   process facility, retaining its process handle and logs. Do not block the
   agent's query workflow on the foreground watcher.
4. Retry `sq graphs` for a short, bounded period. If it still cannot connect,
   report the startup error and the log location rather than guessing at query
   results.

## Query workflow

1. Prefer the [`sq`](https://github.com/ktk/sq) client. Confirm it is available
   with `sq --version`; when missing, install it with
   `cargo install --git https://github.com/ktk/sq`. If Cargo is unavailable or
   installation is not appropriate, use a SPARQL Protocol HTTP request instead.
2. Confirm that the endpoint responds. Use
   `sq -e http://127.0.0.1:7737/ graphs` when no project `.sq.toml` selects it.
   With a configured `.sq.toml`, use `sq graphs`.
3. Inspect the RDF before making semantic assumptions. Discover relevant names,
   types, predicates, and link directions with small `SELECT DISTINCT` queries.
4. Follow the vocabulary and relationship structure that the data actually
   models. Use property paths when the answer spans containment or multi-hop
   relationships.
5. Run the final focused query. Prefer `DISTINCT` for a filesystem-backed
   dataset, where more than one source graph can state the same fact.
6. State only what the result establishes. Call out missing or indirect links
   instead of treating them as a definitive real-world absence.

## Run a query

Pipe one-off SPARQL to `sq`:

```sh
printf '%s\n' 'SELECT DISTINCT ?subject WHERE { ?subject ?predicate ?object } LIMIT 20' \
  | sq -e http://127.0.0.1:7737/
```

Pass a query inline or use `-f query.rq` when a file already exists. Omit `-e`
when the repository's `.sq.toml` selects the endpoint. Use `curl` with
`--data-urlencode` only as a fallback. Do not send SPARQL Update: sparqld
intentionally exposes a read-only endpoint.

## Response

Give a concise answer, distinguish graph facts from any inference, and include
the exact final SPARQL query in a `sparql` code block. Mention the endpoint only
when it helps the user reproduce the result.
