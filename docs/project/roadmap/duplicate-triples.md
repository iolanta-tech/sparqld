---
"@context": urn:sparqld:roadmap
$id: https://github.com/iolanta-tech/sparqld/issues/26
$type: urn:sparqld:vocabulary:Bug
contributesTo: urn:sparqld:roadmap:mission
blockedBy: urn:sparqld:roadmap:oxigraph-fix-available
subjectOf: sparqld:src/server.rs#union_default_graph_counts_a_shared_triple_once
seeAlso: https://github.com/oxigraph/oxigraph/issues/1919
hide: [toc]
---

# Count shared triples once

A triple present in several source graphs must count once when a query reads
the union default graph. Preserve the regression test identified in this record
when updating Oxigraph.

The recorded prerequisite is an Oxigraph release containing the upstream
fix for repeated triples. Track the defect in the [GitHub
issue](https://github.com/iolanta-tech/sparqld/issues/26) and the fix in [the
upstream pull request](https://github.com/oxigraph/oxigraph/pull/1920).
