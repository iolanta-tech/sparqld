---
"@context": context.yamlld
"@id": project/decisions/implement-live-file-backed-sparql-access-with.md
"@type": schema:TechArticle
title: Turn a bunch of files into a queryable graph with sparqld
status: living
date: 2026-11-10
author: Anatoly Scherbakov
tags: [decision]
hide: [toc]
---

# Turn a bunch of files into a queryable graph with `sparqld`

{{ adr_metadata(date, status) }}

## :material-text-box-outline: Context

People and agents need a live queryable graph whose source of truth stays
ordinary files, without operating a separately loaded store. Linked Data formats
and SPARQL are why `sparqld` wins among alternatives that cover the same use
case; they are not the definition of the use case.

```mermaid
flowchart LR
    AuthoritativeFiles["Authoritative files"] --> QueryableGraph["Queryable graph"]
    QueryableGraph --> HumanUsers["Human users"]
    QueryableGraph --> Agents["Agents"]
```

## :material-arrow-decision-outline: Decision

<table data-adr-comparison markdown="1">
  <tr markdown="span">
    <th>Alternative</th>
    <th>Direct file access</th>
    <th>Live reload</th>
    <th>Query access</th>
    <th>[YAML-LD](https://www.w3.org/TR/yaml-ld-10/)</th>
    <th>[Markdown-LD](../../reference/formats.md)</th>
    <th>[GitHub stars](https://docs.github.com/en/get-started/exploring-projects-on-github/saving-repositories-with-stars)</th>
    <th>Latest release</th>
  </tr>
  <tr markdown="span">
    <th class="chosen">[:fontawesome-brands-github: `iolanta-tech/sparqld`](https://github.com/iolanta-tech/sparqld)</th>
    <td class="chosen">[:white_check_mark:](../../index.md "Reads served files directly")</td>
    <td class="chosen">[:white_check_mark:](../../reference/agents/index.md "Watches for file changes")</td>
    <td class="chosen">[SPARQL Protocol](../../reference/api/index.md "SPARQL 1.1 Protocol HTTP endpoint")</td>
    <td class="chosen">[:white_check_mark:](../../reference/formats.md "Supports YAML-LD")</td>
    <td class="chosen">[:white_check_mark:](../../reference/formats.md "Supports Markdown-LD")</td>
    <td class="chosen">[0](https://github.com/iolanta-tech/sparqld/stargazers "Observed 2026-08-07")</td>
    <td class="chosen">[0.1.1<br>2026-08-07](https://crates.io/crates/sparqld/0.1.1 "Released 2026-08-07")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[SPARQL Anything](https://sparql-anything.cc/)</th>
    <td class="excl">[:white_check_mark:](https://sparql-anything.readthedocs.io/stable/ "Queries local files directly")</td>
    <td class="excl">[:warning:](https://sparql-anything.readthedocs.io/stable/ "Processes local files per query; no documented watcher")</td>
    <td class="excl">[SPARQL Protocol](https://sparql-anything.readthedocs.io/stable/ "SPARQL over local files via an HTTP endpoint")</td>
    <td class="excl hot">[:x:](https://sparql-anything.readthedocs.io/stable/formats/YAML/ "YAML support is not YAML-LD")</td>
    <td class="excl hot">[:x:](https://sparql-anything.readthedocs.io/stable/formats/Markdown/ "Markdown support is not Markdown-LD")</td>
    <td class="excl">[302](https://github.com/SPARQL-Anything/sparql.anything/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[v1.2.0<br>2026-07-24](https://github.com/SPARQL-Anything/sparql.anything/releases/tag/v1.2.0 "Released 2026-07-24")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[Apache Jena Fuseki](https://jena.apache.org/documentation/fuseki2/)</th>
    <td class="excl hot">[:x:](https://jena.apache.org/documentation/fuseki2/fuseki-server.html "Loads a file at startup")</td>
    <td class="excl">[:x:](https://jena.apache.org/documentation/fuseki2/fuseki-server.html "No live reload documented")</td>
    <td class="excl">[SPARQL Protocol](https://jena.apache.org/documentation/fuseki2/fuseki-server.html "Provides a SPARQL HTTP endpoint")</td>
    <td class="excl">[:x:](https://jena.apache.org/documentation/fuseki2/fuseki-server.html "YAML-LD input is not documented")</td>
    <td class="excl">[:x:](https://jena.apache.org/documentation/fuseki2/fuseki-server.html "Markdown-LD input is not documented")</td>
    <td class="excl">[1,398](https://github.com/apache/jena/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[6.1.0<br>2026-05-03](https://central.sonatype.com/artifact/org.apache.jena/jena-fuseki-main/6.1.0 "Released 2026-05-03")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[Oxigraph server](https://docs.rs/oxigraph_server/latest/oxigraph_server/)</th>
    <td class="excl hot">[:x:](https://docs.rs/crate/oxigraph_server/0.3.4 "Bulk-loads data into storage")</td>
    <td class="excl">[:x:](https://docs.rs/crate/oxigraph_server/0.3.4 "No live reload documented")</td>
    <td class="excl">[SPARQL Protocol](https://docs.rs/crate/oxigraph_server/0.3.4 "Provides a SPARQL HTTP endpoint")</td>
    <td class="excl">[:x:](https://docs.rs/oxigraph/latest/oxigraph/io/ "YAML-LD is not an RDF format")</td>
    <td class="excl">[:x:](https://docs.rs/oxigraph/latest/oxigraph/io/ "Markdown-LD is not an RDF format")</td>
    <td class="excl">[1,801](https://github.com/oxigraph/oxigraph/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[v0.5.9<br>2026-06-18](https://github.com/oxigraph/oxigraph/releases/tag/v0.5.9 "Released 2026-06-18")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[RDF4J Server](https://rdf4j.org/documentation/tools/server-workbench/)</th>
    <td class="excl hot">[:x:](https://rdf4j.org/documentation/programming/repository/ "Loads local RDF files into a repository")</td>
    <td class="excl">[:x:](https://rdf4j.org/documentation/tools/server-workbench/ "No live reload documented")</td>
    <td class="excl">[SPARQL Protocol](https://rdf4j.org/documentation/tools/server-workbench/ "Provides a SPARQL HTTP endpoint")</td>
    <td class="excl">[:x:](https://rdf4j.org/documentation/tools/server-workbench/ "YAML-LD input is not documented")</td>
    <td class="excl">[:x:](https://rdf4j.org/documentation/tools/server-workbench/ "Markdown-LD input is not documented")</td>
    <td class="excl">[409](https://github.com/eclipse-rdf4j/rdf4j/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[5.3.2<br>2026-06-29](https://github.com/eclipse-rdf4j/rdf4j/releases/tag/5.3.2 "Released 2026-06-29")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[Comunica over local RDF files](https://comunica.dev/)</th>
    <td class="excl">[:white_check_mark:](https://comunica.dev/docs/query/getting_started/query_cli_file/ "Queries local RDF files directly")</td>
    <td class="excl">[:warning:](https://comunica.dev/docs/query/getting_started/query_cli_file/ "Reads local files again on each query; no watcher")</td>
    <td class="excl">[SPARQL Protocol](https://comunica.dev/docs/query/getting_started/setup_endpoint/ "SPARQL over local RDF files via an HTTP endpoint")</td>
    <td class="excl hot">[:x:](https://comunica.dev/docs/query/getting_started/query_cli_file/ "YAML-LD input is not documented")</td>
    <td class="excl hot">[:x:](https://comunica.dev/docs/query/getting_started/query_cli_file/ "Markdown-LD input is not documented")</td>
    <td class="excl">[577](https://github.com/comunica/comunica/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[v5.3.0<br>2026-07-10](https://github.com/comunica/comunica/releases/tag/v5.3.0 "Released 2026-07-10")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[Apache Jena ARQ](https://jena.apache.org/documentation/query/cmds.html)</th>
    <td class="excl">[:white_check_mark:](https://jena.apache.org/documentation/query/cmds.html "Queries RDF files directly")</td>
    <td class="excl">[:warning:](https://jena.apache.org/documentation/query/cmds.html "Rerunning reads a changed file; no watcher")</td>
    <td class="excl hot">[SPARQL CLI](https://jena.apache.org/documentation/query/cmds.html "SPARQL on the command line; no HTTP endpoint")</td>
    <td class="excl">[:x:](https://jena.apache.org/documentation/query/cmds.html "YAML-LD input is not documented")</td>
    <td class="excl">[:x:](https://jena.apache.org/documentation/query/cmds.html "Markdown-LD input is not documented")</td>
    <td class="excl">[1,398](https://github.com/apache/jena/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[6.1.0<br>2026-05-03](https://central.sonatype.com/artifact/org.apache.jena/jena-arq/6.1.0 "Released 2026-05-03")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[:fontawesome-brands-github: `fabio-rovai/open-ontologies`](https://github.com/fabio-rovai/open-ontologies)</th>
    <td class="excl hot">[:x:](https://github.com/fabio-rovai/open-ontologies "Uses an in-memory Oxigraph triple store")</td>
    <td class="excl">[:x:](https://github.com/fabio-rovai/open-ontologies "No live reload documented")</td>
    <td class="excl hot">[SPARQL over JSON-RPC](https://github.com/fabio-rovai/open-ontologies "Serve is JSON-RPC over standard input and output")</td>
    <td class="excl">[:x:](https://github.com/fabio-rovai/open-ontologies "YAML-LD input is not documented")</td>
    <td class="excl">[:x:](https://github.com/fabio-rovai/open-ontologies "Markdown-LD input is not documented")</td>
    <td class="excl">[353](https://github.com/fabio-rovai/open-ontologies/stargazers "Observed 2026-08-07")</td>
    <td class="excl">[v1.1.1<br>2026-08-05](https://github.com/fabio-rovai/open-ontologies/releases/tag/v1.1.1 "Released 2026-08-05")</td>
  </tr>
  <tr markdown="span">
    <th class="excl">[:fontawesome-brands-github: `Volland/typedgraph`](https://github.com/Volland/typedgraph)</th>
    <td class="excl">[:white_check_mark:](https://typedgraph.org/ "Markdown files remain the source of truth")</td>
    <td class="excl">[:white_check_mark:](https://typedgraph.org/ "graph-query blocks update live as the vault changes")</td>
    <td class="excl hot">[openCypher](https://typedgraph.org/ "Read-only openCypher and MCP tools; not SPARQL Protocol")</td>
    <td class="excl hot">[:x:](https://typedgraph.org/ "Schema uses YAML frontmatter, not YAML-LD")</td>
    <td class="excl hot">[:x:](https://typedgraph.org/ "Typed note edges are not Markdown-LD")</td>
    <td class="excl">[15](https://github.com/Volland/typedgraph/stargazers "Observed 2026-11-10")</td>
    <td class="excl">[0.25.0<br>2026-11-09](https://github.com/Volland/typedgraph/releases/tag/0.25.0 "Released 2026-11-09")</td>
  </tr>
</table>

Green row: chosen. Red row: excluded; stronger-red cells are the disqualifying
capability. :warning: re-querying reflects a changed file but there is no
watcher. This ADR is living: when a new alternative appears, re-open the
comparison and bump `date`.

## :material-arrow-right-bold-outline: Consequences

- Work proceeds on `sparqld` as the selected product for turning authoritative
  files into a live queryable graph.
- Revisit this comparison when new alternatives appear.
- Obsidian query UX is tracked on the [Roadmap](../roadmap/index.md), not settled
  here.
