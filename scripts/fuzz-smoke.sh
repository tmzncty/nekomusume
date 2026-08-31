#!/usr/bin/env bash
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
fi
# shellcheck source=scripts/fuzz-toolchain.sh
. "$ROOT/scripts/fuzz-toolchain.sh"

FUZZ_TIME=${FUZZ_TIME:-30}
FUZZ_MAX_LEN=${FUZZ_MAX_LEN:-8192}
SEED_CORPUS="$ROOT/fuzz/corpus/decode"

require_cargo_fuzz_version
if ! rustup run nightly rustc --version >/dev/null 2>&1; then
    printf '%s\n' 'error: the nightly Rust toolchain is required; install it with rustup toolchain install nightly --profile minimal' >&2
    exit 1
fi
if [ ! -d "$SEED_CORPUS" ]; then
    printf 'error: seed corpus not found: %s\n' "$SEED_CORPUS" >&2
    exit 1
fi

WORK=$(mktemp -d "${TMPDIR:-/tmp}/nekomusume-fuzz.XXXXXX")
cleanup() {
    rm -rf -- "$WORK"
}
trap cleanup EXIT INT TERM
mkdir -p "$WORK/corpus" "$WORK/artifacts"
cp -a "$SEED_CORPUS/." "$WORK/corpus/"

printf 'running isolated nightly cargo-fuzz %s decode smoke: time=%s max_len=%s\n' \
    "$CARGO_FUZZ_VERSION" "$FUZZ_TIME" "$FUZZ_MAX_LEN"
RUSTUP_TOOLCHAIN=nightly cargo fuzz build decode
RUSTUP_TOOLCHAIN=nightly cargo fuzz run decode -- \
    -artifact_prefix="$WORK/artifacts/" \
    -max_total_time="$FUZZ_TIME" \
    -max_len="$FUZZ_MAX_LEN" \
    "$WORK/corpus"
