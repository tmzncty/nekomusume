# Independent item-4 review — I4-PORT-RES re-challenge + CLI readiness boundedness

**Reviewed anchor:** `20480d40582e07f0a792d6cdbd585a0817384b1b` (`main` at review start).

**Review class:** source/diff + specification/evidence-boundary review. No reviewer-local Rust/full-gate, cross-target, fuzz, WAN, or performance execution is claimed here.

## New developer commits reviewed

- `0ed814ce4adbc6de9b803904baa665629dc2bb15` — removes `#[cfg(unix)]` from the zero-sized `ReadyProof` / `BarrierProof` markers.
- `20480d40582e07f0a792d6cdbd585a0817384b1b` — closes H-I4-095 in the handoff and persists developer-reported exact-tree provenance.

H-I4-095 is accepted as closed for the stated source-shape defect. The markers contain no target-specific API, while `ReadyServer` stores `ReadyProof` unconditionally; making the proof marker target-neutral removes the deterministic non-Unix missing-type failure without weakening the Linux `/proc` causal gates. The persisted `0ed814c` gate remains **developer-reported local evidence only**; no Windows/macOS/BSD execution is inferred, and no hosted run was visible for that SHA during this review.

## I4-PORT-RES exact-current causal re-challenge — no finding

Inspected owner: `crates/neko-cli/tests/probe.rs` at exact anchor `20480d4`.

Challenged claims:

1. pre-churn Linux resource baseline must be causally after failover-server readiness and before the first malformed UDP send;
2. post-churn Linux resource sample must be causally after all bounded malformed inputs are server-classified after cleanup;
3. moving only the measurement, only the phase/proof check, or the whole measurement block across either edge should not silently preserve the claimed evidence.

Exact-current shape remains fail-closed for the ordinary mutations challenged:

- `ready_failover_server(...)` returns the `ReadyServer` that owns `ReadyProof`; the Linux baseline's `gated_resource_snapshot(...)` destructures that readiness-derived proof **inside the same helper call** that invokes `process_resource_snapshot(pid)` and also asserts `!churn_started`.
- `churn_started = true` occurs before the first malformed `send_to`.
- `malformed_classification_barrier(...)` returns `BarrierProof` only on the successful complete-classification path; timeout/disconnect/EOF return `Err` after child cleanup/join ownership handling.
- the Linux post sample consumes `BarrierProof` inside `gated_resource_snapshot(...)`; non-Linux Unix explicitly sinks the proof and does not claim `/proc` evidence.

No new concrete H-I4-090..095 causal/resource defect was found in this bounded re-challenge. This is a scoped source review, not a new runtime resource result.

## New READY_LOCAL finding — I4-CLI-PROC-096: readiness helper deadlines do not bound blocking `read_line`

This is **not HIGH/BLOCKER** and is not a runtime transport defect. It is an item-4 process-test boundedness/reliability defect with a committed, local answer.

Two exact-current readiness helpers still use blocking `BufRead::read_line` on the main test thread:

- `start_server_for(...)` computes a two-second `deadline` and checks it before each iteration, but the subsequent `stdout.read_line(&mut line)` has no read timeout and can block past that deadline if the child stays alive but emits no newline/EOF. The clock check therefore does not actually bound the wait.
- `ready_endpoint_rebind_server(...)` loops on blocking `read_line` with no deadline at all.

The same file already contains the bounded pattern in `ready_failover_server(...)`: move blocking stdout reading off-thread, use `recv_timeout`, and on timeout kill/reap the child so the reader gets EOF and ownership converges. That existing pattern fixes the correctness issue without changing CLI/runtime semantics.

### Closure contract

Use the smallest test-helper repair; do not create a generic process framework.

1. Make the affected readiness wait(s) genuinely bounded despite a silent-but-live child (off-thread reader + bounded channel wait or an equivalent interruptible mechanism).
2. On timeout, deterministically terminate/reap the owned child and ensure the reader cannot remain stranded.
3. Preserve startup-log accumulation and existing positive readiness strings/semantics.
4. Add a focused negative regression that demonstrates a silent/no-ready child returns/panics within the declared bound rather than hanging; cover EOF separately only if the helper's ownership path changes.
5. Do not touch Session/Carrier/ACK/crypto/wire semantics, D019, capacity/security values, release flags, or standing authorization. No fuzz is needed unless parser/framing owners change.
6. On the final pushed source/test SHA run focused process tests plus `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and a clean-tree check; persist exact reachable provenance.

## Evidence/governance boundary

- `READY_LIVE: none` remains authoritative; no repeated VPS/HY2/failover evidence is requested merely for freshness.
- Release items 3 and 4 remain incomplete.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; this review chooses no TTL/LRU/history-size/capacity value.

After I4-CLI-PROC-096 is repaired/provenanced, continue immediately into the pre-auth malformed/rejection resource-accounting bounded challenge and the remaining deep queue; do not wait for reviewer cadence.
