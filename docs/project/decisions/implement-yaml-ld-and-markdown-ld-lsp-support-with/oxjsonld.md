---
title: "Build on OxJSONLD"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on OxJSONLD

{{ universe_metadata('index.md', status) }}

This planning path uses OxJSONLD as the JSON-LD semantic foundation, not as an existing LSP. “Universe” is planning shorthand for the consequences of committing to this foundation.

[OxJSONLD](https://crates.io/crates/oxjsonld) is separately published as the `oxjsonld` crate, while its crate manifest [:fontawesome-brands-github: `Cargo.toml`](https://github.com/oxigraph/oxigraph/blob/main/lib/oxjsonld/Cargo.toml) places its source in the Oxigraph monorepo.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Select or create the LSP transport and document store, including URI, position-conversion, lifecycle, capability-registration, and workspace-index boundaries.
- [ ] :material-file-code-outline: Preserve a source map for JSON-LD, YAML-LD, and Markdown-LD that maps semantic values and JSON-LD paths back to authored key and scalar spans.
- [ ] :material-code-json: Feed semantic JSON-LD values into [OxJSONLD](https://docs.rs/oxjsonld/latest/oxjsonld/) while retaining their source paths; do not report diagnostics against a serialized intermediate JSON document.
- [ ] :material-book-open-variant: Define the context and document-loader policy, including relative local-IRI resolution, allowed local documents, remote-context handling, and contextual base IRI selection.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued values, expose clickable links, and maintain a workspace index that returns every definition for an IRI.
- [ ] :material-content-save-outline: Make unsaved buffers and incremental edits authoritative for parsing, context resolution, diagnostics, document links, and the definition index.
- [ ] :material-alert-circle-outline: Translate [OxJSONLD processing errors](https://docs.rs/oxjsonld/latest/oxjsonld/) into diagnostics at the original YAML-LD or Markdown-LD source spans.
- [ ] :material-test-tube: Add protocol and end-to-end LSP tests covering diagnostic ranges, context errors, `@type: @id` links, relative cross-file resolution, multiple definition targets, and unsaved edits.
- [ ] :material-package-variant-closed: Publish a Rust binary and editor-installation path, including file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project must supply the complete LSP transport, workspace model, syntax parsing, source mapping, navigation, and editor-distribution layers around OxJSONLD.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD require a maintained semantic-value-to-source-span bridge because OxJSONLD receives JSON-LD values rather than their original YAML or Markdown syntax.
- [ ] :material-source-branch: Changes in the Oxigraph monorepo can affect the separately versioned [OxJSONLD crate](https://crates.io/crates/oxjsonld), requiring dependency and compatibility tracking.
