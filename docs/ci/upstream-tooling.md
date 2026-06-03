# Upstream Tooling Substrate

This repository exposes repository policy through `cargo xtask ...` commands and treats
third-party tools as the engine room behind those stable command names. CI and local
operator instructions should prefer the `xtask` surface so the repo contract remains
stable when an upstream tool, flag, or routing rule changes.

## Repo-facing control surface

| Repo command | Upstream substrate | Role |
| --- | --- | --- |
| `cargo xtask check-pr` | repo policy, Cargo, rustfmt, Clippy | Default PR verification wrapper. |
| `cargo xtask fix-pr` | rustfmt, Clippy | Local formatting and common lint cleanup. |
| `cargo xtask pr-summary` | repo policy report | Review-facing policy summary. |
| `cargo xtask allow-check` | `cargo-allow` | Source exception receipt validation. |
| `cargo xtask allow-diff` | `cargo-allow` | Source exception receipt diffing. |
| `cargo xtask ripr-pr` | `ripr` | Static mutation-exposure PR evidence packet. |
| `cargo xtask unsafe-review-pr` | `unsafe-review` | Unsafe-contract review evidence card. |
| `cargo xtask test-pr` | `cargo-nextest` | Default Rust PR test execution. |
| `cargo xtask test-docs` | `cargo test --doc` | Doctest lane kept separate from nextest. |
| `cargo xtask coverage` | `cargo-llvm-cov`, `cargo-nextest` | Coverage artifact generation. |
| `cargo xtask mutation-targeted` | `cargo-mutants` | Runtime mutation backstop for selected risk. |
| `cargo xtask miri-targeted` | Miri on nightly | Concrete UB witness lane for selected risk. |
| `cargo xtask check-deps` | `cargo-deny` | Dependency policy gate. |
| `cargo xtask check-supply-chain` | `cargo-deny`, `cargo-audit` | Dependency policy plus advisory audit. |
| `cargo xtask semver-check` | `cargo-semver-checks` | Public API compatibility gate. |
| `cargo xtask check-workflows` | `actionlint`, `zizmor` | GitHub Actions correctness and security linting. |
| `cargo xtask check-toml` | `taplo` | TOML formatting and linting. |
| `cargo xtask policy-report` | repo ledgers | Policy rollout and debt summary. |

## Doctrine

- `xtask` is the public control surface for the repository.
- Upstream tools are standardized substrates, not user-facing policy endpoints.
- `ast-grep` finds syntactic candidates; Rust-aware tooling remains authoritative
  where identity, dependency graph, or public API semantics matter.
- `cargo_metadata` remains the basic workspace inventory substrate; richer graph
  planning can be added through `guppy` when CI lane selection needs dependency
  closures or feature graph routing.
- `ripr` shifts mutation signal left with static mutation-exposure evidence;
  `cargo-mutants` remains the runtime backstop for targeted, nightly, or release
  lanes.
- `unsafe-review` makes unsafe changes reviewable; Miri provides concrete runtime
  UB witnesses where risk warrants it.
- Coverage is execution-surface evidence, not a correctness or release-readiness
  claim by itself.
