---
name: block-worktree-commits
enabled: true
event: bash
pattern: git\s+commit
action: warn
---

⚠️ **git commit detected — are you sure you should be committing here?**

If you are a coding agent running inside a Kanban task worktree: do NOT commit. Leave your changes uncommitted so the repository owner can review the diff and commit or open a PR through their normal workflow.

If you are the repository owner working in the main workspace: you may proceed, but prefer using the Cline commit workflow so changes are reviewed before they land.
