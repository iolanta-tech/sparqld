---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
  datePublished: schema:datePublished
"@id": project/roadmap.md
"@type": schema:TechArticle
name: sparqld roadmap
hide: [toc]
title: Roadmap
description: Achievable outcomes that advance sparqld's mission.
datePublished: 2026-11-13
---

# :material-map-marker-path: Roadmap

Arrows point from a blocking outcome to the outcome it enables. The mission is
an enduring direction rather than a completion criterion. Context download
records use [RO-Crate](https://www.researchobject.org/ro-crate/).

```mermaid
flowchart LR
    fragments("Users can query every embedded graph,<br/>including fragment IRIs, without collisions")
    streaming("Users can retrieve large query results<br/>without complete-response buffering")
    configuration("Users can declare reproducible project settings<br/>with documented command-line precedence")
    remote("Users can query local data with cached Linked Data<br/>from explicitly approved remote hosts")
    reasoning("Users can opt in to inferred statements<br/>while retaining asserted data unchanged")
    crateContexts("Users can resolve a context URL from the RO-Crate that records its download,<br/>with the link inferred from crate metadata")
    python("MkDocs projects on Python 3.11 and 3.12<br/>can install the pluglet")
    demo("MkDocs users can reproduce the sparql() demo<br/>against documented example data")
    ask("MkDocs authors can render ASK answers<br/>as SPARQL true and false")
    discovery("Standards-aware clients can discover<br/>sparqld's query service from its endpoint")
    obsidian("Users can run SPARQL against a watched directory<br/>from an Obsidian plugin")
    mission("<strong>Mission</strong><br/>Continuously improve the experience of people and agents<br/>querying local Linked Data with sparqld")

    configuration --> remote
    reasoning --> crateContexts

    fragments --> mission
    streaming --> mission
    remote --> mission
    crateContexts --> mission
    python --> mission
    demo --> mission
    ask --> mission
    discovery --> mission
    obsidian --> mission
```
