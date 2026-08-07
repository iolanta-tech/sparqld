---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/named-graphs.md
"@type": schema:TechArticle
hide: [toc]
name: Named graphs
title: Named graphs
description: How sparqld assigns source graphs and describes the served directory.
---

# :material-source-branch: Named graphs

`sparqld` preserves the boundary between files by loading every source into its
own named graph.

<div class="grid cards" markdown>

-   :material-file-document-outline:{ .lg .middle } **Source graph**

    ---

    Its IRI is `sparqld:` followed by the source path relative to the served
    directory.

-   :material-set-merge:{ .lg .middle } **Default graph**

    ---

    Queries without `GRAPH` see the union of every named graph, including the
    file catalog.

-   :material-folder-information-outline:{ .lg .middle } **File catalog**

    ---

    The reserved `sparqld:` graph describes sources, directories, and their
    containment.

</div>

## :material-link-variant: Graph IRIs

For the tangible [`examples/`](https://github.com/iolanta-tech/sparqld/tree/main/docs/examples)
directory:

| Source path | Named graph |
| --- | --- |
| `alpha-centauri.yamlld` | `sparqld:alpha-centauri.yamlld` |
| `centaurus.md` | `sparqld:centaurus.md` |
| `proxima-centauri-b.jsonld` | `sparqld:proxima-centauri-b.jsonld` |

Paths always use `/` as the separator, and characters that cannot appear
literally in the graph IRI are percent-encoded. Renaming or moving a source
therefore gives it a new graph IRI.

Use `GRAPH` to retain source provenance in query results:

{{ query_data('all-quads.rq') }}

## :material-folder-multiple: File catalog

The `sparqld:` graph models the served directory with the
[Nepomuk File Ontology](https://www.semanticdesktop.org/ontologies/2007/03/22/nfo/).

| Resource | RDF types | Properties |
| --- | --- | --- |
| Served directory | `nfo:FileDataObject`, `nfo:Folder` | `nfo:fileName` |
| Nested directory | `nfo:FileDataObject`, `nfo:Folder` | `nfo:fileName`, `nfo:belongsToContainer` |
| Source file | `nfo:FileDataObject` | `nfo:fileName`, `nfo:belongsToContainer` |

The source resource IRI is also its named graph IRI. The catalog is updated
when source files are created, moved, or deleted.

{{ query_data('file-catalog.rq') }}
