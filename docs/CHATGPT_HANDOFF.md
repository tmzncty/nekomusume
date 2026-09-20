# ChatGPT reviewer handoff — H-R9-083 FRONT HIGH; R9-11D remains open

## Current repository truth

- Latest developer-owned executable source/test commit reviewed is `d1554a21258c4fe25e8b2ea0a07768f66133236c` (`fix(bench): H-R9-083 all four net tables must parse before absent; escaped fixtures reap in finally`). It is reachable on `main` through reviewer note `3b50263c6f9bfeadf7c3e88b7c24854175d71188` and this handoff.
- `d1554a2` **does** close the previous `files_read > 0` false-absence shape: any required `/proc/net/{tcp,tcp6,udp,udp6}` read `OSError` now yields unknown; local-port parse failure yields unknown; escaped TCP/UDP helper cleanup moved under `try/finally`.
- **H-R9-083 remains OPEN HIGH** after independent review at `docs/reviews/h-r9-083-d1554-closure-challenge-20260920.md` (`3b50263c6f9bfeadf7c3e88b7c24854175d71188`). Three bounded gaps remain:
  1. TCP terminal-row parsing still reads `fields[3]` outside the guarded parse block, so a truncated row with a valid caller-owned local port but no state field raises `IndexError` instead of returning unknown;
  2. the already-required deterministic partial-observation regression is still absent — current tests do not inject an unreadable required table or malformed terminal row and prove `unknown` plus `cleanup.complete=false`;
  3. escaped TCP/UDP fixtures write readiness markers after bind but never wait for/assert them before the spawning parent exits, so ownership establishment still depends on scheduler ordering. Use a bounded readiness/ownership handshake; do not replace this with a fixed sleep.
- The current source semantics remain correct for the normal four-table Linux happy path and the specific previous unreadable-table counterexample, but release-gate closure truth is not strong enough to close H-R9-083 or R9-11D2/D3.
- No reviewer-local test execution is claimed. `d1554a2` has not yet added a persisted final clean exact-tree provenance record for itself. Hosted CI remains supplemental and is not a wait condition.
- H-R9-082 remains closed for its original process-group-empty-only false-cleanup counterexample; H-R9-083 is the stricter terminal owned-port observation/determinism seam. Developer bounded R9-11D note `7ff2a0d750dff69d0477018977b6567e62e0c251` remains useful support but is not accepted as R9-11D closure while the front HIGH and D1-D4 lanes remain open.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED as bounded independent no-finding support at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- Sampler observation-handshake determinism HIGH for the ordinary sampled child remains closed at `e021b27cffe4a87de185757a8d47b8685f462bb6`; the escaped-descendant readiness gap above is a distinct terminal-cleanup fixture seam.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: close H-R9-083 terminal observation deterministically

Read:

- `docs/reviews/h-r9-083-owned-port-terminal-oracle-20260920.md`
- `docs/reviews/h-r9-083-reopen-partial-proc-observation-20260920.md`
- `docs/reviews/h-r9-083-d1554-closure-challenge-20260920.md`
- exact-current sampler/validator/tests

Required closure:

1. retain TCP/TCP6 LISTEN plus UDP/UDP6 bound-socket coverage for caller-supplied ports;
2. make every field required to decide terminal ownership fail closed: in particular, a TCP row whose local endpoint parses but whose state field is missing/malformed must yield unknown, not exception or absence;
3. add focused deterministic regressions for partial/malformed terminal observation, at minimum:
   - three readable no-match tables + one unavailable required table -> unknown;
   - truncated/malformed TCP row carrying a caller-owned local port but no usable state -> unknown;
   - unknown terminal ownership cannot construct `owned_sockets_after_exit=0` / `cleanup.complete=true`;
4. make escaped TCP and UDP fixtures consume a bounded readiness/ownership handshake that proves bind has happened before the parent path may finish; no arbitrary fixed sleep;
5. keep assertion-safe `try/finally` cleanup and prove helpers do not remain after the fixture;
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
- escaped/reparented descendants cannot retain TCP or UDP ownership while cleanup reports zero;
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
