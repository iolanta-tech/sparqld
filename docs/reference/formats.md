---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/formats.md
"@type": schema:TechArticle
hide: [toc]
name: Supported file formats
title: File formats
description: File formats recognized by sparqld and their implementation status.
---

# :material-file-multiple-outline: File formats

## :material-check-circle-outline: Supported

<div class="grid cards" markdown>

-   :material-code-json:{ .lg .middle } **JSON-LD**

    ---

    <span class="format-status format-status--supported">Supported</span> · `.jsonld`

    Linked data in JSON.

-   :simple-yaml:{ .lg .middle } **YAML-LD**

    ---

    <span class="format-status format-status--supported">Supported</span> · `.yamlld`

    JSON-LD authored as YAML 1.2.

-   :material-language-markdown:{ .lg .middle } **Markdown-LD**

    ---

    <span class="format-status format-status--supported">Supported</span> · `.md`

    Linked data in YAML-LD front matter.

</div>

## :material-progress-wrench: In development

<div class="grid cards" markdown>

-   :material-turtle:{ .lg .middle } **Turtle**

    ---

    <span class="format-status format-status--development">In development</span> · `.ttl`

    Compact RDF graph syntax.

-   :material-graph-outline:{ .lg .middle } **TriG**

    ---

    <span class="format-status format-status--development">In development</span> · `.trig`

    Turtle syntax for RDF datasets.

-   :material-format-list-bulleted:{ .lg .middle } **N-Triples**

    ---

    <span class="format-status format-status--development">In development</span> · `.nt`

    Line-oriented RDF triples.

-   :material-view-grid-outline:{ .lg .middle } **N-Quads**

    ---

    <span class="format-status format-status--development">In development</span> · `.nq`

    Line-oriented RDF quads.

-   :material-xml:{ .lg .middle } **RDF/XML**

    ---

    <span class="format-status format-status--development">In development</span> · `.rdf`, `.xml`

    XML serialization of RDF.

</div>

In-development formats are not yet part of the supported interface.
