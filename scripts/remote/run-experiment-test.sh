#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
runner="$ROOT/scripts/remote/run-experiment.sh"
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
id=exp-test-20260829
result=$("$runner" --dry-run --experiment-id "$id" --artifact-root "$out")
dir="$out/$id"
test -d "$dir"
test "$(grep -c '"phase"' "$dir/events.jsonl")" -eq 10
for phase in prepare deploy start verify run capture stop collect cleanup verify-clean; do grep -q "\"phase\":\"$phase\"" "$dir/events.jsonl"; done
grep -q '"secrets":false' "$dir/manifest.json"
grep -q '"public_wan":false' "$dir/manifest.json"
test ! -e "$dir/runner.pid"
! "$runner" --dry-run --experiment-id '../escape' --artifact-root "$out" >/dev/null 2>&1
! "$runner" --dry-run --experiment-id "$id" --artifact-root relative >/dev/null 2>&1
printf 'remote runner tests: ok\n'
