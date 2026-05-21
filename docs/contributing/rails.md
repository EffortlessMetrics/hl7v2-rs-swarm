# Contributing to Rails artifacts

When adding or updating Rails artifacts:

1. Use `.rails/` as the durable source-of-truth location.
2. Keep responsibilities separated by artifact type (proposal/spec/ADR/lane/support/policy/closeout).
3. Add or update links in `.rails/index.toml`.
4. Keep lane trackers focused; avoid creating a global all-lanes queue.
5. Never place Rails-owned artifacts under `.codex/`, `.spec/`, `.claude/`, or `.jules/`.

## Suggested proof command

Run:

```bash
git diff --check
```
