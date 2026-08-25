#!/usr/bin/env bash
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
fi

FUZZ_TIME=${FUZZ_TIME:-30}
FUZZ_MAX_LEN=${FUZZ_MAX_LEN:-8192}

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    printf '%s\n' 'error: cargo-fuzz is required; install it with cargo install cargo-fuzz --version 0.12.0 --locked' >&2
    exit 1
fi
if ! rustup run nightly rustc --version >/dev/null 2>&1; then
    printf '%s\n' 'error: the nightly Rust toolchain is required; install it with rustup toolchain install nightly --profile minimal' >&2
    exit 1
fi
if ! cargo fuzz --version >/dev/null 2>&1; then
    printf '%s\n' 'error: cargo-fuzz is not executable' >&2
    exit 1
fi

printf 'running nightly cargo-fuzz decode smoke: time=%s max_len=%s\n' "$FUZZ_TIME" "$FUZZ_MAX_LEN"
RUSTUP_TOOLCHAIN=nightly cargo fuzz build decode
RUSTUP_TOOLCHAIN=nightly cargo fuzz run decode -- -max_total_time="$FUZZ_TIME" -max_len="$FUZZ_MAX_LEN"
