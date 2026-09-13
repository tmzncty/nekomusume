# ChatGPT reviewer handoff — open Core Live Reliable-UDP Integration phase

## Why the queue is open again

The prior exact `3d88ac956e6a8b9616a215178b4db12c8fd3be94` handoff correctly concluded that the **existing implemented/candidate surfaces under the then-current semantics** had been repository-wide bounded-reviewed and that no mechanically repairable defect remained.

That did **not** mean the project itself was complete. The maintainer has now explicitly directed the project to continue advancing rather than remain parked at the current bounded-research plateau.

The next high-value mainline gap is concrete and repository-visible:

- `neko-reliable` contains the socket-free UDP recovery engine (packet number, ACK ranges, RTT/loss/PTO, frame retransmission, Reno/pacing and deterministic fault models).
- `neko-cli` currently does **not** depend on `neko-reliable`.
- `neko-carrier` currently does **not** depend on `neko-reliable`.
- The candidate outer wire already has `RecordType::Ack`, but its payload is still opaque and no live UDP recovery integration exists.
- `docs/status.md` truthfully describes `reliable-udp` as a bounded deterministic packet-recovery model with no live service / Session-delivery promotion.
- Item 3 still lacks natural-loss / packet-recovery-driven degradation evidence; the accepted warm fallback is application reply cessation, not PTO/packet-loss evidence.

Therefore the previous `queue exhausted` state is superseded by a **new implementation phase**, not by another review-filler sweep.

## Phase objective

Integrate the existing `neko-reliable` recovery semantics into the real UDP Carrier path in a bounded, locally testable way, then use that new implementation/instrumentation to create a genuinely new real-network hypothesis for item 3.

This phase must preserve existing architecture:

- Session remains above Carrier.
- UDP packet ACK/recovery is Carrier-local and must never call or imply Session `DeliveryLedger::confirm_received`.
- TCP does not gain a second packet-ACK layer.
- Existing Noise/authz/Session-delivery semantics are not redesigned.
- No concurrent UDP+TCP striping, 0-RTT, FEC enablement or unrelated Experimental Track work.
- Existing D019, `SessionRuntime.events` retention and RSEC-001 policy gates remain open and **must not block this independent implementation lane**.

## Continuous READY_LOCAL queue

The external coding agent should execute continuously. A coherent source/test slice -> focused tests -> pushed developer commit -> developer-local clean exact-tree gate/provenance -> immediately continue to the next dependency-ready slice. Do not wait for reviewer cadence unless a listed stop condition is reached.

### S1 — Define the Carrier-local live-recovery integration boundary

**Goal:** introduce the smallest ownership seam that lets a UDP Carrier own `neko_reliable::Recovery` without coupling packet ACK to Session delivery.

**Owner:** `crates/neko-carrier` + manifests; `neko-reliable` remains its own engine crate.

**Actions:**
- add `neko-reliable` as a normal dependency of the carrier layer (not only CLI glue) unless exact-current structure proves a smaller equally layered seam;
- define a minimal path-local recovery wrapper/state for packet numbers, sent packets, ACK application, PTO and retransmit-frame output;
- no socket behavior yet if a socket-free integration seam is the smallest safe first slice;
- tests must prove packet feedback cannot mutate Session delivery evidence.

**Do not:** create a new crate, redesign Session ACKs, invent new congestion policy values.

### S2 — Specify and implement a bounded candidate UDP packet/ACK payload grammar

**Goal:** assign a deterministic bounded grammar to the already-existing outer `RecordType::Ack` and the matching UDP Data packet metadata required by `neko-reliable`.

**Owner:** `neko-wire` + `docs/spec/` candidate documentation + golden/negative tests.

**Requirements:**
- packet number identity is explicit and bounded;
- ACK ranges encode canonical sorted/merged inclusive ranges with hard count limits;
- reject malformed, noncanonical, duplicate/overlapping noncanonical forms, range inversion, truncation, overflow and trailing bytes;
- do not encode Session delivery acknowledgement in this format;
- current outer version remains candidate/non-frozen unless an unavoidable version change is justified by an explicit ADR.

**Validation:** focused wire tests + full `scripts/check.sh`; because decoder/parser changes are expected, run pinned decode fuzz smoke required by repository policy.

### S3 — Bind recovery metadata to authenticated UDP records

**Goal:** packet number / ACK evidence must be authenticated under the existing crypto/transcript/AAD boundary before it can affect recovery state.

**Owner:** carrier/crypto integration and tests.

**Requirements:**
- unauthenticated or tampered packet/ACK material cannot advance RTT, retire packets, declare loss, schedule retransmit or affect Carrier health;
- duplicate/old/future ACK behavior remains fail-closed according to current `neko-reliable` semantics;
- no Session delivery evidence is produced.

**Stop if:** this requires changing the core Noise construction or Session/Carrier architecture rather than integrating existing authenticated record ownership.

### S4 — Local loopback live reliable-UDP exchange

**Goal:** run actual UDP sockets through packet numbering + ACK + recovery state rather than a socket-free simulator.

**Owner:** `neko-carrier` integration tests and, only where needed, bounded CLI test fixture support.

**Positive coverage:** multi-record authenticated exchange, ACK retirement, RTT sample, no spurious retransmit.

**Negative coverage:** duplicate ACK, old ACK, future ACK, malformed ACK, tamper, wrong generation/context, socket close/timeout cleanup.

This is loopback evidence only; do not update WAN/release claims.

### S5 — Deterministic local loss/reorder integration

**Goal:** prove the live integration actually drives `neko-reliable` loss/PTO/retransmit behavior under controlled loss rather than merely carrying ACK-shaped bytes.

Prefer deterministic local fault injection / netns existing harness over sleeps or ad-hoc timing.

Cover at least:
- one lost Data packet -> frame-level retransmission -> exact application data once;
- ACK loss -> safe retry without false Session confirmation;
- bounded reorder below loss threshold;
- blackhole -> PTO/probe evidence without fabricating packet loss beyond current semantics;
- recovery state cleanup and bounded memory.

### S6 — Congestion/pacing integration checkpoint

**Goal:** connect existing Reno / bytes-in-flight / pacing calculations to the live UDP send decision in the smallest deterministic form.

**Requirements:**
- do not busy-loop or use a simple unaccounted sleep loop as a pacing implementation;
- bytes-in-flight accounting must retire exactly once on ACK/loss;
- packet retransmission must remain frame-level re-encoding, never resend an old encrypted packet image;
- deterministic/netns evidence first; no performance superiority claim.

If the existing synchronous architecture cannot support a truthful pacing seam without choosing a new runtime/timer architecture, record that as a concrete design stop and continue any independent slices that do not depend on it.

### S7 — Carrier health bridge from real recovery evidence

**Goal:** feed authenticated packet-level recovery observations into the existing Carrier health/manager domain without crossing into Session delivery.

Map current evidence only:
- RTT/loss/PTO -> health observations;
- recovery events may mark/degrade Carrier health only through existing manager semantics;
- packet ACK cannot validate a Path and cannot confirm Session delivery;
- hysteresis/single-active invariants remain unchanged.

Add focused cross-layer negative tests.

### S8 — Packet-loss-driven UDP degradation -> existing TCP fallback locally

**Goal:** replace the current application-reply-cessation-only trigger in at least one local integration fixture with a true packet-recovery/health-driven degradation signal feeding the already accepted warm TCP fallback path.

Requirements:
- TCP standby still proves readiness independently;
- Session uncertain/replay/dedup semantics remain the existing D064 contract;
- report failure decision time, first resumed data, uncertain/replayed/duplicate/lost application bytes;
- no TCP packet ACK layer;
- positive + fail-closed negative tests.

This is the first slice that can materially create a new item-3 live hypothesis.

### S9 — Observability and result truth boundary for live recovery

**Goal:** expose secret-free structured events/counters needed to distinguish:
- packet sent / ACKed / lost;
- PTO fired;
- frame retransmitted;
- Carrier degraded / switch decision;
- Session confirmed/uncertain application bytes.

Reuse existing `neko-observe`; do not add a parallel logging framework. Packet recovery events must not be mislabeled as logical delivery.

### S10 — Independent bounded review of the new integrated surface

After S1-S9 are coherent and green, perform a dedicated independent bounded challenge of the new live reliable-UDP integration:
- authentication/evidence-domain separation;
- future/unsent/duplicate ACK;
- packet/frame exact-once accounting;
- retransmission crypto freshness;
- timeout/PTO/loss state transitions;
- memory/queue bounds;
- manager/fallback boundary;
- secret-safe observability.

Concrete defect -> smallest repair + regression + exact-tree closure. No finding -> retain a precise no-finding review note.

### S11 — Create a genuinely new READY_LIVE row

Only after S10 accepts the local integration, reconcile item 3. The implementation change itself creates a materially new hypothesis, so a new bounded self-owned VPS experiment may become READY under standing authorization.

Preferred first live question:

> Does authenticated live UDP packet-recovery evidence detect a bounded controlled packet-loss/blackhole condition and trigger the existing warm TCP fallback while preserving Session delivery/dedup invariants?

Use the minimum experiment that answers this question; keep within standing authorization; preserve cleanup and exact binary/commit provenance. Do not rerun the old application-reply-cessation sample and call it new evidence.

If production qdisc/route/firewall changes would be required to create the loss condition, stop that live mechanism and use an already authorized/non-production isolated injection method or keep the row blocked; do not modify production networking.

### S12 — Item-3 / item-4 reconciliation and next queue refill

After a new live result (positive or negative):
- update status/evidence boundary honestly;
- distinguish controlled packet loss from natural Internet degradation;
- do not convert one sample into a reliability/performance rate;
- preserve D019, RSEC-001, `SessionRuntime.events`, signing/SBOM/RC authority as separate gates;
- refill the next engineering queue from the new evidence, not from speculative Experimental Track features.

## Required gates

For every coherent code slice, the developer-local clean exact pushed-tree gate is primary:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
clean initial/final tree
```

Persist sanitized provenance: exact SHA, commands, UTC start/end, exits, OS/arch, stable Rust, clean-tree state. GitHub Actions is optional extra cross-evidence and never a waiting condition.

Wire decoder/parser changes require the repository's pinned decode fuzz build/run (`-max_total_time=30 -max_len=8192`). Ordinary carrier/docs-only slices do not.

## Policy gates deliberately left open

This new phase is authorized by the maintainer's direction to continue project engineering; it does **not** silently decide the following:

- `SessionRuntime.events` retained-state policy (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 terminal source-retention/no-reset policy;
- RSEC-001 capacity/suitability pressure profile;
- signing/key custody/SBOM/publication policy;
- previous-frozen-release interoperability policy;
- RC/freeze/release/production authority.

Those remain separate decisions and should not block the independent reliable-UDP integration queue above.

## Stop conditions

Stop the affected lane only for:
- unresolved correctness/security/evidence BLOCKER/HIGH;
- a required change to core Session/Carrier/ACK/crypto architecture not already determined by committed semantics;
- destructive/canonical-meaning migration;
- a new safety numeric/policy choice;
- action outside standing authorization or affecting production/third parties;
- new credentials/server permissions;
- real repository breakage or actual runtime/tool exhaustion.

Otherwise continue S1 -> S12 without waiting for the next reviewer pass.

## Governance remains unchanged

Opening this engineering phase changes no release flag:

- item 3 remains incomplete;
- item 4 remains incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- `READY_LIVE: none` remains authoritative **until S10/S11 create and review a new concrete live question**.
