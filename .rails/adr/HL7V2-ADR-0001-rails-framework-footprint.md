# HL7V2-ADR-0001: Rails framework footprint under `.rails/`

Status: accepted
Date: 2026-05-21
Owner: core-maintainers
Linked proposal: HL7V2-PROP-0001
Linked specs: HL7V2-SPEC-0001

## Decision

Long-term proposal/spec/ADR/lane/support/policy/closeout rails live in `.rails/`.

## Context

A single portable framework footprint improves consistency across repositories and separates durable product knowledge from tool-specific state.

## Consequences

- Durable artifacts become discoverable and indexable.
- External namespaces stay awareness-only.

## Alternatives considered

- Repo-scoped framework directory names; rejected.

## Follow-up specs / plans

Maintain artifact linkage in `.rails/index.toml` and lane trackers under `.rails/lanes/`.
