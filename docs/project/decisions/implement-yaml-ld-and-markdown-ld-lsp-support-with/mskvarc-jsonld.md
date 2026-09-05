---
title: "Build on mskvarc/jsonld"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on mskvarc/jsonld

{{ universe_metadata('index.md', status) }}

This planning path uses `mskvarc/jsonld` as the JSON-LD semantic foundation, not as an existing LSP. “Universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `mskvarc/jsonld`](https://github.com/mskvarc/jsonld) is published as the [`jsonld` crate](https://crates.io/crates/jsonld).

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Select or create the LSP transport and document store, including URI, position-conversion, lifecycle, capability-registration, and workspace-index boundaries.
- [ ] :material-file-code-outline: Parse JSON-LD, YAML-LD, and Markdown-LD into source-mapped semantic values that retain authored key and scalar spans.
- [ ] :material-code-json: Bridge [`jsonld`](https://docs.rs/jsonld/latest/jsonld/) semantic processing and errors to original source paths; do not report diagnostics against a serialized intermediate JSON document.
- [ ] :material-book-open-variant: Define the context and local-IRI loader policy, including relative resolution, allowed local documents, remote-context handling, and contextual base IRI selection.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued values, expose clickable links, and maintain a workspace index that returns every definition for an IRI.
- [ ] :material-content-save-outline: Make unsaved buffers and incremental edits authoritative for parsing, context resolution, diagnostics, document links, and the definition index.
- [ ] :material-alert-circle-outline: Translate [`jsonld` processing errors](https://docs.rs/jsonld/latest/jsonld/) into diagnostics at the original JSON-LD, YAML-LD, or Markdown-LD source spans.
- [ ] :material-test-tube: Add protocol and end-to-end LSP tests covering diagnostic ranges, context errors, `@type: @id` links, relative cross-file resolution, multiple definition targets, and unsaved edits.
- [ ] :material-package-variant-closed: Publish a Rust binary and editor-installation path, including file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project must supply the complete LSP transport, workspace model, syntax parsing, source mapping, navigation, and editor-distribution layers around `jsonld`.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD require a maintained semantic-value-to-source-span bridge because `jsonld` receives JSON-LD values rather than their original YAML or Markdown syntax.
- [ ] :material-package-variant-closed: The published [`jsonld` crate](https://crates.io/crates/jsonld) becomes a direct compatibility and release-management dependency for the language server.
