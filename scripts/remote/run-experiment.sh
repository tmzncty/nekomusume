#!/usr/bin/env bash
# Bounded, local-only remote experiment runner. Research evidence only.
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 [--dry-run] [--experiment-id ID] [--artifact-root DIR]

Runs a bounded loopback experiment through explicit prepare/deploy/start/verify/
run/capture/stop/collect/cleanup/verify-clean phases. Dry-run is the default.
USAGE
}

DRY_RUN=1
EXPERIMENT_ID=""
ARTIFACT_ROOT="${TMPDIR:-/tmp}/nekomusume-experiments"
MAX_SECONDS=30
MAX_COUNT=8
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --execute) DRY_RUN=0 ;;
    --experiment-id=*) EXPERIMENT_ID=${arg#*=} ;;
    --artifact-root=*) ARTIFACT_ROOT=${arg#*=} ;;
    --help|-h) usage; exit 0 ;;
    *) printf 'unknown option: %s\n' "$arg" >&2; exit 2 ;;
  esac
done

if [[ -z "$EXPERIMENT_ID" ]]; then
  EXPERIMENT_ID="exp-$(date -u +%Y%m%dT%H%M%SZ)-$$-${RANDOM}"
fi
# IDs and roots are deliberately constrained: no shell/path injection or traversal.
[[ "$EXPERIMENT_ID" =~ ^exp-[A-Za-z0-9][A-Za-z0-9._-]{7,63}$ ]] || { echo 'invalid experiment id' >&2; exit 2; }
[[ "$ARTIFACT_ROOT" = /* ]] || { echo 'artifact root must be absolute' >&2; exit 2; }
[[ "$ARTIFACT_ROOT" != *$'\n'* && "$ARTIFACT_ROOT" != *$'\r'* ]] || { echo 'invalid artifact root' >&2; exit 2; }

ARTIFACT_DIR="$ARTIFACT_ROOT/$EXPERIMENT_ID"
LOG="$ARTIFACT_DIR/events.jsonl"
PIDFILE="$ARTIFACT_DIR/runner.pid"
phase() {
  local name=$1
  printf '{"experiment_id":"%s","phase":"%s","dry_run":%s,"ts":"%s"}\n' \
    "$EXPERIMENT_ID" "$name" "$([[ $DRY_RUN -eq 1 ]] && echo true || echo false)" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$LOG"
}
mkdir -p "$ARTIFACT_DIR"
chmod 700 "$ARTIFACT_DIR"
: > "$LOG"
phase prepare
cat > "$ARTIFACT_DIR/manifest.json" <<MANIFEST
{"experiment_id":"$EXPERIMENT_ID","mode":"$([[ $DRY_RUN -eq 1 ]] && echo dry-run || echo execute)","scope":"loopback-only","max_seconds":$MAX_SECONDS,"max_count":$MAX_COUNT,"secrets":false,"public_wan":false}
MANIFEST
chmod 600 "$ARTIFACT_DIR/manifest.json"

phase deploy
if (( DRY_RUN == 0 )); then
  command -v cargo >/dev/null || { echo 'cargo is required for --execute' >&2; exit 1; }
  cargo build --locked -p neko-cli --quiet
fi
phase start
if (( DRY_RUN == 0 )); then
  # The executable is intentionally bound to loopback and a fixed bounded port.
  port=$((40080 + (RANDOM % 20)))
  target/debug/neko server --transport tcp --bind "127.0.0.1:$port" --port "$port" \
    --duration "$MAX_SECONDS" --count "$MAX_COUNT" --json \
    --identity "$ARTIFACT_DIR/server.identity" >"$ARTIFACT_DIR/server.log" 2>&1 &
  server_pid=$!
  printf '%s\n' "$server_pid" > "$PIDFILE"
  chmod 600 "$PIDFILE"
else
  printf 'dry-run: would start authenticated TCP loopback server on 127.0.0.1:40080-40100\n' > "$ARTIFACT_DIR/plan.txt"
fi
phase verify
if (( DRY_RUN == 0 )); then
  kill -0 "$server_pid" 2>/dev/null || { echo 'server failed to start' >&2; exit 1; }
fi
phase run
if (( DRY_RUN == 0 )); then
  # No remote address is accepted; this is a local-only bounded probe.
  timeout "$MAX_SECONDS" target/debug/neko client --transport tcp --host 127.0.0.1 \
    --port "$port" --duration "$MAX_SECONDS" --count "$MAX_COUNT" --bytes 32 --json \
    --server-key "$(sed -n 's/^server_public_key=//p' "$ARTIFACT_DIR/server.log" | head -1)" \
    --identity "$ARTIFACT_DIR/client.identity" > "$ARTIFACT_DIR/client.log" 2>&1 || true
fi
phase capture
# Capture is metadata-only: never packet capture, payload, keys, or environment.
printf '{"experiment_id":"%s","capture":"metadata-only","payload":false,"keys":false,"addresses":"loopback-only"}\n' "$EXPERIMENT_ID" > "$ARTIFACT_DIR/capture.json"
phase stop
if [[ -f "$PIDFILE" ]]; then
  pid=$(cat "$PIDFILE")
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -f "$PIDFILE"
fi
phase collect
find "$ARTIFACT_DIR" -maxdepth 1 -type f -printf '%f\n' | sort > "$ARTIFACT_DIR/files.txt"
phase cleanup
rm -f "$ARTIFACT_DIR"/*.identity "$ARTIFACT_DIR/runner.pid"
phase verify-clean
if compgen -G "$ARTIFACT_DIR/*.identity" >/dev/null || [[ -e "$PIDFILE" ]]; then
  echo 'cleanup verification failed' >&2; exit 1
fi
printf 'experiment_id=%s mode=%s artifact_dir=%s\n' "$EXPERIMENT_ID" "$([[ $DRY_RUN -eq 1 ]] && echo dry-run || echo execute)" "$ARTIFACT_DIR"
