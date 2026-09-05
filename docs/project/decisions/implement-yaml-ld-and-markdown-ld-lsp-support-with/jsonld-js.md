---
title: "Build on jsonld.js"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on jsonld.js

{{ universe_metadata('index.md', status) }}

This planning path uses jsonld.js for JSON-LD semantic processing and builds an LSP around it. jsonld.js supplies JSON-LD processing, not an LSP transport, workspace model, or editor integration; “universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `digitalbazaar/jsonld.js`](https://github.com/digitalbazaar/jsonld.js) publishes the JavaScript [`jsonld`](https://www.npmjs.com/package/jsonld) package for JSON-LD processing.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Choose the TypeScript or JavaScript LSP runtime, transport, document model, position conversion, lifecycle handling, and capability-registration boundaries.
- [ ] :material-file-code-outline: Build JSON-LD, YAML-LD, and Markdown front-matter syntax layers that preserve a source map from semantic value paths to authored key and scalar ranges.
- [ ] :material-code-json: Adapt mapped semantic values for jsonld.js processing [:fontawesome-brands-github: `digitalbazaar/jsonld.js`](https://github.com/digitalbazaar/jsonld.js) while retaining path associations for diagnostics and semantic results.
- [ ] :material-alert-circle-outline: Translate jsonld.js processing failures into diagnostics at original JSON, YAML-LD, or Markdown-LD source spans rather than generated JSON text.
- [ ] :material-book-open-variant: Define the context, document-loader, and local-resolver policy, including contextual base IRIs, relative document resolution, permitted local paths, and remote-context handling.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued scalars, publish clickable links, and return every workspace definition for an IRI.
- [ ] :material-content-save-outline: Keep unsaved buffers and incremental edits authoritative for parsing, context loading, diagnostics, document links, and the workspace definition index.
- [ ] :material-file-search-outline: Maintain a workspace index of definitions and references that updates as in-memory and on-disk documents change.
- [ ] :material-test-tube: Add protocol and end-to-end tests for diagnostic ranges, context errors, `@type: @id` links, relative cross-file resolution, multiple definition targets, and unsaved edits.
- [ ] :material-package-variant-closed: Publish a Node package and editor integrations, including file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project owns the LSP transport, incremental document model, syntax parsing, source mapping, navigation, workspace indexing, and editor-distribution layers around jsonld.js.
- [ ] :material-language-javascript: The implementation and release toolchain follow the Node and TypeScript ecosystem, including its editor-client packaging conventions.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD need a maintained semantic-value-to-source-span bridge because jsonld.js processes JSON-LD values rather than authored YAML or Markdown syntax.
- [ ] :material-source-branch: jsonld.js dependency and behavioural updates need compatibility tracking behind a stable LSP-facing semantic model.
