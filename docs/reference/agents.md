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
knowledge base. With the example dataset running, ask your agent:

{{ agent_conversation('alpha-centauri') }}
