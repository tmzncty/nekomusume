#!/usr/bin/env bash
# Shared fail-closed cargo-fuzz tooling contract for CI and local smoke runs.
CARGO_FUZZ_VERSION=0.13.2
CARGO_FUZZ_INSTALL="cargo install cargo-fuzz --version $CARGO_FUZZ_VERSION --locked"

require_cargo_fuzz_version() {
    if ! command -v cargo-fuzz >/dev/null 2>&1; then
        printf 'error: cargo-fuzz %s is required; install it with %s\n' \
            "$CARGO_FUZZ_VERSION" "$CARGO_FUZZ_INSTALL" >&2
        return 1
    fi

    cargo_fuzz_actual=$(cargo fuzz --version 2>/dev/null) || {
        printf 'error: cargo-fuzz %s is not executable\n' "$CARGO_FUZZ_VERSION" >&2
        return 1
    }
    cargo_fuzz_expected="cargo-fuzz $CARGO_FUZZ_VERSION"
    if [ "$cargo_fuzz_actual" != "$cargo_fuzz_expected" ]; then
        printf 'error: expected %s, found %s; install it with %s\n' \
            "$cargo_fuzz_expected" "$cargo_fuzz_actual" "$CARGO_FUZZ_INSTALL" >&2
        return 1
    fi
}
