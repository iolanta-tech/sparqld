---
"@context": urn:sparqld:roadmap
$id: urn:sparqld:roadmap:configuration
$type: pm:Goal
title: Users can configure projects with sparqld.toml and command-line precedence
hide: [toc]
---

# Declare reproducible project settings

Implement a `sparqld.toml` project configuration file that contributors can check into
version control. Document how command-line arguments override project settings
so a shared project produces predictable behavior across machines.
Verify a configured setting and its override by a conflicting command-line
argument.

This outcome is a prerequisite for the [remote-data goal](remote.md), which
needs an explicit policy for approved hosts.
