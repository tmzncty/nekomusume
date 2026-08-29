#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SCRIPT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
status=0
while IFS= read -r -d '' script; do
    if ! bash -n "$script"; then
        printf 'shell syntax check failed: %s\n' "${script#"$ROOT"/}" >&2
        status=1
    fi
done < <(find "$SCRIPT_ROOT" -type f -name '*.sh' -print0 | sort -z)
exit "$status"
