# HL7V2-PROP-0001: Rails durable knowledge base adoption

Status: accepted
Owner: core-maintainers
Created: 2026-05-21
Target milestone: next minor release
Linked specs: HL7V2-SPEC-0001
Linked ADRs: HL7V2-ADR-0001
Linked lanes: rails-adoption

## Problem

The repository needs a durable, tool-independent source-of-truth structure for product and engineering decisions.

## Users and surfaces

Maintainers, contributors, and release operators across docs, CI, and roadmap surfaces.

## Success criteria

A standardized `.rails/` framework exists with index-linked proposal/spec/ADR/lane artifacts and documented ownership boundaries.

## Proposed shape

Adopt `.rails/` as the durable framework footprint and keep agent or tool namespaces awareness-only.

## Alternatives considered

- Repo-specific directory naming (`.<repo>-spec/`) rejected due to portability and branding inconsistency.

## Specs to create or update

- HL7V2-SPEC-0001

## Architecture decisions needed

- HL7V2-ADR-0001

## Implementation campaign shape

Establish footprint, templates, first artifact chain, and initial lane tracker.

## Evidence plan

- `git diff --check`

## Risks

Drift if artifacts are not consistently indexed.

## Non-goals

Migrating `.codex/`, `.spec/`, `.claude/`, or `.jules/` content.

## Exit criteria

Rails artifact graph is present, indexed, and documented.
