---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:streaming
$type: pm:Goal
title: Users can retrieve large query results without complete-response
  buffering
contributesTo: urn:sparqld:roadmap:mission
hide: [toc]
---

# Retrieve large query results progressively

Deliver query results progressively so large answers do not require buffering
the complete response before transmission. This goal concerns response delivery
for users retrieving large result sets.

Verify that a client receives result data before query output is complete.
Measure response buffering with a result set large enough to expose full
response allocation.
