---
title: "Build on JSONLD-Java"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on JSONLD-Java

{{ universe_metadata('index.md', status) }}

This planning path uses JSONLD-Java as the JSON-LD semantic foundation and builds an LSP around it. It does not adopt an existing LSP implementation; “universe” is planning shorthand for the consequences of committing to this foundation.

[:fontawesome-brands-github: `jsonld-java/jsonld-java`](https://github.com/jsonld-java/jsonld-java) provides JSON-LD processing for Java and is published to [Maven Central as `com.github.jsonld-java:jsonld-java`](https://central.sonatype.com/artifact/com.github.jsonld-java/jsonld-java).

## :material-format-list-checks: Implementation checklist

- [ ] :material-lan-connect: Choose a Java LSP framework, transport, document store, lifecycle model, position conversion, and workspace-index boundary.
- [ ] :material-file-code-outline: Build source maps for JSON-LD, YAML-LD, and Markdown-LD that map semantic values and JSON-LD paths to authored key and scalar spans.
- [ ] :material-code-json: Bridge JSONLD-Java processing results and errors to mapped source paths instead of reporting diagnostics against a serialized JSON intermediate.
- [ ] :material-book-open-variant: Define context and document-loader policy, including relative local-IRI resolution, allowed local documents, remote-context handling, and contextual base-IRI selection.
- [ ] :material-link-variant: Interpret JSON-LD [`@type: @id` coercion](https://www.w3.org/TR/json-ld11/#type-coercion) to classify IRI-valued values, expose clickable links, and return every matching workspace definition.
- [ ] :material-content-save-outline: Make unsaved buffers and incremental edits authoritative for parsing, context resolution, diagnostics, document links, and the definition index.
- [ ] :material-test-tube: Add JSON-LD conformance, LSP protocol, source-range, navigation, and editor end-to-end tests, including multiple definition targets and unsaved edits.
- [ ] :material-package-variant-closed: Publish a JVM package, language-server launch path, and editor-installation metadata for JSON-LD, YAML-LD, and Markdown-LD.

## :material-call-split: Universe consequences

- [ ] :material-hammer-wrench: The project supplies the complete LSP transport, workspace model, syntax parsing, source mapping, navigation, and editor-distribution layers around JSONLD-Java.
- [ ] :material-file-tree-outline: YAML-LD and Markdown-LD need a maintained semantic-value-to-source-span bridge because JSONLD-Java receives JSON-LD values rather than authored YAML or Markdown syntax.
- [ ] :material-language-java: The server, packaging, editor launch configuration, and contributor toolchain become JVM-based.
