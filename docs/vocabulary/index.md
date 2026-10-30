---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": vocabulary/index.md
"@type": schema:TechArticle
icon: material/book-alphabet
hide:
  - toc
title: Vocabulary
description: RDF terms used for context registration and file catalog metadata.
---

# :material-book-alphabet: Vocabulary

These RDF terms describe context registrations and
[file catalog](../reference/file-catalog.md) metadata.

<div class="grid cards" markdown>

-   :material-link-variant:{ .lg .middle } **[`jsonld:context`](http://www.w3.org/ns/json-ld#context)**

    ---

    Link a source file to the context URIs requested while parsing it.

-   :material-file-link:{ .lg .middle } **[`sparqld:context-file`](sparqld:context-file)**

    ---

    Link an absolute context URI to its local body file in project RDF.

</div>
