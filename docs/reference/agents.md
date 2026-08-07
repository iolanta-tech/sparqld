---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents.md
"@type": schema:TechArticle
name: Agents
title: Agents
description: Let agents query a local sparqld knowledge base.
---

# :material-robot-outline: Agents

`sparqld` gives agents a read-only query surface over the files in your
knowledge base. Give the agent the endpoint URL and a way to send SPARQL HTTP
requests, such as the [`sq` client](clients/sq/index.md). With the example
dataset running, ask your agent:

{{ agent_conversation('alpha-centauri') }}

## :material-refresh: Live updates

`sparqld` reloads a changed source after the editor's filesystem events settle.
If parsing fails, it removes that source's graph and records an `rlog:Entry`
with the error in the file catalog. Give the agent a chance to inspect the
error before asking it to query the changed data again.
