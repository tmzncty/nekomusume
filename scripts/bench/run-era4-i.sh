#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"
iterations=${NEKO_BENCH_ITERS:-1000}
case "$iterations" in (''|*[!0-9]*) echo 'NEKO_BENCH_ITERS must be an integer' >&2; exit 2;; esac
[ "$iterations" -ge 1 ] && [ "$iterations" -le 10000 ] || { echo 'NEKO_BENCH_ITERS must be between 1 and 10000' >&2; exit 2; }
NEKO_BENCH_ITERS="$iterations" cargo run -p neko-bench --release --quiet
