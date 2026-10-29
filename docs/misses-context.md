---
"@context":
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
  rdfs: http://www.w3.org/2000/01/rdf-schema#
  rlog: http://persistence.uni-leipzig.org/nlp2rdf/ontologies/rlog#
  title: rdfs:label
  description: rdfs:comment
  domain:
    "@id": rdfs:domain
    "@type": "@id"
"@id": https://sparqld.iolanta.tech/misses-context
"@type": rdf:Property
title: misses context
description: The subject error entry records a context-resolution failure for the object URI.
domain: rlog:Entry
---

# :material-link-off: misses-context

| Definition | Value |
| --- | --- |
| IRI | [`https://sparqld.iolanta.tech/misses-context`](https://sparqld.iolanta.tech/misses-context) |
| Type | [`rdf:Property`](http://www.w3.org/1999/02/22-rdf-syntax-ns#Property) |
| Domain | [`rlog:Entry`](http://persistence.uni-leipzig.org/nlp2rdf/ontologies/rlog#Entry) |
| Object | The unresolved context URI, as an RDF IRI. |

`sparqld` adds this property to a [file-catalog error entry](reference/file-catalog.md)
when its loader cannot resolve a remote context URI. The error entry retains
its source-file link, severity, and readable message.
Each unresolved URI observed during the failed loading attempt gets its own
property value. The message retains the parser's primary diagnostic.

Local file-access failures and malformed context bodies alone do not add this
property. After a successful reload, deletion, or replacement by another
error, the old property values are removed.

## :material-code-braces-box: Example source

This YAML-LD source requests a context that `sparqld` cannot resolve. Its
catalog error links to `https://example.org/context` through
[`sparqld:misses-context`](https://sparqld.iolanta.tech/misses-context).

{{ example_data('missing-context.yamlld') }}

## :material-database-search-outline: Find affected files

Query the reserved `sparqld:` catalog graph for the property and the source
link. The query does not depend on the error message's wording.

{{ source('docs/queries/missing-contexts.rq') }}

{{ stored_sparql('docs/queries/missing-contexts.rq') | sparql_table }}
