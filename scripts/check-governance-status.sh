#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
status=docs/status.md
test -s "$status"
awk -F'|' '
  /^\| [^ -]/ {
    id=$2; state=$4; evidence=$5
    gsub(/^ +| +$/, "", id); gsub(/^ +| +$/, "", state); gsub(/^ +| +$/, "", evidence)
    if (id != "ID" && state !~ /^(implemented|candidate|provisional|absent|blocked)$/) { print "invalid status: " id ": " state; bad=1 }
    if (id != "ID" && evidence !~ /^`[^`]+`/) { print "missing evidence: " id; bad=1 }
    count[id]++
  }
  END { if (count["G0"] != 1) { print "expected exactly one G0 row"; bad=1 } if (bad) exit 1 }
' "$status"
while IFS= read -r path; do
  case "$path" in \`*\`) path=${path#\`}; path=${path%\`};; *) continue;; esac
  test -e "$path" || { echo "missing status evidence: $path"; exit 1; }
done < <(awk -F'|' '/^\| [^ -]/ && $2 != " ID " {print $5}' "$status")
# Governance guard: these claims must remain absent from repository implementation/status prose.
if grep -RIn --exclude-dir=.git --exclude-dir=target --exclude='status.md' -E 'production[- ]ready|security audit passed|publicly deployable|implemented (protocol|security|tunnel)|protocol (is )?frozen' README.md ROADMAP.md IMPLEMENTATION_PLAN.md docs; then
  echo 'forbidden governance escalation claim found'; exit 1
fi
