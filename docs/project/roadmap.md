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
datePublished: 2026-08-14
---

# :material-map-marker-path: Roadmap

Arrows point from a blocking outcome to the outcome it enables. The mission is
an enduring direction rather than a completion criterion.

```mermaid
flowchart LR
    fragments("Users can query every embedded graph,<br/>including fragment IRIs, without collisions")
    exclusions("Users can keep selected paths, including .git,<br/>outside the watched dataset")
    streaming("Users can retrieve large query results<br/>without complete-response buffering")
    configuration("Users can declare reproducible project settings<br/>with documented command-line precedence")
    remote("Users can query local data with cached Linked Data<br/>from explicitly approved remote hosts")
    reasoning("Users can opt in to OWL 2 RL entailments<br/>while retaining asserted data unchanged")
    python("MkDocs projects on Python 3.11 and 3.12<br/>can install the pluglet")
    demo("MkDocs users can reproduce the sparql() demo<br/>against documented example data")
    ask("MkDocs authors can render ASK answers<br/>as SPARQL true and false")
    graphCounts("Readers can identify the data and catalog graphs<br/>included by the graph-count recipe")
    heading("Readers can navigate the MkDocs integration reference<br/>through its macro-section heading")
    discovery("Standards-aware clients can discover<br/>sparqld's query service from its endpoint")
    mission("<strong>Mission</strong><br/>Continuously improve the experience of people and agents<br/>querying local Linked Data with sparqld")

    configuration --> remote

    fragments --> mission
    exclusions --> mission
    streaming --> mission
    configuration --> mission
    remote --> mission
    reasoning --> mission
    python --> mission
    demo --> mission
    ask --> mission
    graphCounts --> mission
    heading --> mission
    discovery --> mission
```
