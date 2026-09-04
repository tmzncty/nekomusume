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
python3 scripts/check-era4-closure.py
python3 scripts/check-evidence-manifests.py
bash scripts/check-status-evidence.sh
bash scripts/check-status-evidence-test.sh
bash scripts/check-status-coverage.sh
bash scripts/check-status-coverage-test.sh
bash scripts/check-shell-syntax.sh
python3 scripts/bench/process-resource-sampler-test.py
python3 scripts/bench/echo-payload-test.py
bash scripts/bench/compare-hy2-test.sh
bash scripts/check-observability-contract.sh
bash scripts/check-observability-contract-test.sh
bash scripts/check-markdown-links.sh
bash scripts/check-markdown-links-test.sh
bash scripts/check-release-boundaries.sh
bash scripts/check-release-boundaries-test.sh
bash scripts/check-era4-protocol-release.sh
bash scripts/check-era4-protocol-release-test.sh
bash scripts/check-era3-capabilities.sh
bash scripts/check-era3-capabilities-test.sh
bash scripts/check-plan-sync.sh
bash scripts/check-plan-sync-test.sh
bash scripts/check-decision-index.sh
bash scripts/check-decision-index-test.sh
python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json
python3 scripts/validate-canonical-vectors-test.py
python3 scripts/generate-canonical-review.py --check
python3 scripts/generate-canonical-review-test.py
test -f docs/specs/nekomusume-session-v0.md
test -f LICENSE-MIT
test -f LICENSE-APACHE
cargo metadata --locked --format-version 1 --no-deps | grep -q 'MIT OR Apache-2.0'

bash scripts/bench/compare-hy2-owned-lab-test.sh
python3 scripts/bench/parse-listener-test.py
bash scripts/bench/owned-lab-control-plane-test.sh
python3 scripts/bench/validate-hy2-owned-lab-test.py
python3 scripts/bench/run-live-warm-failover-cycle-test.py
python3 scripts/bench/run-repeated-warm-failover-test.py
python3 scripts/bench/run-repeated-warm-failover-command-test.py
python3 scripts/bench/remote-endpoint-exec-test.py
python3 scripts/bench/run-periodic-command-test.py
