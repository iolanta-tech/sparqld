---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/opencode.md
"@type": schema:TechArticle
name: OpenCode agents
title: OpenCode
description: Query a local sparqld knowledge base from OpenCode.
hide: [toc]
---

# :material-console: OpenCode

Install the skill for OpenCode:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g -a opencode
```

Ask OpenCode to set up the knowledge base:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

Approve installing `sparqld` and registering the MCP server when it asks. Use
the `query` tool once it appears.
