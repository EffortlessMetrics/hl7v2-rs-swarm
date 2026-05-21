# Rails framework in this repository

`.rails/` is the durable Rails knowledge base for this repository.

`docs/` explains Rails to humans and provides contribution guidance.

## Ownership boundaries

Rails owns long-term artifacts in `.rails/`:

- proposals
- specs
- ADRs
- lane trackers and implementation plans
- support and policy references
- closeouts

Rails does **not** own, migrate, or validate these external namespaces:

- `.codex/` (Codex execution state)
- `.spec/` (Spec Kit / speckit state)
- `.claude/` (external agent/session state)
- `.jules/` (external agent/session state)

## Artifact rules

- Every Rails artifact is linked through `.rails/index.toml`.
- Owned artifact paths must live under `.rails/`.
- External namespaces may be listed for awareness only and must not contain Rails-owned artifacts.
- Lane trackers are focused by lane; there is no single global mega queue.
