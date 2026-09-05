---
"@context": ../context.yamlld
"@id": project/decisions/implement-yaml-ld-and-markdown-ld-lsp-support-with/index.md
"@type": schema:TechArticle
title: "Implement YAML-LD and Markdown-LD LSP support on SWLS"
status: decided
date: 2026-08-23
author: Anatoly Scherbakov
tags: [decision]
hide: [toc]
---

# Implement YAML-LD and Markdown-LD LSP support on SWLS

{{ adr_metadata(date, status) }}

## :material-text-box-outline: Context

This ADR evaluates foundations for a generic LSP supporting JSON-LD, YAML-LD, and Markdown-LD, without assigning it to a particular consumer.

### :material-format-list-checks: Requirements

- Parse JSON-LD, YAML-LD, and Markdown-LD while preserving authored source spans.
- Interpret JSON-LD contexts, including `@type: @id` coercion, to distinguish IRI-valued values from literals.
- Expose IRI-valued values as clickable document links.
- Make Go to Definition return every matching definition in the workspace when an IRI has several definitions.
- Resolve relative local IRIs and support unsaved buffers.

## :material-arrow-decision-outline: Decision

**Assessment legend.** These are ADR judgments derived from the linked universe
checklists. [★★](swls.md "Medium relative implementation difficulty") means
Medium relative implementation difficulty; [★★★](oxjsonld.md "Hard relative implementation difficulty")
means Hard relative implementation difficulty. [✓✓✓](swls.md "Three of three expected LSP reuse")
means three of three expected LSP reuse; [✓✓](stardog-language-servers.md "Two of three expected LSP reuse")
means two of three; [✓](turtle-language-server.md "One of three expected LSP reuse")
means one of three; and [∅](oxjsonld.md "Zero of three expected LSP reuse")
means zero of three. [:material-account-group:](swls.md "Shared upstream plus local extensions")
means shared upstream plus local extensions; [:material-source-fork:](stardog-language-servers.md "Fork or extraction plus replacement pipeline")
means a fork or extraction plus replacement pipeline; and [:material-hammer-wrench:](oxjsonld.md "Locally owned new or full server layers")
means locally owned new or full server layers.

<table data-adr-comparison markdown="1">
  <tr markdown="span">
    <th>Universe</th>
    <th>Alternative</th>
    <th>Language</th>
    <th>Stars</th>
    <th>Latest published version</th>
    <th>Difficulty (ADR assessment)</th>
    <th>Expected LSP reuse (ADR assessment)</th>
    <th>Expected maintenance ownership (ADR assessment)</th>
    <th>Decision</th>
  </tr>
  <tr markdown="span">
    <th colspan="9">Build on an existing LSP implementation</th>
  </tr>
  <tr markdown="span">
    <td class="chosen">[:material-orbit: Universe](swls.md)</td>
    <th class="chosen">[:fontawesome-brands-github: `SemanticWebLanguageServer/swls`](https://github.com/SemanticWebLanguageServer/swls)</th>
    <td class="chosen">:simple-rust: Rust [:fontawesome-brands-github: `SemanticWebLanguageServer/swls`](https://github.com/SemanticWebLanguageServer/swls)</td>
    <td class="chosen">53 [:fontawesome-brands-github: `SemanticWebLanguageServer/swls`](https://github.com/SemanticWebLanguageServer/swls/stargazers)</td>
    <td class="chosen">swls-v0.4.1, 2026-07-07 [:fontawesome-brands-github: `SemanticWebLanguageServer/swls`](https://github.com/SemanticWebLanguageServer/swls/releases/tag/swls-v0.4.1)</td>
    <td class="chosen">[★★](swls.md "Medium relative implementation difficulty")</td>
    <td class="chosen">[✓✓✓](swls.md "Three of three expected LSP reuse")</td>
    <td class="chosen">[:material-account-group:](swls.md "Shared upstream plus local extensions")</td>
    <td class="chosen">:white_check_mark: Chosen</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](stardog-language-servers.md)</td>
    <th class="excl">[:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers)</th>
    <td class="excl">:simple-typescript: TypeScript [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers)</td>
    <td class="excl">40 [:fontawesome-brands-github: `stardog-union/stardog-language-servers`](https://github.com/stardog-union/stardog-language-servers/stargazers)</td>
    <td class="excl">[sparql-language-server v4.3.0, 2024-08-12](https://www.npmjs.com/package/sparql-language-server/v/4.3.0)</td>
    <td class="excl hot">[★★★](stardog-language-servers.md "Hard relative implementation difficulty")</td>
    <td class="excl">[✓✓](stardog-language-servers.md "Two of three expected LSP reuse")</td>
    <td class="excl">[:material-source-fork:](stardog-language-servers.md "Fork or extraction plus replacement pipeline")</td>
    <td class="excl">:x: [Hard implementation effort](stardog-language-servers.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](qlue-ls.md)</td>
    <th class="excl">[:fontawesome-brands-github: `IoannisNezis/Qlue-ls`](https://github.com/IoannisNezis/Qlue-ls)</th>
    <td class="excl">:simple-rust: Rust [:fontawesome-brands-github: `IoannisNezis/Qlue-ls`](https://github.com/IoannisNezis/Qlue-ls)</td>
    <td class="excl">47 [:fontawesome-brands-github: `IoannisNezis/Qlue-ls`](https://github.com/IoannisNezis/Qlue-ls/stargazers)</td>
    <td class="excl">v3.4.2, 2026-08-21 [:fontawesome-brands-github: `IoannisNezis/Qlue-ls`](https://github.com/IoannisNezis/Qlue-ls/releases/tag/v3.4.2)</td>
    <td class="excl hot">[★★★](qlue-ls.md "Hard relative implementation difficulty")</td>
    <td class="excl">[✓✓](qlue-ls.md "Two of three expected LSP reuse")</td>
    <td class="excl">[:material-source-fork:](qlue-ls.md "Fork or extraction plus replacement pipeline")</td>
    <td class="excl">:x: [Hard implementation effort](qlue-ls.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](turtle-language-server.md)</td>
    <th class="excl">[:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver)</th>
    <td class="excl">:simple-python: Python [:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver)</td>
    <td class="excl">7 [:fontawesome-brands-github: `BrickSchema/langserver`](https://github.com/BrickSchema/langserver/stargazers)</td>
    <td class="excl">[turtle-language-server v0.1.2, 2021-01-13](https://pypi.org/project/turtle-language-server/0.1.2/)</td>
    <td class="excl hot">[★★★](turtle-language-server.md "Hard relative implementation difficulty")</td>
    <td class="excl">[✓](turtle-language-server.md "One of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](turtle-language-server.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](turtle-language-server.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](shaclc-language-server.md)</td>
    <th class="excl">[:fontawesome-brands-github: `jeswr/shaclc-language-server`](https://github.com/jeswr/shaclc-language-server)</th>
    <td class="excl">:simple-typescript: TypeScript [:fontawesome-brands-github: `jeswr/shaclc-language-server`](https://github.com/jeswr/shaclc-language-server)</td>
    <td class="excl">1 [:fontawesome-brands-github: `jeswr/shaclc-language-server`](https://github.com/jeswr/shaclc-language-server/stargazers)</td>
    <td class="excl">[v0.0.1, 2023-01-02](https://marketplace.visualstudio.com/items?itemName=jeswr.shaclc-language-server)</td>
    <td class="excl hot">[★★★](shaclc-language-server.md "Hard relative implementation difficulty")</td>
    <td class="excl">[✓](shaclc-language-server.md "One of three expected LSP reuse")</td>
    <td class="excl">[:material-source-fork:](shaclc-language-server.md "Fork or extraction plus replacement pipeline")</td>
    <td class="excl">:x: [Hard implementation effort](shaclc-language-server.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](rdflangserver.md)</td>
    <th class="excl">[:fontawesome-brands-github: `niklasl/rdflangserver`](https://github.com/niklasl/rdflangserver)</th>
    <td class="excl">:simple-python: Python [:fontawesome-brands-github: `niklasl/rdflangserver`](https://github.com/niklasl/rdflangserver)</td>
    <td class="excl">0 [:fontawesome-brands-github: `niklasl/rdflangserver`](https://github.com/niklasl/rdflangserver/stargazers)</td>
    <td class="excl hot">No published release; repository version 0.1.0-dev [:fontawesome-brands-github: `pyproject.toml`](https://github.com/niklasl/rdflangserver/blob/main/pyproject.toml)</td>
    <td class="excl"></td>
    <td class="excl"></td>
    <td class="excl"></td>
    <td class="excl">:x: [not published to PyPI](https://pypi.org/project/rdflangserver/)</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](vscode-jsonld.md)</td>
    <th class="excl">[:fontawesome-brands-github: `alexkreidler/vscode-jsonld`](https://github.com/alexkreidler/vscode-jsonld)</th>
    <td class="excl">:simple-typescript: TypeScript [:fontawesome-brands-github: `alexkreidler/vscode-jsonld`](https://github.com/alexkreidler/vscode-jsonld)</td>
    <td class="excl">0 [:fontawesome-brands-github: `alexkreidler/vscode-jsonld`](https://github.com/alexkreidler/vscode-jsonld/stargazers)</td>
    <td class="excl hot">No published release; repository version 1.0.0 [:fontawesome-brands-github: `package.json`](https://github.com/alexkreidler/vscode-jsonld/blob/master/package.json)</td>
    <td class="excl"></td>
    <td class="excl"></td>
    <td class="excl"></td>
    <td class="excl">:x: [not published to VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=vscode-samples.vscode-jsonld)</td>
  </tr>
  <tr markdown="span">
    <th colspan="9">Build on an existing JSON-LD parser implementation</th>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](oxjsonld.md)</td>
    <th class="excl">[:fontawesome-brands-github: `oxigraph/oxigraph`](https://github.com/oxigraph/oxigraph) (oxjsonld)</th>
    <td class="excl">:simple-rust: Rust [:fontawesome-brands-github: `oxigraph/oxigraph`](https://github.com/oxigraph/oxigraph)</td>
    <td class="excl">1833 [:fontawesome-brands-github: `oxigraph/oxigraph`](https://github.com/oxigraph/oxigraph/stargazers)</td>
    <td class="excl">[oxjsonld v0.2.5, 2026-04-19](https://crates.io/crates/oxjsonld/0.2.5)</td>
    <td class="excl hot">[★★★](oxjsonld.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](oxjsonld.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](oxjsonld.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](oxjsonld.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](json-ld-rs.md)</td>
    <th class="excl">[:fontawesome-brands-github: `timothee-haudebourg/json-ld`](https://github.com/timothee-haudebourg/json-ld)</th>
    <td class="excl">:simple-rust: Rust [:fontawesome-brands-github: `timothee-haudebourg/json-ld`](https://github.com/timothee-haudebourg/json-ld)</td>
    <td class="excl">154 [:fontawesome-brands-github: `timothee-haudebourg/json-ld`](https://github.com/timothee-haudebourg/json-ld/stargazers)</td>
    <td class="excl">[json-ld v0.21.4, 2026-02-19](https://crates.io/crates/json-ld/0.21.4)</td>
    <td class="excl hot">[★★★](json-ld-rs.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](json-ld-rs.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](json-ld-rs.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](json-ld-rs.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](sophia-jsonld.md)</td>
    <th class="excl">[:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs) (sophia_jsonld)</th>
    <td class="excl">:simple-rust: Rust [:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs)</td>
    <td class="excl">328 [:fontawesome-brands-github: `pchampin/sophia_rs`](https://github.com/pchampin/sophia_rs/stargazers)</td>
    <td class="excl">[sophia_jsonld v0.10.0, 2026-05-19](https://crates.io/crates/sophia_jsonld/0.10.0)</td>
    <td class="excl hot">[★★★](sophia-jsonld.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](sophia-jsonld.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](sophia-jsonld.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](sophia-jsonld.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](mskvarc-jsonld.md)</td>
    <th class="excl">[:fontawesome-brands-github: `mskvarc/jsonld`](https://github.com/mskvarc/jsonld)</th>
    <td class="excl">:simple-rust: Rust [:fontawesome-brands-github: `mskvarc/jsonld`](https://github.com/mskvarc/jsonld)</td>
    <td class="excl">0 [:fontawesome-brands-github: `mskvarc/jsonld`](https://github.com/mskvarc/jsonld/stargazers)</td>
    <td class="excl">[jsonld v0.22.0, 2026-08-04](https://crates.io/crates/jsonld/0.22.0)</td>
    <td class="excl hot">[★★★](mskvarc-jsonld.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](mskvarc-jsonld.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](mskvarc-jsonld.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](mskvarc-jsonld.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](jsonld-java.md)</td>
    <th class="excl">[:fontawesome-brands-github: `jsonld-java/jsonld-java`](https://github.com/jsonld-java/jsonld-java)</th>
    <td class="excl">:simple-openjdk: Java [:fontawesome-brands-github: `jsonld-java/jsonld-java`](https://github.com/jsonld-java/jsonld-java)</td>
    <td class="excl">388 [:fontawesome-brands-github: `jsonld-java/jsonld-java`](https://github.com/jsonld-java/jsonld-java/stargazers)</td>
    <td class="excl">[jsonld-java v0.13.6, 2023-11-06](https://central.sonatype.com/artifact/com.github.jsonld-java/jsonld-java/0.13.6)</td>
    <td class="excl hot">[★★★](jsonld-java.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](jsonld-java.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](jsonld-java.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](jsonld-java.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](json-gold.md)</td>
    <th class="excl">[:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold)</th>
    <td class="excl">:simple-go: Go [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold)</td>
    <td class="excl">313 [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold/stargazers)</td>
    <td class="excl">v0.8.0, 2026-02-23 [:fontawesome-brands-github: `piprate/json-gold`](https://github.com/piprate/json-gold/releases/tag/v0.8.0)</td>
    <td class="excl hot">[★★★](json-gold.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](json-gold.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](json-gold.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](json-gold.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](jsonld-js.md)</td>
    <th class="excl">[:fontawesome-brands-github: `digitalbazaar/jsonld.js`](https://github.com/digitalbazaar/jsonld.js)</th>
    <td class="excl">:simple-javascript: JavaScript [:fontawesome-brands-github: `digitalbazaar/jsonld.js`](https://github.com/digitalbazaar/jsonld.js)</td>
    <td class="excl">1816 [:fontawesome-brands-github: `digitalbazaar/jsonld.js`](https://github.com/digitalbazaar/jsonld.js/stargazers)</td>
    <td class="excl">[jsonld v9.0.0, 2025-11-21](https://www.npmjs.com/package/jsonld/v/9.0.0)</td>
    <td class="excl hot">[★★★](jsonld-js.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](jsonld-js.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](jsonld-js.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](jsonld-js.md "Hard relative implementation difficulty")</td>
  </tr>
  <tr markdown="span">
    <td class="excl">[:material-orbit: Universe](pyld.md)</td>
    <th class="excl">[:fontawesome-brands-github: `digitalbazaar/pyld`](https://github.com/digitalbazaar/pyld)</th>
    <td class="excl">:simple-python: Python [:fontawesome-brands-github: `digitalbazaar/pyld`](https://github.com/digitalbazaar/pyld)</td>
    <td class="excl">678 [:fontawesome-brands-github: `digitalbazaar/pyld`](https://github.com/digitalbazaar/pyld/stargazers)</td>
    <td class="excl">[PyLD v3.2.0, 2026-08-17](https://pypi.org/project/PyLD/3.2.0/)</td>
    <td class="excl hot">[★★★](pyld.md "Hard relative implementation difficulty")</td>
    <td class="excl">[∅](pyld.md "Zero of three expected LSP reuse")</td>
    <td class="excl">[:material-hammer-wrench:](pyld.md "Locally owned new or full server layers")</td>
    <td class="excl">:x: [Hard implementation effort](pyld.md "Hard relative implementation difficulty")</td>
  </tr>
</table>

## :material-arrow-right-bold-outline: Consequences

- SWLS is the implementation foundation for YAML-LD and Markdown-LD LSP support.
- YAML-LD and Markdown-LD source mapping and JSON-LD semantics remain local extension work.
