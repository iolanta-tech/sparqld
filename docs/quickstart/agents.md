---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": quickstart/agents.md
"@type": schema:HowTo
icon: material/robot-outline
hide: [toc]
name: Starting with Agents
title: Starting with Agents
description: Set up sparqld through a coding agent and run a first SPARQL-backed query.
---

# :material-robot-outline: Starting with Agents

Install the `sparqld` skill for your coding agent:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g
```

Ask the agent to set up the knowledge base:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

Approve installing `sparqld` and registering the MCP server when it asks. Then
ask it about the knowledge base; it should answer from SPARQL and show the
queries it ran.

With example Alpha Centauri data configured, this session used its query tool to
find the recorded planets:

{{ agent_conversation('alpha-centauri') }}

See [Agents](../reference/agents/index.md) for each coding agent and the MCP
contract. Prefer [Starting in CLI](cli.md) when you want to drive SPARQL
yourself.
