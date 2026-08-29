#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
STATUS="${1:-$ROOT/docs/status.md}"
python3 - "$ROOT" "$STATUS" <<'PY'
from pathlib import Path
import subprocess,sys
root=Path(sys.argv[1]).resolve(); status=Path(sys.argv[2]).resolve()
refs=[]
for line in status.read_text(encoding='utf-8').splitlines():
    if not line.startswith('|') or line.startswith('|---') or line.startswith('| ID '): continue
    c=[x.strip() for x in line.strip().strip('|').split('|')]
    if len(c)==5: refs.append(c[3].strip('`'))
scoped={x for x in refs if x.startswith('docs/spec/') or x.startswith('docs/adr/')}
if len(scoped)!=len([x for x in refs if x.startswith('docs/spec/') or x.startswith('docs/adr/')]): raise SystemExit('duplicate scoped status evidence paths')
if not scoped: raise SystemExit('status table has no scoped docs/spec or docs/adr evidence')
tracked=set(subprocess.check_output(['git','-C',str(root),'ls-files','--','docs/spec/*.md','docs/adr/*.md'],text=True).splitlines())
missing=sorted(scoped-tracked)
if missing: raise SystemExit('status-referenced governance evidence is not Git-tracked: '+', '.join(missing))
for rel in sorted(scoped):
    raw=root/rel
    p=raw.resolve()
    try: p.relative_to(root)
    except ValueError: raise SystemExit('status evidence escapes repository: '+rel)
    if raw.is_symlink() or not p.is_file(): raise SystemExit('status evidence must be a non-symlink regular file: '+rel)
    # Resolve against the actual worktree, not an arbitrary alternate path.
    worktree=Path(subprocess.check_output(['git','-C',str(root),'rev-parse','--show-toplevel'],text=True).strip()).resolve()
    if p != (worktree/rel).resolve(): raise SystemExit('status evidence path is not in worktree: '+rel)
print(f'status scoped coverage validation passed: {len(scoped)} referenced docs/spec and docs/adr files')
PY
