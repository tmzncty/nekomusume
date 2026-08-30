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
bash scripts/check-status-evidence.sh
bash scripts/check-status-evidence-test.sh
bash scripts/check-status-coverage.sh
bash scripts/check-status-coverage-test.sh
bash scripts/check-shell-syntax.sh
bash scripts/check-observability-contract.sh
bash scripts/check-observability-contract-test.sh
bash scripts/check-markdown-links.sh
bash scripts/check-markdown-links-test.sh
bash scripts/check-release-boundaries.sh
bash scripts/check-release-boundaries-test.sh
bash scripts/check-era3-capabilities.sh
bash scripts/check-era3-capabilities-test.sh
bash scripts/check-plan-sync.sh
bash scripts/check-plan-sync-test.sh
bash scripts/check-decision-index.sh
bash scripts/check-decision-index-test.sh
test -f docs/specs/nekomusume-session-v0.md
test -f LICENSE-MIT
test -f LICENSE-APACHE
cargo metadata --locked --format-version 1 --no-deps | grep -q 'MIT OR Apache-2.0'
