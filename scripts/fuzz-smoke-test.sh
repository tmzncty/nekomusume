#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
# shellcheck source=scripts/fuzz-toolchain.sh
. "$ROOT/scripts/fuzz-toolchain.sh"

# CI and local smoke must consume the same exact pin rather than carrying copies.
grep -Fq '. scripts/fuzz-toolchain.sh' .github/workflows/ci.yml
grep -Fq 'cargo install cargo-fuzz --version "$CARGO_FUZZ_VERSION" --locked' .github/workflows/ci.yml
grep -Fq '. "$ROOT/scripts/fuzz-toolchain.sh"' scripts/fuzz-smoke.sh
if grep -R -n -E 'cargo-fuzz --version [0-9]+\.[0-9]+\.[0-9]+' \
    .github/workflows/ci.yml scripts/fuzz-smoke.sh >/dev/null; then
    printf '%s\n' 'cargo-fuzz version duplicated outside shared tooling contract' >&2
    exit 1
fi

# A stale locally installed tool must fail before any target build or run.
fake_bin=$(mktemp -d "${TMPDIR:-/tmp}/nekomusume-fuzz-contract.XXXXXX")
cleanup_contract() {
    rm -rf -- "$fake_bin"
}
trap cleanup_contract EXIT INT TERM
cat > "$fake_bin/cargo-fuzz" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' 'cargo-fuzz 0.12.0'
FAKE
chmod +x "$fake_bin/cargo-fuzz"
if PATH="$fake_bin:$PATH" require_cargo_fuzz_version 2>"$fake_bin/error"; then
    printf '%s\n' 'stale cargo-fuzz unexpectedly satisfied version contract' >&2
    exit 1
fi
grep -Fq "expected cargo-fuzz $CARGO_FUZZ_VERSION, found cargo-fuzz 0.12.0" "$fake_bin/error"
rm -rf -- "$fake_bin"
trap - EXIT INT TERM

require_cargo_fuzz_version
before=$(git status --porcelain=v1 --untracked-files=all)
FUZZ_TIME=${FUZZ_TEST_TIME:-1} ./scripts/fuzz-smoke.sh >/dev/null

after=$(git status --porcelain=v1 --untracked-files=all)
if [ "$before" != "$after" ]; then
    printf '%s\n' 'fuzz smoke changed repository status' >&2
    diff -u <(printf '%s\n' "$before") <(printf '%s\n' "$after") >&2 || true
    exit 1
fi
printf 'cargo-fuzz %s contract and fuzz smoke isolation regression passed\n' "$CARGO_FUZZ_VERSION"
