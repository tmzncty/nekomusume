#!/usr/bin/env bash
# Fail closed unless tracked and non-ignored source state exactly matches HEAD.
set -eu
ROOT=${1:-$(git rev-parse --show-toplevel)}
cd "$ROOT"
git diff --quiet -- || { echo "release build requires no unstaged tracked changes" >&2; exit 1; }
git diff --cached --quiet -- || { echo "release build requires no staged changes" >&2; exit 1; }
if [ -n "$(git ls-files --others --exclude-standard)" ]; then
  echo "release build requires no non-ignored untracked files" >&2
  exit 1
fi
