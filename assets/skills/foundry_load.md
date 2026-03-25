---
name: foundry:load
description: Load Foundry project/spec context using JSON inventory and status, then read `.foundry` or `~/.foundry` files.
---

# foundry:load

Use when you need project or spec context from the central Foundry store.

## Guardrails

- Prefer **machine-readable output** first so you know paths and state before reading large files.
- If the user’s intent is ambiguous (multiple projects/specs), run inventory JSON and ask which to load.

## Steps

1. **Store overview (optional but recommended)**

   ```bash
   foundry --json status
   ```

   Use `cwd_link_project` when present to infer the default project for a linked repo.

2. **Project list**

   ```bash
   foundry --json list projects
   ```

3. **Active specs for a project**

   ```bash
   foundry --json list specs <project>
   ```

4. **Structured status for one spec** (paths + phase + task counts + instruction)

   ```bash
   foundry --json spec status <project> <spec-id-or-partial>
   ```

   Read the `context_files` paths from JSON, then open those files (or read under `.foundry/` when linked).

5. **Project-level markdown** (when you need vision / tech stack / summary)

   If the repo is linked, read `.foundry/vision.md` (and siblings) directly. Otherwise:

   ```bash
   foundry project load <project>
   ```

6. Pull the relevant excerpts into your working context before changing code or specs.
