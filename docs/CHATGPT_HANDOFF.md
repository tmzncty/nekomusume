# ChatGPT reviewer handoff — H-R9-083 FRONT HIGH; R9-11D remains open

## Current repository truth

- Current reachable source/test repair anchor is `0831076eb96e4fd2fbbb71bed7190a21d41fca0f`. The immediately following `2aea079` accidentally committed only `scripts/bench/__pycache__/process-resource-sampler.cpython-312.pyc`, and `0670ac4` removed it; neither changes product/test semantics. Reviewer/navigation commits after that do not supersede the source/test anchor unless they touch executable owners/tests.
- H-R9-082 fixed the original false inference from process-group emptiness for escaped **TCP** listeners: terminal cleanup now checks caller-owned TCP/TCP6 LISTEN ports before reporting zero sockets. Developer-local clean exact-tree provenance for exact `0831076`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T08:58:37Z -> 2026-09-20T09:03:51Z, Linux x86_64, rustc 1.98.0. No reviewer-local execution is claimed.
- **H-R9-083 is OPEN HIGH** at reviewer anchor `9b44781547acf6373f77c23045b0569d07b72db7`. The replacement terminal oracle is incomplete: `read_proc()` defines `--owned-port` accounting across TCP/TCP6/UDP/UDP6, but `owned_port_listeners_present()` scans only TCP/TCP6 LISTEN state. An escaped descendant retaining a supplied UDP socket can therefore still be certified as `owned_sockets_after_exit=0` / `cleanup.complete=true`. The helper also swallows `/proc/net` read/parse errors and returns `False`, so unknown observation is promoted to absence. Read `docs/reviews/h-r9-083-owned-port-terminal-oracle-20260920.md`.
- Developer bounded R9-11D note `7ff2a0d750dff69d0477018977b6567e62e0c251` remains useful support but is **not** accepted as R9-11D closure while H-R9-083 and the dedicated D1-D4 independent lanes remain open.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED as bounded independent no-finding support at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- Sampler observation-handshake determinism HIGH remains closed at `e021b27cffe4a87de185757a8d47b8685f462bb6`; H-R9-083 is a distinct terminal socket-evidence seam.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: H-R9-083 terminal owned-port oracle

Read `docs/reviews/h-r9-083-owned-port-terminal-oracle-20260920.md` and exact-current sampler/validator/tests before editing.

Required closure:

1. make terminal owned-port evidence cover the same committed protocol surface as sampling: TCP/TCP6 listeners plus UDP/UDP6 bound sockets for caller-supplied ports;
2. represent terminal observation as present / absent / unknown (or equivalent fail-closed state); incomplete `/proc/net` observation must never become `owned_sockets_after_exit=0` or `cleanup.complete=true`;
3. add a deterministic escaped-descendant UDP regression: descendant leaves the original group, binds the supplied loopback UDP port, signals readiness after bind, original group ends, and cleanup remains incomplete while the UDP socket exists;
4. keep the escaped TCP regression green and prove challenged PID/socket existence before accepting the negative oracle;
5. bounded-clean escaped helpers even when assertions fail (for example `try/finally`), without arbitrary sleep as the ownership oracle;
6. run focused process-resource tests, then final pushed exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, and persist exact SHA/UTC/exits/OS-arch/Rust provenance.

No decoder/framing change: do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, or invent timeout/capacity/security policy values. After closure, continue immediately to READY_LOCAL 2.

## READY_LOCAL 2 — R9-11D1 process result-truth / terminal classification

Independently re-challenge exact-current automatic-health failover/migration-back executable owners and process fixtures. Verify success, explicit failure, timeout, shutdown and post-return classifications:

- terminal failure/timeout/stop cannot later be rewritten as READY/success because cleanup/read/child work returns zero;
- structured result JSON, human output and exit status describe the same authoritative terminal outcome;
- missing/unknown cleanup stays failure/unknown;
- post-return work cannot manufacture new delivery/switch/readiness success evidence;
- negative fixtures prove the intended failure phase was actually reached.

Concrete defect -> smallest repair/regression/gate. No defect -> scope-precise bounded no-finding note, then continue immediately to D2.

## READY_LOCAL 3 — R9-11D2 TCP/UDP listener and socket ownership

After H-R9-083:

- success/failure/timeout/shutdown deterministically release no-longer-owned TCP/UDP sockets;
- cleanup is proved by owner/socket evidence or committed same-address/port rebind oracle, never inferred from child exit or original-PGID emptiness;
- escaped/reparented descendants cannot retain TCP **or UDP** ownership while cleanup reports zero;
- partial setup failure cannot strand sockets while returning success/clean;
- repeated close/shutdown is idempotent and creates no false-positive lifecycle evidence.

Repair findings immediately; otherwise record bounded no-finding and continue to D3.

## READY_LOCAL 4 — R9-11D3 child/process-group/temp-runtime cleanup

Challenge process-owned resources independently of socket ownership:

- bounded reap/kill path on success, failure, timeout and interruption, including descendants that leave the original group;
- readiness/observation handshakes remain deterministic;
- temporary runtime paths are removed or truthfully reported unknown/failed; cleanup failure is never silently promoted;
- cleanup reporting exposes no secrets/private topology/unnecessary absolute paths;
- no new timeout/capacity/security policy values merely to satisfy tests.

Concrete defect -> repair/regression/gate; no defect -> bounded no-finding and continue to D4.

## READY_LOCAL 5 — R9-11D4 shutdown / post-return false-success negatives

Adversarially cross-check executable failover/migration-back and probe/process fixtures:

- shutdown during setup/traffic/recovery leaves no later READY/success/delivery event;
- timeout followed by late child/socket completion cannot overwrite terminal outcome;
- cleanup/rebind evidence belongs to the same exact run/process owner, not a stale predecessor;
- application/session success proof plus truthful cleanup/result state is required; cleanup-only success is insufficient;
- failure/unknown remains failure/unknown even if a later independent postcheck finds zero residue.

Do not rewrite immutable historical artifacts. Then do one concise R9-11A-D factual reconciliation.

## READY_LOCAL 6 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair.

## READY_LOCAL 8 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 9 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 10 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 11 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from R9 alone. Prefer implemented surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the prior challenge:

1. `neko-reliable` ACK ranges / future-unsent / loss-retransmit / RTT-PTO / persistent congestion / Reno / fault simulation;
2. `CarrierState` generation / validation / hysteresis / single-active / drain-fail-activate;
3. concurrent manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Memory/UDP/TCP carrier close/error/resource semantics;
6. SessionRuntime lifecycle/resource/window/DeliveryAck accounting;
7. observability event/counter/high-water projection;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarking or invented policy values;
13. release-packet factual consistency/evidence boundaries.

No-finding independent review is valid item-4 support. Do not count one narrow no-finding as repository-wide exhaustion.

## READY_LOCAL 12 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
