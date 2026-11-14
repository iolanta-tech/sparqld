---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:ask
$type: pm:Goal
title: MkDocs authors can render ASK answers as SPARQL true and false
contributesTo: urn:sparqld:roadmap:mission
hide: [toc]
---

# Render ASK answers as SPARQL boolean values

The MkDocs pluglet parses an `ASK` response as a Python boolean. Direct
template rendering must produce the SPARQL spellings `true` and `false` so
authors can show query answers consistently.

Verify both answers in rendered MkDocs pages. Keep query and data fixtures in
the shared example sources.
