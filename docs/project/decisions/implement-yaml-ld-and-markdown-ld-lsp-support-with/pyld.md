---
title: "Build on PyLD"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on PyLD

{{ universe_metadata('index.md', status) }}

This planning path uses PyLD as the JSON-LD semantic foundation and builds an LSP around it. PyLD supplies JSON-LD processing, not an LSP transport, workspace model, or editor integration; “universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `digitalbazaar/pyld`](https://github.com/digitalbazaar/pyld) publishes the Python [`PyLD`](https://pypi.org/project/PyLD/) package for JSON-LD processing.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Choose the Python LSP runtime, transport, document model, position conversion, lifecycle handling, and capability-registration boundaries.
- [ ] :material-file-code-outline: Build JSON-LD, YAML-LD, and Markdown front-matter syntax layers that preserve a source map from semantic value paths to authored key and scalar ranges.
- [ ] :material-code-json: Adapt mapped semantic values for PyLD processing [:fontawesome-brands-github: `digitalbazaar/pyld`](https://github.com/digitalbazaar/pyld) while retaining path associations for diagnostics and semantic results.
- [ ] :material-alert-circle-outline: Translate PyLD processing failures into diagnostics at original JSON, YAML-LD, or Markdown-LD source spans rather than generated JSON text.
- [ ] :material-book-open-variant: Define the context, document-loader, and local-IRI resolver policy, including contextual base IRIs, relative document resolution, permitted local paths, and remote-context handling.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued scalars, publish clickable links, and return every workspace definition for an IRI.
- [ ] :material-content-save-outline: Keep unsaved buffers and incremental edits authoritative for parsing, context loading, diagnostics, document links, and the workspace definition index.
- [ ] :material-file-search-outline: Maintain a workspace index of definitions and references that updates as in-memory and on-disk documents change.
- [ ] :material-test-tube: Add PyLD, LSP-protocol, and end-to-end tests for diagnostic ranges, context errors, `@type: @id` links, relative cross-file resolution, multiple definition targets, and unsaved edits.
- [ ] :material-package-variant-closed: Publish a Python package and editor integrations, including file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project owns the LSP transport, incremental document model, syntax parsing, source mapping, navigation, workspace indexing, and editor-distribution layers around PyLD.
- [ ] :material-language-python: The implementation and release toolchain follow the Python ecosystem, including its LSP-client and editor-extension packaging conventions.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD need a maintained semantic-value-to-source-span bridge because PyLD processes JSON-LD values rather than authored YAML or Markdown syntax.
- [ ] :material-source-branch: PyLD dependency and behavioural updates need compatibility tracking behind a stable LSP-facing semantic model.
