#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
status=docs/status.md
roadmap=ROADMAP.md
test -s "$status"
test -s "$roadmap"

while IFS=$'\t' read -r id state evidence; do
  [ -n "$id" ] || continue
  case "$evidence" in
    \`*\`) path=${evidence#\`}; path=${path%\`};;
    *) echo "missing evidence: $id"; exit 1;;
  esac
  case "$path" in
    ""|/*|*..*|*\\*) echo "unsafe or missing status evidence path: $id: $path"; exit 1;;
  esac
  test -f "$path" || { echo "missing status evidence file: $path"; exit 1; }
done < <(awk -F'|' '
  function trim(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
  /^\|/ {
    id=trim($2); state=trim($4); evidence=trim($5)
    if (id == "" || id == "ID" || id ~ /^-+$/) next
    if (state !~ /^(implemented|candidate|provisional|absent|blocked)$/) { print "invalid status: " id ": " state > "/dev/stderr"; bad=1 }
    if (evidence !~ /^`[^`]+`$/) { print "missing evidence: " id > "/dev/stderr"; bad=1 }
    print id "\t" state "\t" evidence
    count[id]++
  }
  END { if (count["G0"] != 1) { print "expected exactly one G0 row" > "/dev/stderr"; bad=1 } if (bad) exit 1 }
' "$status")

# ROADMAP checkboxes are descriptive; docs/status.md remains the status source.
check_checkbox() {
  local id=$1 expected=$2 label=$3 actual
  actual=$(awk -v label="$label" '$0 ~ "^- \\[.\\].*" label { print substr($0,4,1); found=1 } END { if (!found) exit 2 }' "$roadmap") || { echo "missing roadmap checkbox: $label"; return 1; }
  [ "$actual" = "$expected" ] || { echo "roadmap/status mismatch: $id ($label) is [$actual], status requires [$expected]"; return 1; }
}
check_checkbox workspace x '建立 Cargo workspace / crate 边界'
check_checkbox cli x 'CLI skeleton'
check_checkbox wire x '明确协议版本与 magic'
check_checkbox wire x '固定第一版 session record / UDP packet header'
check_checkbox session x '定义 Session delivery state / acknowledgement 语义'
check_checkbox fuzz x 'fuzz：畸形输入不得 panic、越界或无限分配'

if grep -RIn --exclude-dir=.git --exclude-dir=target --exclude='status.md' -E 'production[- ]ready|security audit passed|publicly deployable|implemented (protocol|security|tunnel)|protocol (is )?frozen' README.md ROADMAP.md IMPLEMENTATION_PLAN.md docs; then
  echo 'forbidden governance escalation claim found'; exit 1
fi

# Administrator authorization permits only bounded research implementation.
test -f docs/adr/m1-g0-research-authorization.md
grep -q 'external security audit' docs/adr/m1-g0-research-authorization.md
grep -q 'public, non-loopback listeners' docs/adr/m1-g0-research-authorization.md
grep -q 'research-authorized / not-security-approved' docs/adr/m1-g0-research-authorization.md
