#!/usr/bin/env bash
set -euo pipefail

bad=0
workflow_dir=".github/workflows"

if [ ! -d "$workflow_dir" ]; then
  echo "No GitHub workflow directory found; skipping bare self-hosted check."
  exit 0
fi

echo "Checking for bare self-hosted runner usage..."

if rg -n -i 'runs-on:[[:space:]]*\[[^]]*self-hosted[^]]*linux[^]]*x64[^]]*\]' "$workflow_dir"; then
  echo "Bare inline self-hosted/linux/x64 runs-on is forbidden; use an explicit runner group and capacity labels." >&2
  bad=1
fi

while IFS=: read -r file line _; do
  window="$(sed -n "${line},$((line+16))p" "$file")"

  if printf '%s\n' "$window" | rg -q -i '^[[:space:]]*-[[:space:]]*linux[[:space:]]*$' &&
     printf '%s\n' "$window" | rg -q -i '^[[:space:]]*-[[:space:]]*x64[[:space:]]*$' &&
     ! printf '%s\n' "$window" | rg -q 'group:[[:space:]]*em-ci-' &&
     ! printf '%s\n' "$window" | rg -q '^[[:space:]]*-[[:space:]]*(em-ci|ci-nano|policy-nano|workflow-nano|rust-tiny|rust-medium|rust-large|rust-16gb|cx23|cx33|cx43|cx53|cpx42)[[:space:]]*$'; then
    echo "$file:$line: bare self-hosted block lacks group/capacity labels" >&2
    bad=1
  fi
done < <(rg -n '^[[:space:]]*-[[:space:]]*self-hosted[[:space:]]*$' "$workflow_dir" || true)

exit "$bad"
