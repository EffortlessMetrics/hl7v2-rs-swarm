# HL7V2-SPEC-0001: Rails artifact graph contract

Status: accepted
Owner: core-maintainers
Created: 2026-05-21
Linked proposal: HL7V2-PROP-0001
Linked ADRs: HL7V2-ADR-0001
Linked lane: rails-adoption
Linked issues:
Linked PRs:
Support-tier impact: none
Policy impact: reference-only

## Problem

Without a contract, durable artifacts can drift across locations and lose traceability.

## Behavior

- Rails artifacts must be indexed through `.rails/index.toml`.
- Rails-owned artifacts must live under `.rails/`.
- External namespaces may be listed for awareness only and not as owned artifacts.
- Specs define behavior contracts, not PR order.
- Lane trackers define focused implementation sequencing.

## Non-goals

- Managing external agent/session state.

## Required evidence

- Structural checks pass via repository diff hygiene and index consistency reviews.

## Acceptance examples

- Artifact entries resolve to existing `.rails/` files.
- External namespace entries exist in metadata only.

## Test mapping

- `git diff --check`

## Implementation mapping

- `.rails/index.toml`
- `.rails/README.md`
- `.rails/lanes/*`

## CI proof

- `git diff --check`

## Metrics / promotion rule

Adoption is complete when all first-class Rails artifacts link through the index.

## Failure modes

- Owned artifact path outside `.rails/`.
- Missing linked artifact IDs.
