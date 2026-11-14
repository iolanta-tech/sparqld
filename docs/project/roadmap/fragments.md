---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:fragments
$type: urn:sparqld:vocabulary:Bug
title: Embedded graph IRIs containing fragments fail to load
contributesTo: urn:sparqld:roadmap:mission
hide: [toc]
---

# Load embedded graph IRIs with fragments

Investigate the recorded loading failure for embedded graph IRIs containing
fragment identifiers. Preserve the full graph IRI when loading the embedded RDF
data.

Establish a regression scenario with a fragment-bearing graph IRI. Verify that
the graph loads and its statements remain queryable.
