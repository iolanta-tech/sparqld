# sparqld

[![CI](https://github.com/iolanta-tech/sparqld/actions/workflows/ci.yml/badge.svg)](https://github.com/iolanta-tech/sparqld/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/sparqld.svg)](https://crates.io/crates/sparqld)
[![License](https://img.shields.io/crates/l/sparqld.svg)](#license)

**A live, read-only SPARQL endpoint for linked data stored in files.**

`sparqld` watches a directory and makes its RDF files queryable immediately.
The files remain authoritative: there is no database to operate, import step to
run, or duplicate copy to synchronize. Each successful edit becomes available
atomically.

## Install

Install [Rust and Cargo](https://rustup.rs/) if they are not already available,
then install `sparqld` from crates.io:

```console
cargo install sparqld
```

## Start an endpoint

Serve a directory of RDF files:

```console
sparqld ./data
```

The endpoint listens locally at `http://127.0.0.1:7737/` and watches the
directory for changes.

Query it with any SPARQL client. For example, install
[`sq`](https://github.com/ktk/sq) and inspect the first twenty triples:

```console
cargo install --git https://github.com/ktk/sq
sq -e http://127.0.0.1:7737/ \
  'SELECT ?subject ?predicate ?object { ?subject ?predicate ?object } LIMIT 20'
```

For copy-pasteable example data and a first query, see the
[Quickstart](https://sparqld.iolanta.tech/quickstart/).

## What it supports

- [JSON-LD](https://www.w3.org/TR/json-ld11/),
  [YAML-LD](https://www.w3.org/TR/yaml-ld-10/), and Markdown-LD;
- Turtle, TriG, N-Triples, N-Quads, RDF/XML, and Notation3;
- local JSON-LD contexts, including relative `@import` references, scoped to
  the served directory;
- a [live file catalog](https://sparqld.iolanta.tech/reference/file-catalog/)
  and [named graph model](https://sparqld.iolanta.tech/reference/named-graphs/)
  for locating source data.

See the [file formats reference](https://sparqld.iolanta.tech/reference/formats/)
for extensions and format-specific behavior.

## How it behaves

Every source file is represented by its own named graph. A file that declares
named graphs, such as a TriG file or nanopublication, retains them as graphs
scoped to that source. The default graph is the union of source graphs and the
catalog, so ordinary SPARQL queries search the whole directory.

When a file or its local context changes, `sparqld` parses the affected data
before replacing it. A failed parse removes that source graph and records the
error in the catalog. Pass `--no-watch` to load once without watching.

The endpoint is permanently read-only: modify RDF by editing files, never via
SPARQL Update.

## Security

By default, `sparqld` listens only on `127.0.0.1`. Using `--host 0.0.0.0`
makes its read-only endpoint reachable on the network; it provides neither
authentication nor TLS. Put network-facing deployments behind appropriate
access controls.

## Project

`sparqld` is under active development. See the
[documentation](https://sparqld.iolanta.tech/),
[roadmap](https://sparqld.iolanta.tech/project/roadmap/), and
[issue tracker](https://github.com/iolanta-tech/sparqld/issues).

## License

Licensed under either of
[Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
