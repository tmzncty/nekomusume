# H-I4-121 — rootless hard-loss evidence boundary mismatch

**Severity:** HIGH — release-item-4 evidence reliability.  
**Reviewed source/test anchor:** reachable `62d59141dfe30ea1412e9e9d330c5c21c697f7f9`.  
**Review class:** reviewer source/control-flow only; no Rust/full-gate, hosted CI, WAN, fuzz, cross-platform or performance execution is claimed here.

## Finding

The current R-MBOX-ROOTLESS-LOSS review/handoff overstates the semantics of the new `--drop-all-udp` seam.

Exact-current `failover_server` parses `--drop-all-udp`, but the flag is only consulted around the authenticated Session `DeliveryAck` reply branch. Before that point the same UDP server still sends version-selection responses and Noise handshake responses without consulting `drop_all_udp`; cached negotiation/handshake replies are likewise independent of that flag. In reliable-UDP mode, Carrier packet ACK sends are also outside the `drop_all_udp` guard.

Therefore the statements that `--drop-all-udp` means “the server receives UDP datagrams but never sends a UDP reply”, “reply blackhole from the first datagram”, or a process-level “hard UDP black hole” are not true for exact `62d5914...`.

The new process test actually proves a narrower and still useful invariant: UDP negotiation/authentication can succeed, then authenticated application-level UDP replies are withheld from the first application record, health evidence drives cold TCP fallback, and Session delivery completes over TCP without the tested duplicate-delivery symptom.

Separately, the `FaultInjectCarrier { loss_percent: 100 }` unit test does prove deterministic all-send loss at that in-memory Carrier seam. It is valid hard-loss fault-injection evidence for that owner, but it is not process/network-topology evidence.

## Why this is HIGH

`docs/reviews/independent-r-mbox-rootless-loss-62d5914-20260924.md` currently says the process seam is a reply blackhole “from the first datagram”, and `docs/CHATGPT_HANDOFF.md` would allow the slice to become evidence-closed after provenance alone. Persisting exact-tree provenance without correcting the path-condition classification would turn a source-review overclaim into accepted item-4 evidence.

This does not establish a production transport correctness defect. It is an evidence/harness truth defect in a release-item-4 support slice.

## Smallest authorized closure

Do not change Session/Carrier/ACK/crypto/wire architecture and do not invent policy values.

1. Narrow the process-seam contract to what exact code can prove. Rename/reword the new process diagnostic/test/comments so they describe a **post-authenticated UDP application-reply blackhole** (or equivalent precise wording), not “all UDP” / “from first datagram”. The machine-readable `failover_mode=hard_udp_loss` label must not remain if the implementation still allows negotiation/handshake UDP replies.
2. Keep the existing `FaultInjectCarrier(loss_percent=100)` regression as deterministic Carrier-level hard-loss evidence, explicitly separated from topology/process evidence.
3. Preserve the process assertions that matter: successful bounded failover, `udp_health_failed`, cold fallback classification, complete 3-record/48-byte TCP delivery, bounded child ownership, and no fabricated duplicate metric.
4. If the developer chooses instead to make a process flag literally suppress every UDP response, first ensure the resulting scenario remains dependency-compatible with the intended failover test; do not silently change startup/negotiation semantics merely to match the old label. A true startup hard-block question may be a separate bounded slice.
5. Run the final pushed repaired source SHA through `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist reachable developer-local exact-tree provenance with exact UTC start/end, exit codes, OS/arch and stable Rust version. No decoder/parser/crypto-framing change is implied, so no mechanical fuzz run.
6. Supersede the specific “from first datagram / server never replies / hard black hole” claims in the earlier reviewer note and handoff; do not rewrite historical evidence silently.

## Queue consequence

H-I4-121 is FRONT before R-MBOX-ROOTLESS-LOSS can be evidence-closed. After the narrow evidence-truth repair + exact-tree provenance, continue immediately with R-MBOX-REORDER-DELAY and the existing deep queue. `READY_LIVE: none` remains unchanged.

H-I4-119 remains a separate maintainer/core ACK-PTO semantics gate and is untouched by this finding.
