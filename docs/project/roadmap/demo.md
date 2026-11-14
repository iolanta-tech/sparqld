---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:demo
$type: pm:Goal
title: MkDocs users can reproduce the sparql() demo against documented
  example data
contributesTo: urn:sparqld:roadmap:mission
hide: [toc]
---

# Reproduce the MkDocs SPARQL demo

Make the MkDocs integration example reproducible from its documented data and
query sources. A contributor should be able to build the example and compare
the rendered answer with the documented result.

Use the same scenario files for documentation and integration tests. See the
[MkDocs integration](../../reference/integrations/mkdocs.md) for the current
pluglet setup.
