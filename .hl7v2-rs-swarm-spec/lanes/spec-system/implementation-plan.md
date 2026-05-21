# Repo-native spec system implementation plan

Status: active
Owner: architecture
Linked proposal: HLSWARM-PROP-0001
Linked specs: HLSWARM-SPEC-0001
Linked ADRs: HLSWARM-ADR-0001

## End state

Durable spec artifacts are repo-owned, linked, and separate from tool session state.

## Work items

### Work item: namespace-doctrine

Status: done
Linked proposal: HLSWARM-PROP-0001
Linked spec: HLSWARM-SPEC-0001
Linked ADR: HLSWARM-ADR-0001

#### Goal

Install and document the repo-native namespace doctrine.

#### Proof commands

```bash
git diff --check
```
