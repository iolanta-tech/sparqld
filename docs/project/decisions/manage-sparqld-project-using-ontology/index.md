---
"@context": urn:sparqld:context:decisions
$id: urn:sparqld:decision:manage-sparqld-project-using-ontology
$type: ibis:Issue
decides:
  - https://vocab.methodandstructure.com/ibis#
  - https://vocab.methodandstructure.com/process-model
ibis:response:
  - "@type": ibis:Position
    "@id": https://oslc-op.github.io/oslc-specs/specs/cm/change-mgt-vocab.html
    title: OSLC Change Management
  - "@type": ibis:Position
    "@id": https://ipocore.sourceforge.io/
    title: Issue Procedure Ontology
  - "@type": ibis:Position
    "@id": https://se-on.org/
    title: SEON
  - "@type": ibis:Position
    "@id": https://vocab.methodandstructure.com/ibis#
    title: IBIS and gIBIS
  - "@type": ibis:Position
    "@id": https://vocab.methodandstructure.com/process-model
    title: Process Model Ontology
  - "@type": ibis:Position
    "@id": https://w3id.org/dio
    title: Design Intent Ontology
  - "@type": ibis:Position
    "@id": https://ontology.inferal.com/modules/decision/current/spec/
    title: Inferal Decision Ontology
title: Manage the sparqld project using IBIS and the Process Model ontology
status: decided
date: 2026-11-12
author: Anatoly Scherbakov
tags: [decision]
hide: [toc]
---

# Manage the sparqld project using IBIS and the Process Model ontology

{{ adr_metadata(date, status) }}

## :material-text-box-outline: Context

The roadmap describes desired user outcomes and their dependencies in a manually maintained Mermaid diagram. Structured project data could instead support queries and generate the roadmap, while also representing tasks, bugs, design questions, alternatives, and selected options.

## :material-arrow-decision-outline: Decision

Use IBIS and the accompanying Process Model ontology for now.

The date column identifies the latest change or published version verified for
each ontology; release and specification dates do not establish the date of
its most recent edit. An empty capability cell means no matching term was
verified. The foundation column identifies the stated modeling foundation or, where no theory is
claimed, the vocabulary's practical scope.

<table data-adr-comparison markdown="1">
  <tr markdown="span">
    <th>Ontology</th>
    <th>Last verified date</th>
    <th>Foundation</th>
    <th>Task</th>
    <th>Bug</th>
    <th>Blocking or dependency</th>
    <th>Decision or question</th>
    <th>Alternatives</th>
    <th>Chosen alternative</th>
    <th>Decision</th>
  </tr>
  <tr markdown="span">
    <th>[OSLC Change Management](https://oslc-op.github.io/oslc-specs/specs/cm/change-mgt-vocab.html)</th>
    <td>[2023-07-06](https://www.open-services.net/specifications/vocabularies/) standard with errata; last edit unknown</td>
    <td>[Linked-data integration for change-management resources](https://oslc-op.github.io/oslc-specs/specs/cm/change-mgt-vocab.html); no formal decision theory identified</td>
    <td>[`oslc_cm:Task`](http://open-services.net/ns/cm#Task)</td>
    <td>[`oslc_cm:Defect`](http://open-services.net/ns/cm#Defect)</td>
    <td>[`oslc_cm:relatedChangeRequest`](http://open-services.net/ns/cm#relatedChangeRequest) is broader than blocking</td>
    <td></td><td></td><td></td><td></td>
  </tr>
  <tr markdown="span">
    <th>[Issue Procedure Ontology](https://ipocore.sourceforge.io/)</th>
    <td>[2015-07-30](https://ipocore.sourceforge.io/) version 1.2.0; last edit unknown</td>
    <td>[Issues, symptoms, and procedures modeled as workflows](https://ipocore.sourceforge.io/)</td>
    <td></td>
    <td>[`ipo:Issue`](http://purl.org/ipo/core#Issue) is general</td>
    <td>[`ipo:directlyDependsOn`](http://purl.org/ipo/core#directlyDependsOn); broader transitive [`ipo:dependsOn`](http://purl.org/ipo/core#dependsOn)</td>
    <td></td><td></td><td></td><td></td>
  </tr>
  <tr markdown="span">
    <th>[SEON](https://se-on.org/)</th>
    <td>Last edit unknown; [ontology imports use 2012 version paths](https://se-on.org/ontologies/ontologies/all___197968168.html)</td>
    <td>[Software evolution and repository mining, with layered domain models](https://se-on.org/)</td>
    <td></td>
    <td>[Bug](https://se-on.org/) in the issue-tracking model</td>
    <td>[General `dependsOn`](https://se-on.org/) supports specialized dependencies</td>
    <td></td><td></td><td></td><td></td>
  </tr>
  <tr markdown="span">
    <th class="chosen">[:fontawesome-brands-github: `doriantaylor/owl-ibis`](https://github.com/doriantaylor/owl-ibis)</th>
    <td class="chosen">[2026-06-10](https://github.com/doriantaylor/owl-ibis/commit/cc611ab2a9521d043531a47adca8e1f1bd800f3e) repository commit</td>
    <td class="chosen">IBIS and gIBIS: issue-based reasoning and structured argumentation [:fontawesome-brands-github: `doriantaylor/owl-ibis`](https://github.com/doriantaylor/owl-ibis)</td>
    <td class="chosen"></td>
    <td class="chosen">[`ibis:Issue`](https://vocab.methodandstructure.com/ibis#Issue) is general</td>
    <td class="chosen"></td>
    <td class="chosen">[`ibis:Issue`](https://vocab.methodandstructure.com/ibis#Issue) for the question</td>
    <td class="chosen">[`ibis:Position`](https://vocab.methodandstructure.com/ibis#Position) via [`ibis:response`](https://vocab.methodandstructure.com/ibis#response)</td>
    <td class="chosen"></td><td class="chosen">:white_check_mark: Chosen</td>
  </tr>
  <tr markdown="span">
    <th class="chosen">[Process Model Ontology](https://vocab.methodandstructure.com/process-model)</th>
    <td class="chosen">[2026-06-10](https://github.com/doriantaylor/owl-process-model/commit/c72052dcb8c5aa88c50fc47abc0b98a56484dd7f) repository commit</td>
    <td class="chosen">IBIS extended with goals, states, actions, and processes [:fontawesome-brands-github: `README.md`](https://github.com/doriantaylor/owl-process-model/blob/main/README.md)</td>
    <td class="chosen">[`pm:Task`](https://vocab.methodandstructure.com/process-model#Task); subclass of IBIS position</td>
    <td class="chosen"></td>
    <td class="chosen">[`pm:dependency`](https://vocab.methodandstructure.com/process-model#dependency) relates an IBIS entity to a state; an issue's resolution can make the entity actionable [:fontawesome-brands-github: `README.md`](https://github.com/doriantaylor/owl-process-model/blob/main/README.md). Task-to-task blocking is not established.</td>
    <td class="chosen">[`pm:Goal`](https://vocab.methodandstructure.com/process-model#Goal) extends IBIS issue</td>
    <td class="chosen">IBIS positions and candidate tasks [:fontawesome-brands-github: `README.md`](https://github.com/doriantaylor/owl-process-model/blob/main/README.md)</td>
    <td class="chosen"></td><td class="chosen">:white_check_mark: Chosen</td>
  </tr>
  <tr markdown="span">
    <th>[Design Intent Ontology](https://nimonika.github.io/ontologies/dio/doc/)</th>
    <td>[04-01-2016](https://nimonika.github.io/ontologies/dio/doc/) specification date as written; month and day ambiguous; last edit unknown</td>
    <td>[Design rationale, IBIS, argumentation, and PROV](https://nimonika.github.io/ontologies/dio/doc/)</td>
    <td></td><td></td><td></td>
    <td>[`dio:DesignIssue`](https://w3id.org/dio#DesignIssue); resulting [`dio:DesignDecision`](https://w3id.org/dio#DesignDecision)</td>
    <td>[`dio:AlternativeSolution`](https://w3id.org/dio#AlternativeSolution), via [`dio:hasAlternativeSolution`](https://w3id.org/dio#hasAlternativeSolution)</td>
    <td>Issue to solution via [`dio:hasMandatedSolution`](https://w3id.org/dio#hasMandatedSolution)</td><td></td>
  </tr>
  <tr markdown="span">
    <th>[`Inferal` Decision Ontology](https://ontology.inferal.com/modules/decision/current/spec/)</th>
    <td>[2026-10-21](https://ontology.inferal.com/modules/decision/current/spec/) unofficial draft; last edit unknown</td>
    <td>[UFO decision-making analysis, design rationale, and PROV](https://ontology.inferal.com/modules/decision/current/spec/); no optimization algorithm prescribed</td>
    <td></td><td></td><td></td>
    <td>[`decision:Issue`](https://ontology.inferal.com/modules/decision/Issue), [`decision:Deliberation`](https://ontology.inferal.com/modules/decision/Deliberation), and [`decision:Decision`](https://ontology.inferal.com/modules/decision/Decision)</td>
    <td>[`decision:Option`](https://ontology.inferal.com/modules/decision/Option), linked from deliberation</td>
    <td>Decision to option via [`decision:selectedOption`](https://ontology.inferal.com/modules/decision/selectedOption)</td><td></td>
  </tr>
</table>
