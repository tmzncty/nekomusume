# Independent bounded resource/abuse-control review — exact `f184cf2`

**Reviewed revision:** `f184cf21cd7b51c02d2b6e0f7850600f1be28784` (`origin/main` at review time).
**Later docs-only descendant:** `2ffde7444f4705adb0b82b07a57fee75b261ba5f` adds only packet/item-4 index rows; `git diff --stat f184cf2..2ffde74 -- ':(exclude)docs' ':(exclude)*.md'` is empty, so all code citations below hold at both revisions.
**Review type:** independent bounded audit of **non-policy** pre-auth resource/abuse controls. **Date:** 2026-09-11 (UTC `2026-09-10T21:14:22Z`).
**Scope:** `SECURITY.md`, `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/adr/m1-g0-preauth-resource-budget.md`, `PreauthBudget`, `ProcessPreauthAdmission`, `ListenerAdmission`/permits, the responder inventory, and focused tests. **Not** a security audit, cryptanalysis, adversarial-load assessment, penetration test, D019 policy decision, or production approval. Review host and checkout path intentionally sanitized; `rustc 1.98.0`.

## Verdict

**BLOCKER: none. HIGH defect: none found** in the audited non-policy controls. `RSEC-001` (adversarial-load suitability / promotion), `D019` (`SOURCE_RETENTION_POLICY_BLOCKED`) and item-4 closure remain **open and unmodified** by this review.

Every advertised responder path charges before its protected work, bounds amplification/state/queue/memory/work, settles permits explicitly, and releases/terminalizes.

## Method / reproduction

- `git rev-parse HEAD origin/main` -> both `f184cf2...`; tree clean; later `2ffde74` is a docs-only descendant (verified above).
- `python3 scripts/check-preauth-responder-inventory.py` -> `preauth responder inventory passed: 9 surfaces, 7 admission sites`.
- `python3 scripts/check-evidence-manifests.py` -> `29 files across 1 manifest(s)`.
- `cargo test -p neko-crypto preauth` -> `24 passed; 0 failed` (includes `default_candidate_state_queue_and_response_boundaries_reject_first_excess`, `rejected_global_window_state_cannot_revive_after_rollover`, `queue_promotion_and_cancellation_release_exactly_once`, `response_permit_is_controller_bound_and_one_pending_per_state`, `response_requires_charged_input_and_respects_amplification`).
- `cargo test -p neko-cli` -> `39 passed; 0 failed` (includes `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`).

## Answers to the seven audited questions

### 1. Charge-before-work — all 9 inventoried surfaces (verified by exact callsite)

Charge precedes parse/alloc/send on every path:

| Surface | charge-before-work anchors |
|---|---|
| `ordinary_tcp_probe` | `main.rs:566-575` (staged charge) -> `:577-579` parse; `:580-582` charge -> `:583-585` send |
| `ordinary_udp_probe` | `:659-661` charge -> `:665-667` parse; `:668-670` charge -> `:671-673` send; `:683-685` charge -> `:686-694` parse |
| `failover_udp_pending` | `:1124-1126` charge -> `:1127` compare -> `:1128-1133` charge -> `:1134-1141` send; `:1145-1153` parse -> `:1154-1156` charge -> `:1157-1166` send |
| `failover_udp_new` | `:1192-1195` charge -> `:1199` parse; `:1207-1214` charge -> `:1215-1224` send |
| `failover_udp_cached` | `:1251-1262` charge -> `:1263-1264` compare -> `:1268-1273` charge+send -> `:1285` AEAD open |
| `failover_tcp` | `:1371-1381` staged charge -> `:1384-1386` parse; `:1402-1412` staged charge -> `:1413-1422` parse |
| `endpoint_rebind_udp` | `:2824-2826` charge -> `:2829-2831` parse; `:2845-2847` charge -> `:2848-2852` parse |
| `periodic_tcp` | `periodic.rs:173-183` staged -> `:184-186` parse; `:195-198` staged -> `:199-203` parse |
| `multistream_tcp` | `multistream.rs:75-85` staged -> `:86-88` parse; `:106-116` staged -> `:117-119` parse |

Staged framing charges and bounds **before** allocating: `framed.rs:100` charges the header, `:102-107` rejects `len > max_frame_len`, `:108-111` charges the declared body, and only then `:112` allocates `vec![0; len]`; the `?` on the Body stage prevents allocation on rejection. The inventory checker enforces this ordering mechanically (`scripts/check-preauth-responder-inventory.py:63-67`).

### 2. Bounded amplification / state / queue / memory / work

- Per-state: `PreauthBudget` (`neko-crypto/src/lib.rs:818-950`) enforces per-state bytes/packets (`:826-909`) and the ADR anti-amplification `min(3*input_bytes, max_response_bytes)` / `min(input_packets, max_response_packets)` (`:919-927`).
- Process-wide: `ProcessPreauthAdmission` (`:1072-1730`) bounds states per source/global and memory (`:1201-1240`), per-source and per-window input bytes/packets/work (`:1256-1310`, `:1435-1483`), per-packet work ceiling (`:1264-1266`, `:1338-1345`), queue per source/global (`:1497-1515`), response bytes/packets per source/window (`:1545-1615`), and idle/lifetime (`:1184-1199`).
- All arithmetic uses `checked_add`/`checked_sub`/`saturating_sub`; overflow is treated as exhaustion.
- **ADR numeric conformance verified:** every one of the 22 `ProcessPreauthLimits::default()` values (`:976-1002`) equals the corresponding ADR candidate ceiling (8/1024 states, 64 KiB/64 packets per source, 8 MiB/8192 per window, 4096/131072/1048576 work, 16 KiB/16 MiB memory, 4/256 queue, 2 KiB/4 response per source, 256 KiB/512 per window, 1000/5000/100 ms).

### 3. Permit settlement

`PreauthResponsePermit` is `#[must_use]`, controller-bound and one-pending-per-state (`:1020-1028`, `:1553-1555`); it must be completed, suppressed, or abandoned (`:1619-1674`). Queue ownership settles via `enqueue`/`dequeue` (`:1497-1530`) and `ListenerAdmission::enqueue/dequeue` (`preauth.rs:497-510`). Staged input ownership is one-shot (`:1312-1419`). Response deadlines are inclusive and capped by idle/lifetime (`:1592-1597`, `:1638-1643`); `ListenerAdmission::effective_response_deadline` intersects the permit deadline with any earlier socket timeout (`preauth.rs:380-392`).

### 4. Cleanup and terminalization

`reject_state` marks terminal idempotently (`:1158-1167`); `release` removes state and its source accounting exactly once (`:1676-1690`); `expire_states` releases idle/lifetime-expired states atomically (`:1692-1711`) and the pending-owner path invalidates its queue reservation (`preauth.rs:53-57`, `main.rs:1087-1096`). Failure paths abandon charged input permits (`preauth.rs:33-41`, `:195-197`). No path emits `Delivery`/`PathValidated`/`ACK` evidence on exhaustion.

### 5. Inventory integrity

`docs/preauth-responder-inventory.v1.json` names 9 responders across 4 files and 7 admission sites; the checker verifies identifier completeness, per-responder ordered anchors **and** scoped pending-lifecycle anchors, mutation regressions (a stray anchor outside the intended region must not satisfy ownership), and that the admission-site count is exactly 7 (`scripts/check-preauth-responder-inventory.py:47-142`). This is an executable anti-drift control, not a static claim.

### 6. Tests

24 `neko-crypto` preauth tests and 39 `neko-cli` process tests pass, including atomic boundary-first-excess, window-rollover non-revival, queue promotion/cancel exact-once, permit controller-binding, amplification, and the 8-distinct-source malformed-UDP recovery. The ordinary-probe exit-on-malformed behavior is explicitly asserted at `crates/neko-cli/tests/probe.rs:2807-2812` (non-success status + no response bytes).

### 7. `SECURITY.md` red lines (non-policy subset)

"UDP amplification strictly limited", "per-connection and global memory/CPU/rate ceilings", "no unauthenticated control frame mutates connection state", and "no default open proxy" are each backed by the controls above plus fixed authenticated echo/session workloads. Secret-safe diagnostics are aggregate-only (`ciphertext_bytes`, counts); no addresses or payloads are logged in the audited paths.

## Bounded observations (non-defect; for the record)

- **[nit] Process-exit vs in-process recovery is path-dependent.** On `ordinary_*_probe`, `failover_tcp`, `endpoint_rebind_udp`, `periodic_tcp` and `multistream_tcp`, a pre-auth admission/parse rejection calls `fail()` -> `process::exit(2)` (`main.rs:255-258`); only the failover UDP new/pending paths `continue` in-process. Wire silence holds on every path, but "UDP malformed first packets remain silent" (`resource-abuse-evidence-2026-09-04.md:23`) should be read as wire-silence + tested process exit for the ordinary probe (`probe.rs:2807-2812`), and as wire-silence + process survival only for the failover UDP path. This is consistent with the declared "bounded temporary research probe; not a public listener" scope and is already captured by the open `RSEC-001` adversarial-load finding; it is **not** a defect against any claim.
- **[nit] Additional per-state inner budget.** `PreauthLimits::default()` (`neko-crypto/src/lib.rs:802-811`) adds a per-state cap of 8192 input bytes / 4 input packets / 2048 response bytes / 4 response packets. The ADR table has no per-state *input* row, so the effective per-state input ceiling is stricter than the per-source 64 KiB / 64 packets. Stricter, not weaker; numeric policy is explicitly **deferred to the policy owner** and not decided here.
- **[nit] `memory_bytes` excludes the source-key map.** `ProcessPreauthAdmission::memory_bytes` accounts `RESERVED_STATE_BYTES` per state (`preauth.rs:10`, `:249`) but not the `sources` BTreeMap keys (<=256 B x <=1024 -> <=256 KiB, ~1.6% above the 16 MiB cap at full occupancy). Bounded and observability-only.
- **[nit]** Earlier docs cite "eight"/"seven" inventoried surfaces historically (`docs/reviews/release-item4-subgates-20260909.md:51`); the current authoritative count is 9 surfaces / 7 admission sites, machine-checked at `f184cf2`. No action required.

## Boundary of this review

Performed: exact-callsite ordering audit of all 9 surfaces, numerical ADR-conformance check, permit/cleanup lifecycle review, executable inventory/manifest checks, and focused test reproduction. **Not** performed: independent cryptanalysis, adversarial/load testing, rate-suitability judgment of candidate values, D019 retention policy decision, service-manager hardening, WAN/VPS execution, or secrets access.

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `RSEC-001` open; `D019` `SOURCE_RETENTION_POLICY_BLOCKED`.
