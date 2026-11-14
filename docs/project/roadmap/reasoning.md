---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:reasoning
$type: pm:Goal
title: Users can opt in to inferred statements while retaining asserted
  data unchanged
contributesTo: urn:sparqld:roadmap:mission
blockedBy: urn:sparqld:decision:inference
hide: [toc]
---

# Opt in to inferred statements

Choose inference semantics before implementing this outcome. The [inference
decision](../decisions/answer-sparql-queries-with.md) compares the available
approaches and blocks this goal.

The resulting workflow must let users enable inferred statements while keeping
their asserted source data unchanged. Verify query answers and source data
preservation with a shared example scenario.
