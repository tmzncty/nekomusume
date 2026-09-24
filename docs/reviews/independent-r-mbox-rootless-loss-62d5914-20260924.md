# R-MBOX-ROOTLESS-LOSS — bounded independent source/control-flow review at `62d5914`

**Reviewed developer source/test anchor:** reachable `62d59141dfe30ea1412e9e9d330c5c21c697f7f9`.

**Review result:** no concrete correctness/security defect found in this bounded source/control-flow pass. The slice is **not yet evidence-closed** because no reachable developer-local exact-tree gate/provenance for `62d5914...` is present at review time. This note is reviewer source review only; it does not claim Rust execution, hosted CI, WAN, fuzz, cross-platform, or performance evidence.

## Developer delta reviewed

Relative to reviewer handoff parent `95e9a2c0d84980e91f13a0902664a4b44fb3b2c0`, the developer-owned commit changes exactly:

- `crates/neko-carrier/src/lib.rs` — deterministic `FaultInjectCarrier` tests for the already-existing `one_way` and `loss_percent=100` policies;
- `crates/neko-cli/src/main.rs` — opt-in `--drop-all-udp` behavior in the bounded experimental failover server plus `hard_udp_loss` diagnostic classification;
- `crates/neko-cli/tests/probe.rs` — a bounded loopback hard-UDP-loss -> cold TCP failover integration regression using existing child-process deadline helpers.

The change does not modify wire decode/parser/crypto framing, Session delivery semantics, ACK/PTO semantics, CarrierState policy values, D019, release flags, or standing authorization.

## Invariant challenged

The merged middlebox/reachability plan requires a rootless first step that can model UDP reply cessation / one-way silence and hard UDP loss without host route/firewall/qdisc mutation, while preserving the existing split:

- Carrier owns path health/failure/promotion evidence;
- Session owns authenticated protocol/delivery state;
- a failure fixture must not invent a new readiness/hysteresis/PTO/capacity/security policy value;
- bounded test-process ownership must remain intact.

The exact-current integration test uses an opt-in server-side reply blackhole (`--drop-all-udp`) and existing `--automatic-health-failover --cold-health-failover`; it keeps the ordinary failover path unchanged when the flag is absent. It uses `bounded_client_output(..., 12s)` and the existing bounded server owner, and asserts health failure, cold fallback classification, bounded TCP delivery-ACK evidence, complete 3-record/48-byte Session delivery and no fabricated duplicate metric.

## Interpretation boundary

`FaultPolicy::one_way` itself is **not** a directional network topology model: its existing implementation suppresses alternating sends from one wrapped endpoint (`sent % 2 == 0`). The new unit test correctly demonstrates that exact deterministic behavior, but its comment/name must not be promoted into evidence that the in-memory wrapper alone proves one-directional UDP reply cessation.

For this slice, the actual rootless path-behavior evidence comes from:

- the pre-existing `--cease-udp-replies-after` process seam for replies-then-silence; and
- the new opt-in `--drop-all-udp` process seam for reply blackhole from the first datagram.

No repair is required merely to rename the historical `one_way` field. Future R-MBOX notes should preserve this evidence boundary instead of calling alternating-send suppression a real directional path.

## Evidence/provenance gap

At review time:

- GitHub combined status for `62d5914...` exposes no status entries;
- no PR-triggered hosted workflow run is visible for the SHA;
- no reachable repository note records the required developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, UTC start/end, OS/arch, and stable Rust version for this source tree.

Therefore R-MBOX-ROOTLESS-LOSS is **source-reviewed but closure-pending-local-provenance**. The coding agent should run the normal final-pushed-SHA gate in a clean checkout/worktree and persist a reachable exact-tree provenance note. If that gate exposes a concrete defect, the defect becomes FRONT and this no-finding source review does not override it.

## Exclusions

This review did not execute Rust tests locally, did not run fuzz (not required by the delta), did not perform WAN/VPS experiments, did not exercise privileged netem/netns/qdisc, and did not claim performance or public-network reachability. `READY_LIVE: none` remains unchanged.

## Queue consequence

After exact-tree provenance for `62d5914...` is persisted (or a repaired successor if the gate fails), continue immediately with the existing dependency-ordered queue:

1. R-MBOX-REORDER-DELAY;
2. R-MBOX-MTU;
3. R-MBOX-CLEAN-RECOVERY;
4. R-MBOX-REPEATED-FAILOVER;
5. R-MBOX-RESOURCE;
6. R-MBOX-TRANSITION-MATRIX;
7. grouped release/item-4 factual reconciliation after 3–4 coherent slices;
8. conditional refill of any materially moved core/CLI owner;
9. conditional live only if a new concrete real-network question appears within standing authorization;
10. final repository-wide 13-surface reconciliation before any queue-exhausted statement.

H-I4-119 remains a maintainer/core ACK-PTO semantics gate and is outside this slice.
