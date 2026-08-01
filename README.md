---
hide: [toc]
---

# sparqld

**A live, read-only SPARQL server for RDF files.**

`sparqld` watches a directory and exposes its RDF contents through a local SPARQL endpoint.

The directory remains the source of truth. Affected sources are parsed in staging and their changes become visible atomically.

## Usage

Start `sparqld` for a directory:

```console
sparqld ./data
```

The endpoint is available at:

```text
http://127.0.0.1:7737/
```

Query it with [`sq`](https://github.com/ktk/sq):

```console
sq -e http://127.0.0.1:7737/ any
```

Run an arbitrary SPARQL query:

```console
sq -e http://127.0.0.1:7737/ \
  'SELECT ?subject ?predicate ?object {
    ?subject ?predicate ?object
  } LIMIT 20'
```

For repeated use, configure the endpoint in `.sq.toml`:

```toml
default = "sparqld"

[endpoints.sparqld]
url = "http://127.0.0.1:7737/"
```

You can then omit the endpoint URL:

```console
sq any
sq classes
sq graphs
sq -f query.rq
```

## Dataset model

Each source file is exposed through named graphs derived from its path.
Relative IRIs in a source resolve against the internal `sparqld:` IRI of the
directory containing that source.

A directory may define `context.jsonld` or `context.yamlld` for JSON-LD,
YAML-LD, Markdown-LD, and other JSON-LD-inspired formats. The context
is inherited by nested directories until a nearer context file replaces it.
When both context formats exist in one directory, `context.jsonld` takes
precedence.

The `sparqld:` named graph is a file catalog. It describes every source graph
as an NFO `FileDataObject`, represents directories as NFO `Folder` resources,
and connects each child to its directory with `nfo:belongsToContainer`.

Each loaded source is associated with the named graph derived from its path.

The default graph is the union of all named graphs, so ordinary queries operate across the complete directory:

```sparql
SELECT ?person ?name
WHERE {
  ?person <http://xmlns.com/foaf/0.1/name> ?name
}
```

Use `GRAPH` when source boundaries matter:

```sparql
SELECT ?graph ?subject ?predicate ?object
WHERE {
  GRAPH ?graph {
    ?subject ?predicate ?object
  }
}
```

## Live updates

`sparqld` reacts to files being created, modified, moved, or deleted.

Changes are debounced to accommodate editors that emit several filesystem events
for one save. Ordinary file changes reload only that source. Context changes
reload sources below the context's directory because inherited terms may affect
every descendant. If affected sources cannot be parsed, `sparqld` reports the
error and continues serving their last valid graphs.

Pass `--no-watch` to load the directory once and disable live updates.

The endpoint is permanently read-only. RDF is changed by editing the source files, not through SPARQL Update.

## Formats

Supported:

* JSON-LD
* YAML-LD
* Markdown-LD

In development:

* Turtle
* TriG
* N-Triples
* N-Quads
* RDF/XML

## Why?

`sparqld` is intended for RDF that already lives in files:

* ontology and Linked Data repositories;
* local knowledge bases;
* generated RDF build output;
* application and test fixtures;
* documentation and static-site projects;
* coding agents that edit RDF and query the result;
* reproducible projects where external data is downloaded and versioned locally.

It provides the SPARQL equivalent of a simple local file server:

```text
directory of RDF files → live SPARQL endpoint
```

No repository creation, import scripts, or database synchronization step is required.

## Status

Under development.
