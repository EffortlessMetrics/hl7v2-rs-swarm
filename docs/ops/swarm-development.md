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

## JavaScript Action Runtime

The routed workflow opts into the GitHub Actions Node 24 JavaScript action
runtime with:

```text
FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true
```

This is scoped to `HL7v2 Rust Small` during burn-in so the future branch
protection gate exercises the newer action runtime before GitHub-hosted runners
make it the default. If an action compatibility issue appears, revert the
workflow-scoped opt-in while updating the affected action; do not set
`ACTIONS_ALLOW_USE_UNSECURE_NODE_VERSION` as a durable bypass.

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

The latest post-merge routed proofs after the Node 24 opt-in are:

- Push proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26128842531`
  passed on `main` at `070ee2effb0ca1cb0639656ecd66f10ff3294e48`.
- The route selected `router_target=github` with
  `router_reason=runner_token_missing`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed.
- `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- `CI`, `CI Policy`, and `Security` also passed on the same commit.
- Checkout v6 alignment proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26134306537`
  passed on `main` at `860d55d58c3fca2f9892bb25e5a5c9a9458891d6`.
- Source readiness receipt sync proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26134936717`
  passed on `main` at `1e2e9daf85993a1c98c84a2aab4accfca72b0d9c`.
- For both newer proofs, `CI`, `CI Policy`, and `Security` also passed on
  the same `main` commit.
- Nightly mutation output-directory guard sync proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26141263822`
  passed on `main` at `3e148421e4050041f15283238b98a477a66be9e9`.
- On the same commit, `CI`, `CI Policy`, `API Contracts`, `Coverage`, and
  `Security` also passed.
- Source parity audit sync proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26145221001`
  passed on `main` at `2e73bf92b14b480319dfef58dfd4208f698a83f4`.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, and `Security` also passed.

This proves the routed gate still works under the workflow-scoped Node 24
JavaScript action runtime opt-in. It still does not prove CX53 or CX43
execution.

## Source Sync Boundary

As of 2026-05-20, source-only TestPyPI OIDC, release-readiness receipt,
cross-surface parity audit, and Nightly mutation output-directory guard updates
have been synced into `hl7v2-rs-swarm`. The remaining source-vs-swarm tree delta
is intentional swarm infrastructure:

- `.github/workflows/ci-policy.yml`
- `.github/workflows/em-ci-routed-rust.yml`
- `docs/ci/ci-lane-whitelist.md`
- `docs/ops/swarm-development.md`
- swarm routed-lane entries in `policy/ci-lane-whitelist.toml`
- swarm workflow allowlist entry in `policy/workflow-allowlist.toml`
- `xtask` source-sync boundary command wiring
- the active-goal work item for the swarm cutover

Do not remove those deltas when syncing source changes into the swarm repo.

The source-sync boundary check is:

```bash
cargo run -p xtask -- check-source-sync-boundary --source-ref source/main --swarm-ref HEAD
```

It fails when any source-vs-swarm delta appears outside the intentional swarm
infrastructure allowlist. Run it after fetching both `origin` and `source`
before claiming the repositories are synchronized.
The `CI Policy` workflow runs the same check on non-PR events after fetching
`EffortlessMetrics/hl7v2-rs` as the `source` remote. It is intentionally not a
pull-request step because normal swarm development may create product deltas
before the source mirror is updated. `check-ci-lane-whitelist` guards that
workflow wiring so the source-sync step cannot be removed silently.

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

Live checks on 2026-05-20 still showed zero visible repository runners, no
visible repository secrets, and no `main` branch protection. The token used for
repo work could read repository secrets but could not list organization runner
groups; GitHub returned `403` with the `admin:org` scope requirement for the
runner-group API. Branch protection should remain deferred until CX53, CX43
fallback, and hosted fallback are all proven.

Until those are complete, routed workflow proof can exercise hosted fallback
behavior only. Do not claim CX53 or CX43 execution without run receipts.
