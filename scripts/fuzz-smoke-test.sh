#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
before=$(git status --porcelain=v1 --untracked-files=all)
FUZZ_TIME=${FUZZ_TEST_TIME:-1} ./scripts/fuzz-smoke.sh >/dev/null

after=$(git status --porcelain=v1 --untracked-files=all)
if [ "$before" != "$after" ]; then
    printf '%s\n' 'fuzz smoke changed repository status' >&2
    diff -u <(printf '%s\n' "$before") <(printf '%s\n' "$after") >&2 || true
    exit 1
fi
printf '%s\n' 'fuzz smoke isolation regression passed'
