---
name: foundry:done
description: Checkpoint Foundry in git and/or collapse a finished spec into a completed-work summary plus archived files.
---

# foundry:done

Use after a meaningful chunk of work, or when a spec is **fully complete** and should become history.

## Guardrails

- **Git sync** commits whatever changed under `~/.foundry/`; use an accurate message.
- **Collapse** is irreversible as an *active* spec: the CLI moves working files into `completed/<spec-id>/archive/`. Only run when the user agrees the spec is finished.
- The **summary narrative is agent-authored**. Foundry creates `summary.md` (empty) on `collapse prepare`; you fill it before `finalize`.

## Steps

### A. Ongoing checkpoint (git backup)

1. Summarize what changed (completed tasks, decisions, follow-ups).
2. If `foundry git init` was run for `~/.foundry/`:

   ```bash
   foundry git sync "<short accurate message>"
   ```

3. If git backup is not enabled, skip—files still live under `~/.foundry/`.

### B. Finish a spec (completed-work record)

1. Confirm all tasks are checked:

   ```bash
   foundry spec status <project> <spec-id-or-partial>
   ```

   Expect derived phase / state indicating **completed_pending_collapse** before collapsing.

2. Verify the implementation matches the spec:

   ```bash
   foundry spec instructions verify <project> <spec-id-or-partial>
   ```

   Read every file listed in `context_files`. For each task in `tasks`,
   confirm the work is actually present in the codebase as described.
   If you find gaps or drift, fix them or flag to the user before collapsing.

3. Get collapse guidance (JSON only):

   ```bash
   foundry spec instructions collapse <project> <spec-id-or-partial>
   ```

   Follow `instruction` and `needs_prepare`.

4. Create the completed directory scaffold:

   ```bash
   foundry spec collapse prepare <project> <spec-id-or-partial>
   ```

5. **Write** `summary.md` at the path implied by `~/.foundry/<project>/completed/<spec-id>/summary.md` (or use `summary_path` from `spec collapse prepare` JSON). Include what shipped, scope, and key decisions.

6. Finalize (moves `spec.md`, `task-list.md`, `notes.md`, `meta.json` into `archive/`):

   ```bash
   foundry spec collapse finalize <project> <spec-id-or-partial> --confirm
   ```

7. Optionally run **A** again to commit the collapsed layout.
