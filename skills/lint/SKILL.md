---
name: lint
description: Run and resolve project lint findings in a focused, safe iteration. Use when asked to run `j lint`, investigate Ruff, Flake8/WPS, Cargo formatting, or Clippy failures, or fix a lint issue in this repository.
---

# Lint

Resolve one coherent lint problem at a time. Keep the working tree intact and
make the next failure easier to understand.

## Run and group the lint results

1. From the repository root, record `git status --short` and the current diff.
   Do not overwrite, revert, stage, or otherwise subsume existing work.
2. Run `j lint`. If the command is unavailable, run `.venv/bin/j lint`.
   This command runs `cargo fmt`, `cargo clippy`, `ruff format`, `ruff check`,
   and Flake8; formatting can modify files before a later check fails.
3. Compare the post-run status and diff with the snapshot. Retain formatter
   edits and report them separately from pre-existing changes.
4. Group failures by tool and diagnostic code, with the description, count, and
   affected files. For WPS, use the `WPS###` code as the initial group key;
   split it when its occurrences require materially different fixes.

When lint passes, report that result and any formatter-only changes; do not
invent another task.

## Choose and resolve a group

Select the group with the safest shared, low-risk fix. Use occurrence count
only to break ties. State the selected group, its locations, and the intended
approach, then implement every safe instance of that group in the current
working tree. Do not start a second group in the same invocation.

Inspect each affected call site before editing. Preserve behavior and existing
project conventions. Leave an occurrence unresolved when it does not share the
selected safe fix, explain why, and include it in the remaining findings.

Refactor source by default. When a narrowly scoped Flake8/WPS configuration
change is a viable alternative, present the code and configuration approaches,
with consequences and a recommendation, then wait for the user's choice before
changing configuration. Likewise, pause for a short options-based interview
when the fix has a meaningful API, behavior, or design tradeoff. Otherwise,
continue directly after the proposal.

## Verify and learn

1. Rerun `j lint` after the edits and summarize resolved and remaining groups.
2. Run relevant existing tests when the refactor changes behavior; run
   `cargo test --locked` when Rust code changed.
3. Review the final diff for accidental formatter or unrelated changes.

When a validated choice applies across the project—for example, the same
approved pattern is used in multiple occurrences—or the user calls it a
project practice, propose a concise addition to this skill. Update this file
only after explicit approval, and keep the rule specific enough to guide a
future lint fix without duplicating general project guidance.
