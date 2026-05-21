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
- After branch protection is enabled, verify the live GitHub state with
  `cargo run -p xtask -- check-swarm-branch-protection`. Before it is enabled,
  `cargo run -p xtask -- check-swarm-branch-protection --allow-unprotected`
  records the expected blocked state without claiming completion.
- After runner-group and token authorization setup, verify organization runner
  discovery plus CPX42/CX43/CX53 runner setup with
  `cargo run -p xtask -- check-swarm-runner-setup`. If the exact
  `EM_RUNNER_READ_TOKEN` value is available locally, export it before running
  that command so the checker exercises the same token as the router. Before
  setup is complete, `cargo run -p xtask -- check-swarm-runner-setup
  --allow-unavailable` records the expected blocked state without claiming
  CPX42/CX43/CX53 proof.

## Initial Routed CI Target

The first routed gate is intentionally narrow:

```text
HL7v2 Rust Small:
  CPX42 -> CX43 -> CX53 -> GitHub-hosted
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
- `router_reason=runner_token_unauthorized` means the token was present but the
  GitHub organization runner API returned HTTP `401`.
- `router_reason=runner_token_forbidden` means the token was present but the
  GitHub organization runner API returned HTTP `403`; check token scopes, SSO,
  organization runner access, and runner-group visibility.
- `router_reason=runner_api_failed` means the token was present but the GitHub
  organization runner API did not return a usable response for a non-401/non-403
  failure.
- `router_reason=no_idle_runner` means the runner API was readable but no
  eligible CPX42, CX43, or CX53 runner was online and idle.

Historical runs before the router split HTTP `401`/`403` outcomes recorded
those failures as `runner_api_failed`.

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
- Branch-protection verifier proof: PR #24, `ci: add swarm branch protection
  verifier`, merged on 2026-05-20 at
  `52d0abad93842355caebe9afeccb2c10811d7002`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26151749719`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `API Contracts`, `Coverage`, and
  `Security` also passed.
- Evidence parity support-map audit guard sync proof: PR #30, `ci: guard
  evidence parity support map audit`, merged on 2026-05-20 at
  `ef4922fe9e6aeffbd08a30cea4c043d92399e0eb`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26167964447`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `API Contracts`, `Coverage`, and
  `Security` also passed.
- A local pre-protection verifier refresh passed and confirmed branch
  protection is still intentionally deferred:
  `cargo +1.95.0 run -p xtask -- check-swarm-branch-protection --allow-unprotected`.
- Source RIPR calibration sync proof: PR #32, `sync: merge source ripr
  calibration`, merged on 2026-05-20 at
  `61d85c756311cc8614f893db04550ab0f363a128`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26171472072`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, and `Security` also passed.
- Runner setup verifier guard proof: PR #34, `ci: require runner token in
  swarm setup check`, merged on 2026-05-20 at
  `9b8a7e162470dc0bafcd1e684e1cc0793e027a59`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26174342959`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `API Contracts`, `Coverage`, and
  `Security` also passed.
- Current-manual dispatch proof: `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26175197837`
  passed on `main` at `9b8a7e162470dc0bafcd1e684e1cc0793e027a59` via
  GitHub-hosted fallback.
- That manual route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- Source profile inheritance cycle sync proof: PR #36, `sync: bring profile
  inheritance cycle fix to swarm`, merged on 2026-05-20 at
  `6b1a99eaaaa307fc1171ec1cfe1f4d148b7e9217`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26179802862`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_api_failed`; the route annotation recorded
  `runner API returned HTTP 403; using GitHub-hosted fallback`.
- The workflow environment contained a masked `EM_RUNNER_READ_TOKEN`, so this
  run proves token presence in workflow context but not usable runner API
  access.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `Coverage`, `Python Wheels`,
  `Security`, and `Server Docker Smoke` also passed.
- Runner token fallback split proof: PR #38, `ci: split runner token fallback
  reasons`, merged on 2026-05-20 at
  `217cc76a588ebf643d5e349ad01d08bf35221b76`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26181954381`
  passed on `main` via GitHub-hosted fallback.
- That route selected `router_target=github` with
  `router_reason=runner_token_forbidden`. The result job logged
  `ROUTER_REPO=EffortlessMetrics/hl7v2-rs-swarm`,
  `ROUTER_WORKFLOW=em-ci-routed-rust`, and
  `ROUTER_RUN_ID=26181954381`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `API Contracts`, `Coverage`, and
  `Security` also passed.

This proves the routed gate still works under the workflow-scoped Node 24
JavaScript action runtime opt-in. It still does not prove CX53 or CX43
execution.

Post-queue cleanup and sync proof:

- Source PR #855, `devex: remove legacy cargo-make entrypoint`, merged on
  2026-05-20 and was synced to swarm in PR #41 at
  `0b98c48f896461c0e292ea66f13f94af203cbeb9`.
- Source PR #856, `docs: add source-of-truth stack guide`, merged on
  2026-05-20 and was synced to swarm in PR #40 at
  `028d70647f0c51bebb41cffac804b0e74b43a028`.
- Source PR #857, `test: cover lifecycle retention edge cases`, merged on
  2026-05-20 and was synced to swarm in PR #42 at
  `c947ba1422cb9abc1267f5335a015476acb85001`.
- Swarm-only refactor PRs #43, #44, #45, and #46 were closed rather than
  merged because they were source-affecting product/refactor changes without a
  source-first merge or explicit swarm-only exception.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26189447394`
  passed on `main` at `c947ba1422cb9abc1267f5335a015476acb85001`.
- The result job logged `router_target=github`,
  `router_reason=runner_token_forbidden`,
  `ROUTER_REPO=EffortlessMetrics/hl7v2-rs-swarm`,
  `ROUTER_WORKFLOW=em-ci-routed-rust`, and `ROUTER_RUN_ID=26189447394`.
- `Rust Small on GitHub Hosted` and `HL7v2 Rust Small Result` passed while
  `Rust Small on CX53` and `Rust Small on CX43` were skipped.
- On the same commit, `CI`, `CI Policy`, `Python Wheels`, `Security`, and
  `Server Docker Smoke` also passed.
- A local sync-boundary refresh after fetching `source/main` passed with nine
  intentional swarm-only deltas:
  `cargo +1.95.0 run -p xtask -- check-source-sync-boundary --source-ref source/main --swarm-ref HEAD`.
- Local blocked-state verifiers still showed the admin boundary: no locally
  visible `EM_RUNNER_READ_TOKEN`, zero visible repository runners, no online
  CX53/CX43 labels, and branch protection intentionally deferred.

## Source Sync Boundary

As of 2026-05-20, source-only TestPyPI OIDC, release-readiness receipt,
cross-surface parity audit, Nightly mutation output-directory guard, evidence
parity support-map audit guard, cargo-make cleanup, source-of-truth guide, and
lifecycle retention coverage updates have been synced into `hl7v2-rs-swarm`.
The remaining source-vs-swarm tree delta is intentional swarm infrastructure:

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

Guard proof:

- PR #22, `ci: guard source sync policy wiring`, merged on 2026-05-20 at
  `10a2fc1dfa44e79645606d6b6ddf70cbfd584775`.
- Post-merge `CI Policy` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26149295355`
  passed on `main` and ran both `Fetch source repository main` and
  `Check source/swarm sync boundary`.
- Post-merge `HL7v2 Rust Small` run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26149295445`
  passed via GitHub-hosted fallback with the annotation
  `runner API returned HTTP 403; using GitHub-hosted fallback`. `Rust Small on
  CX53` and `Rust Small on CX43` were skipped.
- A local boundary refresh after fetching `source/main` passed with nine
  intentional swarm-only deltas:
  `cargo +1.95.0 run -p xtask -- check-source-sync-boundary --source-ref source/main --swarm-ref HEAD`.

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
routed workflow now discovers the organization runner fleet, matching the
working swarm-router pattern. The following proof steps are required before
branch protection can be enabled:

- Confirm `EM_RUNNER_READ_TOKEN` can read
  `orgs/EffortlessMetrics/actions/runners`.
- Prove CPX42 primary routing. Done in PR #73 run
  `https://github.com/EffortlessMetrics/hl7v2-rs-swarm/actions/runs/26225834323`
  with `router_target=cpx42`, `router_reason=cpx42_idle`, `Rust Small on
  CPX42` success, CX43/CX53/GitHub-hosted skipped, and `HL7v2 Rust Small
  Result` success.
- Prove CX43 fallback routing.
- Prove CX53 fallback routing.
- Re-prove GitHub-hosted fallback after organization discovery is live.
- The `workflow_dispatch` route has manual proof inputs for fallback receipts:
  `skip_cpx42`, `skip_cx43`, and `skip_cx53`. These inputs only apply to
  manual dispatches; normal push and pull-request routing still follows
  CPX42 -> CX43 -> CX53 -> GitHub-hosted.
- Verify `EM_RUNNER_READ_TOKEN` and runner visibility with:

  ```bash
  cargo +1.95.0 run -p xtask -- check-swarm-runner-setup
  ```

  Until setup is complete, use the same checker only as a blocked-state
  receipt:

  ```bash
  cargo +1.95.0 run -p xtask -- check-swarm-runner-setup --allow-unavailable
  ```

The latest post-#71 routed proof on 2026-05-21 showed the workflow receives a
masked `EM_RUNNER_READ_TOKEN`, then receives HTTP `403` from the repository
runner-list API. That ruled out missing workflow-secret injection and pointed at
the repository endpoint as the wrong discovery surface for organization runner
groups. The router and local verifier now use the organization runner API,
matching the working swarm pattern.

PR #73 then proved the organization discovery path could select and schedule a
CPX42 runner, but the first implementation attempt failed before the Rust gate
because the selected CPX42 runner did not have the local Docker image
`em-ci-rust:1.95`. The CPX42 implementation path now uses the pinned Rust
toolchain action directly on the self-hosted runner, with scratch directories
created before the toolchain action because it honors `TMPDIR`.

PR #74 added manual `workflow_dispatch` skip inputs for fallback proof runs. The
first CPX42-skip proof selected CX53, proving CX53 discovery and scheduling,
but then failed because the CX53 local image also lacked `cargo`/`rustc` on
`PATH`. CX43 and CX53 now use the same pinned Rust 1.95 direct-toolchain pattern
as CPX42 while retaining their own runner labels, build-job caps, scratch
directories, disk guards, and cleanup.

The blocked runner setup verifier now treats local `EM_RUNNER_READ_TOKEN` as an
optional exact-token check. If the environment variable is not set locally, it
uses the current `gh` identity only as an advisory runner-list check and does
not claim anything about the Actions secret. Current blocked-state evidence
still does not print a success claim without exact token visibility and run
receipts. Branch protection should remain deferred until CX43 fallback, CX53
fallback, and hosted fallback are all proven.

Exact-token discovery check:

```bash
GH_TOKEN="$EM_RUNNER_READ_TOKEN" gh api \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "orgs/EffortlessMetrics/actions/runners?per_page=100"
```

Expected completion state is HTTP `200` plus visible CPX42/CX43/CX53 runners
with the labels guarded by `check-ci-lane-whitelist`. HTTP `403` means the
token/SSO/organization runner authorization is still insufficient for runner
discovery.

Until those are complete, branch protection stays deferred. Do not claim CX43 or
CX53 execution without run receipts.
