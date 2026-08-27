#!/usr/bin/env bash
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
fi

cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings

# Keep repository policy checks in one place for local and CI parity.
bash scripts/check-governance-status.sh
test -f docs/specs/nekomusume-session-v0.md
test -f LICENSE-MIT
test -f LICENSE-APACHE
cargo metadata --locked --format-version 1 --no-deps | grep -q 'MIT OR Apache-2.0'
