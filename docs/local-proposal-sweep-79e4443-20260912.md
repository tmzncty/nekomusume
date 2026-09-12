# Local bounded proposal sweep — exact `79e4443`

Developer-performed one-time bounded proposal sweep required by reviewer handoff `79e4443` (`docs(handoff): close superseded PR and require bounded proposal sweep`). Inspected reachable exact `79e44431b3cb2a90f93985c60e2e7129958da252` source and tests. This is not an independent review, security approval, release decision, or a claim that no defect exists anywhere.

## Scope inspected

- `crates/neko-cli/src/preauth.rs` — full file: `remaining_budget`, `write_all_until`, `write_frame_until`, `write_frame_restoring_timeout`, `read_staged_frame`, `ListenerAdmission` admit/charge/response/send/queue/release, `source_key`, and all unit tests.
- `crates/neko-cli/src/framed.rs` — staged frame read and charge-before-allocate ordering.
- `crates/neko-cli/src/main.rs` — UDP responder call sites of `send_udp_response` (selection, Noise response, cached retry) and their `fail()`/expiry/pending-owner handling.
- `docs/preauth-responder-inventory.v1.json` — 9 responder surfaces across 4 files / 7 admission sites.
- `docs/reviews/independent-resource-abuse-review-f184cf2-20260911.md` — already-confirmed non-policy controls and declared nits.

## `PROPOSAL_SWEEP: no dependency-ready candidate`

No candidate satisfies `ACCEPT_CANDIDATE` under the required seven fields. Findings, honestly classified:

### Considered and rejected — `send_{tcp,udp}_response` post-send socket-timeout restore failure

- **Owner:** `crates/neko-cli/src/preauth.rs` `send_udp_response` (`:480-484`) and `send_tcp_response` via `write_frame_restoring_timeout` (`:130-141`, `:430-441`).
- **Observation:** when the actual `send_to`/frame `write` has already succeeded but restoring the prior socket `set_write_timeout` fails, the permit is `abandon_response`d and the caller `fail()`s. `abandon_response` keeps the response charged and terminalizes the pre-auth state (`crates/neko-crypto/src/lib.rs:1617-1635`).
- **Why not ACCEPT_CANDIDATE:** this is **not** a behavior-vs-claim inconsistency. Both TCP and UDP paths apply the identical rule — a send that cannot be cleanly confirmed/settled is abandoned and terminalized. `abandon_response` retaining charge and rejecting the state is the deliberately reviewed fail-closed semantic (independent review `f184cf2`, `preauth.rs` permit lifecycle). Whether a purely-local bookkeeping failure (timeout restore) after a physically-successful send should instead settle as `complete_response` is a fail-closed-vs-fail-open **value/policy judgment**, not an answer already fixed by current semantics. Classified `DEFER_POLICY_OR_ENVIRONMENT` at most; it is not a defect whose answer current semantics already determine, so it is not implemented.

### Considered and rejected — `memory_bytes` excludes source-key map

- Already recorded by reviewer as a bounded observability-only nit (`f184cf2` nit list); not a new contradiction. `REJECT_FILLER`.

### Considered and rejected — process-exit vs in-process recovery path difference

- Already recorded by reviewer as consistent with the declared bounded temporary research probe scope and folded into open `RSEC-001`; "not a defect against any claim." `REJECT_FILLER`.

## Conclusion

`PROPOSAL_SWEEP: no dependency-ready candidate`. No code, wire, test, release flag, or live/VPS action was taken or is warranted by repository truth at `79e4443`. Idle per handoff; resume only on a new dependency-ready defect/question or a maintainer decision that unlocks a blocked gate.
