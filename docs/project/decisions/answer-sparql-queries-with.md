---
"@context":
  schema: https://schema.org/
  ibis: https://vocab.methodandstructure.com/ibis#
  name: schema:name
  title: schema:headline
  status: schema:creativeWorkStatus
$id: urn:sparqld:decision:inference
$type: ibis:Issue
ibis:response:
  - $type: ibis:Position
    $id: urn:sparqld:decision:inference:owl
    name: OWL
    status: undecided
  - $type: ibis:Position
    $id: urn:sparqld:decision:inference:datalog
    name: Datalog
    status: undecided
  - $type: ibis:Position
    $id: urn:sparqld:decision:inference:none-by-design
    name: None by design
    status: undecided
title: sparqld should provide inference via …
status: undecided
date: 2026-11-13
author: Anatoly Scherbakov
tags: [decision]
hide: [toc]
---

# `sparqld` should provide inference via …

{{ adr_metadata(date, status) }}

## :material-text-box-outline: Context

The roadmap goal is “Users can opt in to inferred statements while retaining asserted data unchanged.” The semantics and approach for inference remain unresolved, so this record will compare mutually exclusive ways to answer SPARQL queries with inference.

## :material-arrow-decision-outline: Decision

=== ":question: OWL"

    <div class="grid" markdown>
    <div markdown>
    #### Pro
    </div>

    <div markdown>
    #### Contra
    </div>
    </div>

=== ":question: Datalog"

    <div class="grid" markdown>
    <div markdown>
    #### Pro
    </div>

    <div markdown>
    #### Contra
    </div>
    </div>

=== ":question: None by design"

    <div class="grid" markdown>
    <div markdown>
    #### Pro
    </div>

    <div markdown>
    #### Contra
    </div>
    </div>

## :material-arrow-right-bold-outline: Consequences

Record consequences after evaluating the alternatives.
