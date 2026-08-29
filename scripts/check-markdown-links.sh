#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 - "$ROOT" "$@" <<'PY'
import pathlib,re,sys
root=pathlib.Path(sys.argv[1]).resolve()
files=[pathlib.Path(x) for x in sys.argv[2:]] or [root/'README.md',root/'SECURITY.md',root/'AGENTS.md',root/'docs/status.md',root/'docs/carrier-architecture.md',root/'docs/learning-path.md']
pat=re.compile(r'\[[^\]]*\]\(([^)]+)\)')
count=0
for f in files:
 f=f if f.is_absolute() else root/f
 text=f.read_text(encoding='utf-8')
 for raw in pat.findall(text):
  target=raw.split('#',1)[0]
  if not target or '://' in target or target.startswith('mailto:'): continue
  if '`' in target or '\\' in target or target.startswith('/') or re.match(r'^[A-Za-z]:',target): raise SystemExit(f'unsafe markdown link: {f}: {raw}')
  dest=(f.parent/target).resolve()
  # Normal checked documents must stay in-repository. Explicit test fixtures
  # may live in a temporary directory and are allowed to resolve within that directory.
  boundary=root if f.is_relative_to(root) else f.parent
  try: dest.relative_to(boundary)
  except ValueError: raise SystemExit(f'markdown link escapes boundary: {f}: {raw}')
  if not dest.exists(): raise SystemExit(f'missing markdown link: {f}: {raw}')
  count+=1
print(f'markdown link validation passed: {count} local links')
PY
