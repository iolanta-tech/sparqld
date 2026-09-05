---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": project/contributing.md
"@type": schema:TechArticle
name: Contributing to sparqld
title: Contributing
description: Contributor workflows for sparqld.
---

# :material-source-pull: Contributing

Contributors work from a local checkout. Review the [Roadmap](roadmap.md) and
[architecture decisions](decisions/index.md) before proposing a change.

## :material-hammer-wrench: Test the Codex plugin locally

To test uncommitted plugin changes, add the checkout as a marketplace source
and install the plugin:

```sh
codex plugin marketplace add /path/to/sparqld
codex plugin add sparqld@sparqld-development
```
