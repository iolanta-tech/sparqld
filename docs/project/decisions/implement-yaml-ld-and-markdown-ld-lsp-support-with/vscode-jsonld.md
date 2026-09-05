---
title: "Build on vscode-jsonld"
status: excluded
date: 2026-08-23
author: Anatoly Scherbakov
hide: [toc]
---

# Build on vscode-jsonld

{{ universe_metadata('index.md', status) }}

This planning path assumes that `vscode-jsonld` becomes an installable LSP foundation. It is currently unavailable through the VS Code Marketplace; this page records the work that would become necessary if publication made that foundation usable.

[:fontawesome-brands-github: `alexkreidler/vscode-jsonld`](https://github.com/alexkreidler/vscode-jsonld) declares repository version `1.0.0` [:fontawesome-brands-github: `package.json`](https://github.com/alexkreidler/vscode-jsonld/blob/master/package.json), but its [expected VS Code Marketplace item is absent](https://marketplace.visualstudio.com/items?itemName=vscode-samples.vscode-jsonld).

## :material-format-list-checks: Implementation checklist

- [ ] :material-package-variant-closed: Publish a release and an installable extension or standalone server package before depending on this code as an LSP foundation.
- [ ] :material-lan-connect: Audit the VS Code extension and language-server boundary; extract a generic server interface that does not require the VS Code extension host.
- [ ] :material-file-tree-outline: Implement YAML-LD parsing with a source map from JSON-LD value paths to authored YAML key and scalar spans.
- [ ] :material-language-markdown-outline: Extract Markdown-LD front matter, preserve its document offsets, and map diagnostics and navigation ranges back to the Markdown buffer.
- [ ] :material-graph-outline: Define JSON-LD context loading, local-document access, and relative-IRI base-resolution policies before connecting semantic processing to workspace files.
- [ ] :material-link-variant: Interpret `@type: @id` coercion to distinguish IRI-valued strings from literals; emit clickable document links and all matching workspace definitions.
- [ ] :material-content-save-outline: Make in-memory, unsaved buffers authoritative for parsing, context resolution, indexing, links, and navigation.
- [ ] :material-test-tube: Test the extracted generic server with at least one non-VS Code LSP client, including diagnostics, links, multiple definitions, relative IRIs, and unsaved edits.
- [ ] :material-package-variant-closed: Distribute the generic server independently, document editor configuration and file associations, and keep the extension package aligned with it.

## :material-alert-outline: Consequences

- [ ] :material-source-branch: Maintain a fork or upstream a generic server boundary if the extension’s architecture cannot be reused without VS Code assumptions.
- [ ] :material-file-tree-outline: Own the YAML-LD and Markdown-LD parsers, source mappings, workspace index, and their regression suites.
- [ ] :material-package-variant-closed: Operate and support a publication channel that does not exist for this project today.
- [ ] :material-account-wrench: Track compatibility between the generic server, the VS Code extension, and other LSP clients as separate release surfaces.
