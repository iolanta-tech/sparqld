---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/index.md
"@type": schema:TechArticle
name: Agents
title: Agents
description: Let agents query a local sparqld knowledge base.
---

# :material-robot-outline: Agents

`sparqld` gives agents read-only SPARQL access to your knowledge base.

<div class="grid cards" markdown>

-   [:material-connection: **MCP server**](../index.md#mcp-server)

    Run `sparqld mcp <DIRECTORY>` to connect an MCP-capable agent.

-   [:fontawesome-brands-openai: **Codex**](codex.md)

    Configure the `sparqld` Codex plugin and its local MCP server.

</div>

Agents without MCP can use the endpoint URL with a SPARQL HTTP client, such as
[`sq`](../clients/sq/index.md).
