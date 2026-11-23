---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:performance-benchmark
$type: pm:Task
title: Measure startup time, memory use, and query performance at scale
contributesTo: urn:sparqld:roadmap:mission
blockedBy:
- urn:sparqld:roadmap:configuration
- urn:sparqld:roadmap:query
hide: [toc]
---

# Measure performance at scale

Build a reproducible, bounded benchmark for loading local linked data and
querying it. Record startup time to a verified query result, peak memory during
startup, memory after loading, and query latency. Record the machine, version,
cache state, input size, triple count, file count, and query results.

Evaluate [WatDiv](https://dsg-uwaterloo.github.io/watdiv/) for scalable graphs
and query workloads, and [SolidBench](https://comunica.github.io/Article-EDBT2023-SolidQuery/)
for many-file layouts. Vary graph size and file count independently. Document
any adaptations to published workloads and retain reproducible results.

Use the [project configuration](configuration.md) and the [one-off query
command](query.md). Measure load and query times separately to explain the
total. Check load errors and expected results before accepting a measurement.
