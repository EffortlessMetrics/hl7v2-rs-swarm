# hl7v2-rs-swarm repo-native spec namespace

This namespace is the durable source of truth for long-lived planning and specification artifacts.

## Scope

`./.hl7v2-rs-swarm-spec/` owns:

- roadmap direction
- proposals (PRD-style)
- behavior specs
- architecture decisions (ADRs)
- lane trackers and implementation plans
- support-claim mappings
- policy ledger references
- closeouts

## External namespaces (awareness-only)

The following directories may exist, but are **not** owned by this system:

- `.codex/`
- `.spec/`
- `.claude/`
- `.jules/`

Agents can read this namespace, but durable rails must live here.
