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

## :material-currency-usd: Dollar keyword aliases

`sparqld` automatically applies the canonical
[JSON-LD dollar-convenience context](https://json-ld.org/contexts/dollar-convenience.jsonld)
to JSON-LD, YAML-LD, and Markdown-LD. Their document bodies can use aliases such
as `$id`, `$type`, and `$graph` without declaring the context or fetching it over
the network:

{{ example_data('alpha-centauri.yamlld') }}

`@context` is the one JSON-LD keyword that cannot be aliased. Keywords used
inside a context definition cannot be aliased either, so contexts must continue
to use literal `"@context"`, `"@id"`, `"@type"`, and related keywords.

The built-in aliases have the lowest precedence. See
[Context files](context-files.md) for directory inheritance and inline-context
precedence.

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
