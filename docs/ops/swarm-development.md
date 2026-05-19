# Swarm Development Surface

`EffortlessMetrics/hl7v2-rs-swarm` is the swarm development surface for
`EffortlessMetrics/hl7v2-rs`.

## Repository Roles

- `EffortlessMetrics/hl7v2-rs-swarm` is where new development PRs should be
  opened after the routed CI cutover is proven.
- `EffortlessMetrics/hl7v2-rs` remains the source and release mirror until that
  role is deliberately changed.
- Release, publish, signing, crates.io, PyPI, and TestPyPI secrets remain on the
  source repository unless a later release operation explicitly moves them.
- Existing local clones should not be retargeted in place. Clone
  `hl7v2-rs-swarm` side-by-side.

## Branch Rules

- Do not push directly to `main`.
- Use PRs against `main` for swarm development.
- Branch protection is deferred until routed CI is proven.
- Once enabled, the intended required check is the normalized result check:
  `HL7v2 Rust Small Result`.
- Do not require conditional implementation jobs directly in branch protection.

## Initial Routed CI Target

The first routed gate is intentionally narrow:

```text
HL7v2 Rust Small:
  CX53 -> CX43 -> GitHub-hosted
```

The Rust command should match the current source-repo PR gate semantics:

```bash
cargo run -p xtask -- gate --check
```

CX33 is not part of the default Rust gate during burn-in. It can be considered
later for docs-only or small policy checks after timing and disk receipts exist.

## Current Proof

As of 2026-05-19, the routed workflow is installed and the GitHub-hosted
fallback route is proven:

- Push proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26125581115`
  passed on `main` at `190ee0aac3e71de19533adaf5e68d8f7158b997b`.
- Manual dispatch proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26126337302`
  passed on `main` at `190ee0aac3e71de19533adaf5e68d8f7158b997b`.
- The manual dispatch route selected `router_target=github` with
  `router_reason=runner_api_failed` because `EM_RUNNER_READ_TOKEN` is not
  configured in this repository yet.
- In that manual dispatch run, `Rust Small on GitHub Hosted` passed,
  `Rust Small on CX53` and `Rust Small on CX43` were skipped, and
  `HL7v2 Rust Small Result` passed.

This proves the normalized result check and hosted fallback behavior. It does
not prove CX53 or CX43 execution.

Later routed workflow runs distinguish missing token setup from API failures:

- `router_reason=runner_token_missing` means `EM_RUNNER_READ_TOKEN` is not
  configured for this repository.
- `router_reason=runner_api_failed` means the token was present but the GitHub
  runner API did not return a usable `200` response.
- `router_reason=no_idle_runner` means the runner API was readable but no
  eligible CX53 or CX43 runner was online and idle.

## Self-hosted Guardrails

- Run self-hosted jobs only for same-repository trusted PRs.
- Do not run fork PR code on self-hosted runners.
- Use the Rust 1.95 runner image for self-hosted Linux Rust work.
- Use disk guards before self-hosted cargo work.
- Use container-safe cleanup after self-hosted cargo work.
- Keep server smoke, Python wheels, coverage, mutation, nightly, publish, and
  release workflows non-required during initial burn-in.

## Current Admin Boundary

The swarm repository has been created and seeded from the source history. The
following admin steps are required before self-hosted proof can complete:

- Add `hl7v2-rs-swarm` to the `em-ci-small` runner group selected repositories.
- Scope `EM_RUNNER_READ_TOKEN` to `hl7v2-rs-swarm`.

Until those are complete, routed workflow proof can exercise hosted fallback
behavior only. Do not claim CX53 or CX43 execution without run receipts.
