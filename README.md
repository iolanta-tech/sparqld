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

JSON-LD-derived sources declare their own `@context`. Relative `.jsonld` and
`.yamlld` context files are resolved inside the served directory, including
relative `@import` references. The canonical
[JSON-LD dollar-convenience context](https://json-ld.org/contexts/dollar-convenience.jsonld)
is available by its URL and is served from the bundled copy; other web context
URLs are rejected. `context.jsonld` and `context.yamlld` have no implicit
effect and work only when a source explicitly references them.

The `sparqld:` named graph is a file catalog. It describes every source graph
as an NFO `FileDataObject`, represents directories as NFO `Folder` resources,
and connects each child to its directory with `nfo:belongsToContainer`.

Each loaded source is associated with the named graph derived from its path.

The default graph is the union of every named graph, including the file
catalog, so ordinary queries operate across the complete directory:

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
for one save. A changed source reloads only that source; changing a local
context reloads every source that declares it, directly or through `@import`.
If parsing fails, `sparqld` removes its graph and adds an `rlog:Entry`
describing the error to the file catalog.

Pass `--no-watch` to load the directory once and disable live updates.

The endpoint is permanently read-only. RDF is changed by editing the source files, not through SPARQL Update.

## Formats

sparqld recognizes JSON-LD (`.jsonld`, `.json`), YAML-LD (`.yamlld`), and
Markdown-LD (`.md`). Markdown-LD reads YAML-LD front matter; the Markdown body
does not contribute RDF. JSON-LD-derived sources use the contexts they declare.

It also recognizes `.n3` as Notation3, `.nq` as N-Quads, `.nt` and `.txt` as
N-Triples, `.rdf` and `.xml` as RDF/XML, `.trig` as TriG, and `.ttl` as
Turtle. See the [File formats reference](https://sparqld.iolanta.tech/reference/formats/)
for the current details.

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
