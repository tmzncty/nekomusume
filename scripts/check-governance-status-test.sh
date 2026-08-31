#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/governance-status-test.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
cp -a "$ROOT/." "$TMP/"
run_checker() { (cd "$TMP" && bash scripts/check-governance-status.sh); }
mutate_workspace_evidence() {
  local replacement=$1
  REPLACEMENT="$replacement" awk '
    !done && index($0, "| workspace |") {
      marker = "`Cargo.toml`"
      start = index($0, marker)
      if (start) {
        print substr($0, 1, start - 1) ENVIRON["REPLACEMENT"] substr($0, start + length(marker))
        done = 1
        next
      }
    }
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
expect_rejection() {
  local name=$1 rc stderr_file
  stderr_file="$TMP/${name//[^[:alnum:]_.-]/_}.stderr"
  set +e
  run_checker >/dev/null 2>"$stderr_file"
  rc=$?
  set -e
  printf '%s mutation exit: %s\n' "$name" "$rc"
  [ "$rc" -ne 0 ] || { echo "mutation unexpectedly passed: $name"; exit 1; }
  if grep -iE '(^|[^[:alnum:]_])warning([^[:alnum:]_]|$)' "$stderr_file" >/dev/null; then
    echo "unexpected warning on rejected mutation: $name" >&2
    cat "$stderr_file" >&2
    exit 1
  fi
  printf '%s mutation stderr: no warnings\n' "$name"
}

run_checker >/dev/null

# The reviewer-owned handoff may quote forbidden escalation language while
# explicitly denying it; the checker must not require mutating that read-only file.
grep -F -- "--exclude='CHATGPT_HANDOFF.md'" "$ROOT/scripts/check-governance-status.sh" >/dev/null
printf '%s\n' 'reviewer handoff exclusion contract passed'

# Leading and trailing whitespace still resolves to the existing Cargo.toml file.
mutate_workspace_evidence "  \`Cargo.toml\`  "
run_checker >/dev/null
printf '%s\n' 'valid evidence whitespace mutation exit: 0'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Missing evidence is a distinct mutation and its checker exit code is captured.
mutate_workspace_evidence "\`does-not-exist.md\`"
expect_rejection 'missing evidence'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Evidence must be a regular file, not a directory.
mutate_workspace_evidence "\`docs\`"
expect_rejection 'directory evidence'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Absolute, traversal, and backslash paths are independently rejected.
mutate_workspace_evidence "\`/etc/passwd\`"
expect_rejection 'absolute evidence path'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"
mutate_workspace_evidence "\`../Cargo.toml\`"
expect_rejection 'traversal evidence path'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"
mutate_workspace_evidence "\`docs\\status.md\`"
expect_rejection 'backslash evidence path'
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"

# Workspace and CLI checkbox drift are independently rejected.
mutate_roadmap '建立 Cargo workspace / crate 边界' ' '
expect_rejection 'workspace checkbox drift'
cp "$ROOT/ROADMAP.md" "$TMP/ROADMAP.md"
mutate_roadmap 'CLI skeleton' ' '
expect_rejection 'CLI checkbox drift'

# Keep the file-only contract explicit in the regression test itself.
grep -F 'test -f' "$ROOT/scripts/check-governance-status.sh" >/dev/null
printf '%s\n' 'governance checker regression tests passed'

# N0 vector mutations must be rejected, and production authorization must remain
# explicitly independent from the RC prerequisite set.
for mutation in \
  'IMPLEMENTATION_COMPLETE=true|IMPLEMENTATION_COMPLETE=false' \
  'RELEASE_CANDIDATE=false|RELEASE_CANDIDATE=true' \
  'PRODUCTION_READY=false|PRODUCTION_READY=true' \
  'FREEZE=false|FREEZE=true' \
  'RELEASED=false|RELEASED=true'; do
  old=${mutation%%|*}; new=${mutation#*|}
  cp "$ROOT/docs/status.md" "$TMP/docs/status.md"
  sed -i "s/${old}/${new}/" "$TMP/docs/status.md"
  expect_rejection "N0 vector ${old}"
done
cp "$ROOT/docs/status.md" "$TMP/docs/status.md"
sed -i 's/`PRODUCTION_AUTHORIZATION` is \*\*not\*\* an RC prerequisite./`PRODUCTION_AUTHORIZATION` is an RC prerequisite./' "$TMP/docs/status.md"
expect_rejection 'N0 authorization coupling'
printf '%s\n' 'N0 governance vector regression tests passed'
