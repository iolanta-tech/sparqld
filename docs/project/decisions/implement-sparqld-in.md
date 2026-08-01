---
"@id": implement-sparqld-in.md
"@type": schema:TechArticle
title: Implement sparqld in Rust
status: decided
date: 2026-07-31
author: Anatoly Scherbakov
tags: [decision]
hide: [toc]
---

# Implement sparqld in Rust

{{ adr_metadata(date, status) }}

## :material-text-box-outline: Context

`sparqld` will watch a directory of JSON-LD files and expose their current RDF dataset through a live, read-only SPARQL endpoint. The implementation language must support reliable JSON-LD processing, SPARQL evaluation, filesystem watching, HTTP serving, and straightforward distribution as a local command-line tool; choosing it now will establish the project’s library ecosystem and runtime model before implementation begins.

## :material-arrow-decision-outline: Decision

<table data-adr-comparison markdown="1">
  <tr markdown="span">
    <th>Language</th>
    <th>JSON-LD 1.1</th>
    <th>SPARQL 1.1</th>
    <th>Combined semantic stack</th>
    <th>Decision</th>
  </tr>
  <tr markdown="span">
    <th class="chosen">:simple-rust: Rust</th>
    <td class="chosen" title="Nearly fully conformant JSON-LD 1.1 implementation">:warning: [:fontawesome-brands-github: `oxigraph/oxigraph`](https://github.com/oxigraph/oxigraph)</td>
    <td class="chosen">[:white_check_mark:](https://docs.rs/oxigraph/latest/oxigraph/ "Oxigraph implements SPARQL 1.1 Query")</td>
    <td class="chosen">[0.5.9](https://docs.rs/oxigraph/latest/oxigraph/ "Oxigraph 0.5.9 documents both capabilities")</td>
    <td class="chosen">:white_check_mark: Chosen</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:simple-python: Python</th>
    <td class="not-selected">[:white_check_mark:](https://rdflib.readthedocs.io/en/stable/apidocs/rdflib.plugins.parsers.jsonld/ "Parser defaults to JSON-LD 1.1")</td>
    <td class="not-selected">[:white_check_mark:](https://rdflib.readthedocs.io/en/latest/apidocs/rdflib.plugins.sparql/ "RDFLib supports SPARQL 1.1 queries")</td>
    <td class="not-selected">[7.6.0](https://pypi.org/project/rdflib/ "RDFLib 7.6.0 was released 2026-02-13")</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:simple-typescript: TypeScript</th>
    <td class="not-selected">[:white_check_mark:](https://www.npmjs.com/package/jsonld "jsonld.js implements JSON-LD 1.1")</td>
    <td class="not-selected">[:white_check_mark:](https://comunica.dev/blog/2020-08-24-release_1_16/ "Comunica passes the SPARQL 1.1 query test suite")</td>
    <td class="not-selected">[jsonld 9.0.0](https://www.npmjs.com/package/jsonld) + [Comunica 5.3.0](https://www.npmjs.com/package/%40comunica/query-sparql-file)</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:material-language-csharp: C#</th>
    <td class="not-selected">[:white_check_mark:](https://dotnetrdf.org/docs/latest/user_guide/jsonld/api.html "dotNetRDF implements the JSON-LD 1.1 APIs")</td>
    <td class="not-selected">[:white_check_mark:](https://dotnetrdf.org/docs/stable/developer_guide/sparql/sparql_engine.html "Leviathan supports full SPARQL 1.1 queries")</td>
    <td class="not-selected">[3.5.1](https://www.nuget.org/packages/dotNetRDF/ "dotNetRDF 3.5.1 was released in February 2026")</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:fontawesome-brands-java: Java<br><small>JVM</small></th>
    <td class="not-selected">[:white_check_mark:](https://jena.apache.org/documentation/io/json-ld-11.html "Jena defaults to JSON-LD 1.1")</td>
    <td class="not-selected">[:white_check_mark:](https://jena.apache.org/documentation/query/ "ARQ evaluates standard SPARQL and its 1.1 features")</td>
    <td class="not-selected">[6.1.0](https://jena.apache.org/download/ "Current official Jena release")</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:simple-kotlin: Kotlin<br><small>JVM</small></th>
    <td class="not-selected">[:material-swap-horizontal:](https://kotlinlang.org/docs/java-interop.html "Java interop gives access to Jena JSON-LD 1.1") [Jena](https://jena.apache.org/documentation/io/json-ld-11.html "JSON-LD 1.1")</td>
    <td class="not-selected">[:material-swap-horizontal:](https://kotlinlang.org/docs/java-interop.html "Java interop gives access to Jena ARQ") [ARQ](https://jena.apache.org/documentation/query/ "SPARQL query engine")</td>
    <td class="not-selected">[:material-swap-horizontal:](https://kotlinlang.org/docs/java-interop.html "Kotlin calls Java directly") [Jena 6.1.0](https://jena.apache.org/download/ "The semantic stack remains Jena 6.1.0")</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="not-selected">:simple-clojure: Clojure<br><small>JVM</small></th>
    <td class="not-selected">[:material-swap-horizontal:](https://clojure.org/reference/java_interop "Java interop gives access to Jena JSON-LD 1.1") [Jena](https://jena.apache.org/documentation/io/json-ld-11.html "JSON-LD 1.1")</td>
    <td class="not-selected">[:material-swap-horizontal:](https://clojure.org/reference/java_interop "Java interop gives access to Jena ARQ") [ARQ](https://jena.apache.org/documentation/query/ "SPARQL query engine")</td>
    <td class="not-selected">[:material-swap-horizontal:](https://clojure.org/reference/java_interop "Clojure calls Java directly") [Jena 6.1.0](https://jena.apache.org/download/ "The semantic stack remains Jena 6.1.0")</td>
    <td class="not-selected">:material-minus-circle-outline: Not selected</td>
  </tr>
  <tr markdown="span">
    <th class="excl">:simple-go: Go</th>
    <td class="excl">[:white_check_mark:](https://pkg.go.dev/github.com/tggo/goRDFlib/jsonld "goRDFlib delegates to JSON-goLD 1.1") [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold)</td>
    <td class="excl">[:white_check_mark:](https://pkg.go.dev/github.com/tggo/goRDFlib@v0.1.13 "goRDFlib provides a SPARQL 1.1 query engine")</td>
    <td class="excl hot">[v0.1.13](https://pkg.go.dev/github.com/tggo/goRDFlib@v0.1.13 "Published 2026-07-02; no known importers")</td>
    <td class="excl">:x: Excluded<br><small>[Insufficient adoption](https://pkg.go.dev/github.com/tggo/goRDFlib@v0.1.13 "v0.1.13, published 2026-07-02, has no known importers")</small></td>
  </tr>
  <tr markdown="span">
    <th class="excl">:simple-cplusplus: C++</th>
    <td class="excl hot" title="In progress; expansion and toRdf only">:warning: [:fontawesome-brands-github: `dcdpr/jsonld-cpp`](https://github.com/dcdpr/jsonld-cpp)</td>
    <td class="excl hot">[:warning:](https://librdf.org/rasqal/ "Rasqal omits multiple SPARQL 1.1 features, including property paths and MINUS")</td>
    <td class="excl hot">:warning: no release [:fontawesome-brands-github: `dcdpr/jsonld-cpp`](https://github.com/dcdpr/jsonld-cpp)<br>[Rasqal 0.9.33](https://librdf.org/rasqal/ "Released 2014-12-15")</td>
    <td class="excl">:x: Excluded<br><small>Incomplete JSON-LD [:fontawesome-brands-github: `dcdpr/jsonld-cpp`](https://github.com/dcdpr/jsonld-cpp) + [partial SPARQL](https://librdf.org/rasqal/ "Rasqal 0.9.33 omits SPARQL 1.1 features and was released 2014-12-15")</small></td>
  </tr>
  <tr markdown="span">
    <th class="excl">:simple-haskell: Haskell</th>
    <td class="excl hot">[:x:](https://hackage.haskell.org/package/rdf4h "rdf4h lists N-Triples, Turtle, and RDF/XML, but not JSON-LD")</td>
    <td class="excl hot">[:x:](https://hackage.haskell.org/package/rdf4h "rdf4h exposes triple and predicate matching, not SPARQL evaluation")</td>
    <td class="excl hot">[rdf4h 5.2.2](https://hackage.haskell.org/package/rdf4h "Uploaded 2026-02-05")<br><small>2018 [:fontawesome-brands-github: `agentultra/json-ld`](https://github.com/agentultra/json-ld) · archived [:fontawesome-brands-github: `cordawyn/aeson-ld`](https://github.com/cordawyn/aeson-ld)</small></td>
    <td class="excl">:x: Excluded<br><small>[Maintained stack lacks JSON-LD and SPARQL](https://hackage.haskell.org/package/rdf4h "Maintained rdf4h provides neither capability")</small></td>
  </tr>
  <tr markdown="span">
    <th class="excl">:simple-zig: Zig</th>
    <td class="excl hot">[:x:](https://github.com/search?q=jsonld+language%3AZig&type=repositories "No Zig repository appears in GitHub's JSON-LD language search")</td>
    <td class="excl hot" title="Custom pattern matching, joins, and filters; no SPARQL 1.1 conformance claim">:warning: [:fontawesome-brands-github: `gHashTag/zig-knowledge-graph`](https://github.com/gHashTag/zig-knowledge-graph)</td>
    <td class="excl hot">Unreleased [:fontawesome-brands-github: `gHashTag/zig-knowledge-graph`](https://github.com/gHashTag/zig-knowledge-graph)</td>
    <td class="excl">:x: Excluded<br><small>[No JSON-LD implementation identified](https://github.com/search?q=jsonld+language%3AZig&type=repositories) + unreleased SPARQL [:fontawesome-brands-github: `gHashTag/zig-knowledge-graph`](https://github.com/gHashTag/zig-knowledge-graph)</small></td>
  </tr>
</table>

Capability rows: :white_check_mark: direct support · :material-swap-horizontal: host-language interoperability · :warning: partial or unverified · :x: unavailable. Decision row: :white_check_mark: chosen · :material-minus-circle-outline: not selected · :x: excluded.

## :material-arrow-right-bold-outline: Consequences

- Building locally and in release automation requires a Rust toolchain.
- [Distributed executables run without Rust installed](https://doc.rust-lang.org/book/ch01-02-hello-world.html).
