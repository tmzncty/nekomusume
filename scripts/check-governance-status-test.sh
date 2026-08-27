#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/governance-status-test.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
cp -a "$ROOT/." "$TMP/"
run_checker() { (cd "$TMP" && bash scripts/check-governance-status.sh); }
mutate_workspace_evidence() {
  local replacement=$1
  awk -v replacement="$replacement" '
    !done && index($0, "| workspace |") { sub(/`Cargo\.toml`/, replacement); done=1 }
    { print }
  ' "$TMP/docs/status.md" > "$TMP/docs/status.md.new"
  mv "$TMP/docs/status.md.new" "$TMP/docs/status.md"
}
mutate_roadmap() {
  local label=$1 mark=$2
  awk -v label="$label" -v mark="$mark" '
    !done && index($0, label) { sub(/^- \[[x ]\]/, "- [" mark "]"); done=1 }
    { print }
  ' "$TMP/ROADMAP.md" > "$TMP/ROADMAP.md.new"
  mv "$TMP/ROADMAP.md.new" "$TMP/ROADMAP.md"
}
run_checker >/dev/null

# Leading/trailing whitespace is trimmed and still resolves to the existing Cargo.toml file.
mutate_workspace_evidence "  \`Cargo.toml\`  "
run_checker >/dev/null
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Missing evidence must fail; preserve and assert the checker exit code.
mutate_workspace_evidence "\`does-not-exist.md\`"
set +e
run_checker >/dev/null 2>&1
rc=$?
set -e
printf 'missing evidence mutation exit: %s\n' "$rc"
[ "$rc" -ne 0 ] || { echo 'mutation unexpectedly passed: missing evidence'; exit 1; }
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# A directory is not valid file evidence; test -f must reject it.
mutate_workspace_evidence "\`docs\`"
set +e
run_checker >/dev/null 2>&1
rc=$?
set -e
printf 'directory evidence mutation exit: %s\n' "$rc"
[ "$rc" -ne 0 ] || { echo 'mutation unexpectedly passed: directory evidence'; exit 1; }
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Workspace and CLI checkbox drift are independently rejected.
mutate_roadmap '建立 Cargo workspace / crate 边界' ' '
set +e
run_checker >/dev/null 2>&1
rc=$?
set -e
printf 'workspace checkbox mutation exit: %s\n' "$rc"
[ "$rc" -ne 0 ] || { echo 'mutation unexpectedly passed: workspace drift'; exit 1; }
cp "$ROOT/ROADMAP.md" "$TMP/ROADMAP.md"
mutate_roadmap 'CLI skeleton' ' '
set +e
run_checker >/dev/null 2>&1
rc=$?
set -e
printf 'CLI checkbox mutation exit: %s\n' "$rc"
[ "$rc" -ne 0 ] || { echo 'mutation unexpectedly passed: CLI drift'; exit 1; }

echo 'governance checker regression tests passed'
