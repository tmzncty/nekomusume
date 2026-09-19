# Independent R9-11 lifecycle challenge — H-R9-077 receiver ACK ownership survives terminal teardown

Review anchor: reachable `main` exact `ff8f3f71bd9ad980bc4fb20638ef92c8fcdb159a`. Latest developer-owned source/test change remains `e021b27cffe4a87de185757a8d47b8685f462bb6`; that change is sampler-only, so the inspected carrier/runtime owners are unchanged on this tree.

Scope is the R9-11 lifecycle/resource closure only: `PacketAckTracker`, `ReliableUdpRuntime::on_packet_received`, `poll_outgoing_ack`, and `ReliableUdpRuntime::teardown`, plus the existing H-R9-064/065/066 teardown regressions. This review does not change Session/Carrier/ACK wire semantics, D019, any capacity/policy value, release authority, or live/WAN classification.

## H-R9-077 — HIGH correctness/resource-lifecycle finding

`ReliableUdpRuntime::teardown()` quiesces sender-side live ownership (`packet_frames`, retained retransmit plaintext, `PathRecovery`/Reno charge) and then marks the runtime terminal, but it does **not** quiesce the receiver-side `PacketAckTracker`.

`PacketAckTracker` owns generation-scoped live receiver state:

- bounded `AckRanges` containing every observed packet range;
- `largest_observed`;
- `pending_ack`, which is the outstanding ACK-emission obligation.

A concrete sequence is therefore:

1. create `ReliableUdpRuntime` for generation G;
2. `on_packet_received(7, true)` records packet 7 and sets `pending_ack=true`;
3. call `teardown()`;
4. sender-side recovery/plaintext ownership is zero and `torn_down=true`, but the ACK tracker still contains the observed range/largest value and pending obligation.

`poll_outgoing_ack()` correctly returns `None` after teardown because it checks `torn_down`, so no stale ACK escapes on the wire. That is necessary but not sufficient for R9-11: the generation-scoped ACK ownership itself is still retained. The current regression `teardown_is_terminal_and_fail_closed_on_every_mutator` proves only non-emission after a pre-teardown pending ACK; it does not prove that the tracker was emptied.

This contradicts the current R9-11 closure invariant that terminal path/runtime teardown releases live sent/retransmit/Reno/**ACK** ownership. Unlike the intentionally preserved Recovery lifetime diagnostics (`packets_sent/lost`, RTT/PTO/history), the receiver ACK tracker has no documented lifetime-diagnostic role and no public historical projection that requires retaining its ranges or pending obligation after terminalization.

The state is hard-bounded, so this is not an unbounded-memory finding. It is nevertheless a lifecycle correctness/evidence HIGH because a release review claiming complete terminal ownership reconciliation would be false while generation-scoped receiver ACK state survives solely behind an emission gate.

## Smallest repair contract

Keep all committed ACK architecture and limits unchanged.

1. Add a deterministic ACK-tracker quiesce/reset operation that releases `ranges`, `largest_observed`, and `pending_ack` for the terminal generation. Replacing the tracker with a fresh empty tracker for the same generation is also acceptable if it is the smallest implementation shape.
2. Call it from `ReliableUdpRuntime::teardown()` together with the existing sender-side cleanup.
3. Add a discriminating regression that creates more than one observed ACK range and a pending ACK before teardown, then proves after teardown that:
   - receiver ACK observations/ranges are empty;
   - `largest_observed`/equivalent observation state is absent;
   - `pending_ack` is false;
   - `poll_outgoing_ack()` remains `None`;
   - sender-side `in_flight`, Reno bytes and retained plaintext remain zero;
   - preserved lifetime Recovery diagnostics remain preserved exactly as H-R9-066 requires.
4. Do not add a production-only inspection API merely for the test if a crate-private/test-only oracle or direct `PacketAckTracker` invariant can establish the cleanup cleanly.
5. After the repair, run the focused carrier tests and the required clean exact-tree developer gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, with exact SHA/times/exit codes/OS-arch/stable Rust persisted. No wire decoder/parser/crypto framing change is required by this repair, so fuzz is not mechanically required.

## R9-11 remainder after H-R9-077

Once this HIGH is closed, continue the same R9-11 bounded review rather than declaring lifecycle closure immediately. In particular, still challenge:

- `SessionRuntime` remote-close/cancel/idle/close-deadline cleanup and post-terminal mutation;
- executable automatic-health failover/migration-back socket/process cleanup;
- failed/retired Carrier generation non-reactivation;
- unresolved `FailoverController`/`ConcurrentCarrierManager` retained Session replay ownership across Carrier/path termination;
- service lifecycle one-way terminal behavior and cleanup oracles;
- success/failure/timeout/shutdown/post-return process result truth.

`READY_LIVE` remains `none`; this finding is entirely local and creates no new real-network question.
