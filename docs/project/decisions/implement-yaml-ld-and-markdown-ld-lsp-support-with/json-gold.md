---
title: "Build on JSON-Gold"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on JSON-Gold

{{ universe_metadata('index.md', status) }}

This planning path uses JSON-Gold as the JSON-LD semantic foundation and builds an LSP around it. JSON-Gold supplies JSON-LD processing, not LSP transport, workspace state, or editor integration; “universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold) is a Go JSON-LD processor; its latest v0.8.0 release (2026-02-23) [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold/releases/tag/v0.8.0) is published on GitHub.

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Choose a Go LSP framework, transport, document model, position conversion, lifecycle handling, and capability-registration boundaries.
- [ ] :material-file-code-outline: Build JSON-LD, YAML-LD, and Markdown front-matter syntax layers that preserve source maps from semantic value paths to authored key and scalar ranges.
- [ ] :material-code-json: Connect mapped semantic values and JSON-Gold processing [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold) errors to their original source spans rather than generated JSON text.
- [ ] :material-book-open-variant: Define the context, document-loader, and local relative-IRI policy, including contextual base IRIs, permitted local paths, and remote-context handling.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued scalars, publish clickable links, and return every workspace definition for an IRI.
- [ ] :material-content-save-outline: Keep unsaved buffers and incremental edits authoritative for parsing, context loading, diagnostics, document links, and the workspace definition index.
- [ ] :material-test-tube: Add JSON-LD conformance and LSP end-to-end tests for diagnostic ranges, context errors, `@type: @id` links, relative cross-file resolution, multiple definition targets, and unsaved edits.
- [ ] :material-package-variant-closed: Build and distribute a Go binary with editor-installation guidance and file associations for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project owns the LSP transport, incremental document model, syntax parsing, source mapping, navigation, workspace index, and editor-distribution layers around JSON-Gold.
- [ ] :material-language-go: The implementation, dependency management, cross-compilation, and release process follow the Go ecosystem.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD require a maintained semantic-value-to-source-span bridge because JSON-Gold processes JSON-LD values rather than authored YAML or Markdown syntax.
- [ ] :material-source-branch: JSON-Gold dependency and behavioural changes require compatibility tracking behind a stable LSP-facing semantic model.
