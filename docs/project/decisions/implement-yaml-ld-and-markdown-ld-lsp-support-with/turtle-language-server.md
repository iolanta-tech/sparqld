---
title: "Build on Turtle Language Server"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on Turtle Language Server

{{ universe_metadata('index.md', status) }}

This planning path builds the server on Turtle Language Server. “Universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver) is a Python language server for Turtle, published as [`turtle-language-server`](https://pypi.org/project/turtle-language-server/).

## :material-format-list-checks: Implementation checklist

- [ ] Evaluate the reusable `pygls` lifecycle and document synchronization [:fontawesome-brands-github: `server.py`](https://github.com/BrickSchema/langserver/blob/main/src/turtle_language_server/server.py), separating protocol plumbing from Turtle-specific behaviour.
- [ ] Generalize Turtle and RDFLib-backed language behaviour [:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver/tree/main/src/turtle_language_server) into JSON-LD-aware analysis without regressing Turtle documents.
- [ ] Implement a [YAML AST](https://yaml.readthedocs.io/en/latest/) adapter that retains authored key and scalar spans while producing JSON-LD values.
- [ ] Extract [Markdown front matter](https://spec.commonmark.org/current/#front-matter) and map every embedded YAML range back through its containing Markdown offset.
- [ ] Select a [JSON-LD 1.1 processor and context model](https://www.w3.org/TR/json-ld11-api/) and implement a safe [relative-IRI resolver](https://www.rfc-editor.org/rfc/rfc3986#section-5) shared by all three document formats.
- [ ] Interpret [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI values, expose clickable [document links](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentLink), and return every target through [Go to Definition](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_definition).
- [ ] Make the derived graph, ranges, links, and definition index follow the authoritative in-memory [open and change notifications](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didChange) before workspace files on disk.
- [ ] Add pytest-based fixtures [:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver/tree/main/test) and end-to-end LSP tests for diagnostic ranges, Markdown offsets, relative IRIs, multiple definitions, links, and unsaved edits.
- [ ] Publish a Python package and validate editor deployment through the [`turtle-language-server` installation channel](https://pypi.org/project/turtle-language-server/).

## :material-call-split: Universe consequences

- [ ] Maintain a Python and `pygls` runtime, including its LSP lifecycle and editor-distribution compatibility.
- [ ] Own a new JSON-LD semantic layer and its context-loading, IRI-resolution, and source-map policies beside the original Turtle/RDFLib implementation.
- [ ] Maintain YAML-LD and Markdown-LD parsers, mappings, and incremental-edit tests that the Turtle server does not provide.
- [ ] Separate generic server packaging and documentation from the Brick-specific identity of the existing published package.
