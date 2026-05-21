# HLSWARM-SPEC-0001: Repo-native spec rails contract

Status: accepted
Owner: architecture
Created: 2026-05-21
Linked proposal: HLSWARM-PROP-0001
Linked ADRs: HLSWARM-ADR-0001
Linked lane: spec-system
Support-tier impact: referenced by claim map
Policy impact: references live policy ledgers

## Behavior

- Durable rails are owned under `.hl7v2-rs-swarm-spec/`.
- `docs/` explains method and contributor usage.
- `policy/*.toml` stays live enforcement truth and is only referenced.
- Artifact ownership must not point into `.codex/`, `.spec/`, `.claude/`, or `.jules/`.
