---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/agents/claude-code.md
"@type": schema:TechArticle
name: Claude Code agents
title: Claude Code
description: Query a local sparqld knowledge base from Claude Code.
hide: [toc]
---

# :simple-claude: Claude Code

Install the skill for Claude Code:

```sh
npx skills add iolanta-tech/sparqld --skill sparqld -g -a claude-code
```

Ask Claude Code to set up the knowledge base:

```text
/sparqld set up a queryable knowledge base at my documents folder
```

Approve installing `sparqld` and registering the MCP server when it asks. Use
the `query` tool once it appears.
