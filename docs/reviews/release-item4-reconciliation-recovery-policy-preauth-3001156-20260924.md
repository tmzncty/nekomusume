# Release/item-4 factual reconciliation — Recovery policy gate + pre-auth/current-owner refill

**Repository anchor before this note:** reachable `3001156724537c11eb427cb6edd9ddeb11be4a3c`.

**Product/test source anchor:** `68d938279478b91661f786e830e99af83db457f1`; commits after it through this reconciliation are reviewer navigation/evidence notes only.

This is the grouped reconciliation after three coherent reviewer slices: current Recovery residual challenge, pre-auth owner-diff reuse, and repository-wide 13-surface current-owner inventory.

## Factual state

1. **Recovery current owner:** H-I4-116 (zero-loss Reno), H-I4-117 (runtime persistent-congestion wiring), and H-I4-118 (stale-ACK time-threshold loss) are repaired with reachable developer-local exact-tree provenance. The current residual Recovery challenge found no additional concrete defect outside the disputed H-I4-119 seam.
2. **H-I4-119:** remains a **MAINTAINER / CORE ACK-PTO SEMANTICS GATE**, not a coding-agent correctness repair. Repository truth does not currently determine whether PTO backoff reset must be any newly acknowledged sent packet or only newly acknowledged ack-eliciting packets. No reviewer-side patch is authorized.
3. **Pre-auth:** exact owner diff from the latest dedicated pre-auth challenge shows no later source movement in `crates/neko-cli/src/preauth.rs`, `crates/neko-crypto/src/lib.rs`, or production pre-auth call sites in `crates/neko-cli/src/main.rs`; the prior bounded no-finding therefore remains current. D019 and RSEC-001 remain separate gates.
4. **Core inventory:** all thirteen required implemented core surfaces now have a current dedicated challenge or exact owner-diff reuse classification. No second materially moved/unreviewed production-code owner was identified at source anchor `68d9382`.
5. **Live:** no new code/instrumentation/hypothesis/path condition creates a fresh real-network question. `READY_LIVE: none` remains authoritative.

## Release packet delta

`docs/release-security-review-packet.md` remains an evidence index, not an attestation. It has not yet been reconciled with the latest H-I4-116/117/118 repair chain, H-I4-119 policy reclassification, current CarrierState/manager owner-diff reviews, or the Recovery/pre-auth/current-owner notes from this run.

A later narrow packet-index maintenance slice may add those reachable review/provenance references while preserving historical anchors and evidence classes. It must **not**:

- invent one all-tests SHA;
- promote reviewer source review into developer-local CI or hosted CI;
- change item-3/item-4 completion state merely because references were indexed;
- change `RELEASE_CANDIDATE`, `PRODUCTION_READY`, `FREEZE`, or `RELEASED`;
- characterize H-I4-119 as resolved;
- erase D019, RSEC-001, retained-history capacity, signing/key-custody/SBOM/publication, external review, or release-authority gates.

One wording seam in the existing packet should also be treated carefully during maintenance: an older Session-delivery-context boundary says there is no independent review, while later rows now index dedicated independent SessionRuntime/FairScheduler review and repair evidence. The intended current meaning is that there is no *complete release/security validation or protocol-freeze approval*, not that no independent bounded Session review has ever occurred. Update only if the packet owner confirms that wording is meant globally; do not rewrite historical evidence to manufacture broader assurance.

## Item-4 consequence

This reconciliation does **not** close item 4. The next real local work is narrower than the earlier queue:

1. **R-RPKT-CURRENT:** packet evidence-index/factual wording reconciliation for the latest reachable review chain, preserving evidence boundaries.
2. **R-SEC-DIFF:** exact security-boundary owner diff across crypto/trust/pre-auth/security owners since their latest dedicated reviews; create a new challenge only if a semantic owner actually moved.
3. **R-POST-DIFF-REFILL:** rerun repository owner history after those two slices or immediately after any new developer commit; refill from any moved semantic owner.
4. **Conditional current-owner spot challenge:** only if R-SEC-DIFF or new developer work exposes a materially changed/unreviewed implemented owner. Do not create filler.
5. **Conditional live:** only for a new concrete unresolved real-network question within standing authorization.

If these local factual/diff lanes close without owner movement, the remaining gaps may predominantly be policy/external/environment/release-authority gates; perform one final repository-wide reconciliation before any `queue exhausted` classification.

## Evidence boundary

This note is GitHub source/history/review-ledger reconciliation. It does not claim reviewer-local Rust/full-gate execution, hosted CI, fuzz, WAN, cross-platform execution, adversarial-load evidence, or performance results. No code, wire/parser/crypto framing, release flag, or protocol architecture is changed by this note.

Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
