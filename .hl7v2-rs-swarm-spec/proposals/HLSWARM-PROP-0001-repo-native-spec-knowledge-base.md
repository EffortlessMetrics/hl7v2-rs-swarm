# HLSWARM-PROP-0001: Repo-native spec knowledge base

Status: accepted
Owner: architecture
Created: 2026-05-21
Target milestone: spec-system bootstrap
Linked specs: HLSWARM-SPEC-0001
Linked ADRs: HLSWARM-ADR-0001
Linked lanes: spec-system

## Problem

Durable planning and specification truth can drift when captured in tool/session namespaces.

## Proposed shape

Own the full proposal->spec->ADR->lane->proof->closeout chain under `.hl7v2-rs-swarm-spec/`.

## Non-goals

Do not migrate or mutate `.spec/`, `.codex/`, `.claude/`, or `.jules/`.
