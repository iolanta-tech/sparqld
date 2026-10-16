---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/mcp.md
"@type": schema:TechArticle
name: MCP server
title: MCP server
description: MCP tools, prompts, and structured SPARQL query responses.
---

# :simple-modelcontextprotocol: MCP server

Configure your MCP client to launch `sparqld mcp <DIRECTORY>` over stdio.
See the [CLI reference](../index.md#mcp-server) for directory and pattern options,
or [Codex](codex.md) for plugin setup.

## :material-database-search: Query tool

`query` accepts one required string argument, `query`, containing a read-only
SPARQL query. The default graph is the union of the loaded source graphs.
Queries wait for initial loading; subsequent queries reflect watched file changes.

Every successful tool result contains a JSON object in MCP's `structuredContent`
field. The same object is serialized as JSON in a text block in `content` for
clients that consume text results. The examples below show the value of
`structuredContent`.

### SELECT

Follows [SPARQL Results JSON](https://www.w3.org/TR/sparql11-results-json/):
`head.vars` lists the projected variables, and `results.bindings` contains one
object per row. Unbound variables are omitted from that row.

{{ source('docs/results/mcp-select.json', title='SELECT result') }}

### ASK

Uses the SPARQL Results JSON `head` and `boolean` members.

{{ source('docs/results/mcp-ask.json', title='ASK result') }}

### CONSTRUCT and DESCRIBE

Return a `sparqld`-specific `triples` array. Each triple has `subject`, `predicate`,
and `object` RDF terms encoded as described below. These query forms return a
single graph, so triples have no graph-name field. Triple order is unspecified;
an empty graph has an empty `triples` array.

{{ source('docs/results/mcp-graph.json', title='Graph result') }}

### RDF terms

Both bindings and triples use the term representation from
[SPARQL Results JSON](https://www.w3.org/TR/sparql11-results-json/#select-encode-terms).
Each term has a string `type` and string `value`:

- `uri`: `value` is the complete IRI.
- `bnode`: `value` is a blank-node identifier without the `_:` prefix. Its
  identity is scoped to this result; identifiers cannot be compared across calls.
- `literal`: `value` is the lexical text. Language-tagged literals include
  `xml:lang`; other typed literals include the datatype IRI in `datatype`.
  Plain string literals omit `datatype`.

A triple's subject is an IRI or blank node, its predicate is an IRI, and its
object can be any of these term kinds.

## :material-alert-circle-outline: Errors and limits

Tool errors set `isError` to `true` and explain the failure in `content` text.
This includes invalid queries, SPARQL Update, evaluation failures, and fatal
initial dataset failures. Queries larger than 1 MiB in UTF-8 are rejected.
Change data by editing the source files.

## :material-text-box-search-outline: Investigation prompt

The `investigate` prompt takes a required `question` string. It guides the agent
to discover the vocabulary, run focused queries, and report evidence with the
SPARQL used. Prompt availability in the interface depends on the MCP client.
