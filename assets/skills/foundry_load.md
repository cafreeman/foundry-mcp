---
name: foundry:load
description: Load Foundry project/spec context from ~/.foundry (via the .foundry symlink when linked).
---

# foundry:load

Use this when you need project or spec context from Foundry.

## Steps

1. If the repo uses Foundry linking, read files under `.foundry/` directly (for example `.foundry/vision.md`).
2. Otherwise, run:

```bash
foundry project load <project>
```

3. To load a specific spec (supports partial IDs when unambiguous):

```bash
foundry spec load <project> <spec-id>
```

4. Paste relevant excerpts into your working context before making changes.
