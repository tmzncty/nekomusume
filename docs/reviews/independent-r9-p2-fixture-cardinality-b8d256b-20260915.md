# Independent R9 P2 fixture-cardinality review — exact `b8d256b`

**Reviewed tree:** `b8d256b4d541bc887ef16a9081c4e3941463c074` (docs-only descendant of latest developer source/test exact `422c16b106b5568cb87f81419b45941508c2d4a4`).

**Scope:** static bounded review of the current `--reliable-udp + --automatic-health-failover + --migration-back` P2 process fixture and its ownership partition. This is not a WAN run, performance result, security approval, release decision, or protocol/wire change.

## Verdict

**H-R9-018 — HIGH for R9-2H acceptance evidence: current P2 `count=3` fixture is deterministically unable to reach UDP recovery/migration-back under the accepted ownership partition.**

The source/test tree itself is green under hosted `stable checks` and `nightly decode fuzz smoke`; this finding is a semantic reachability contradiction in the P2 test configuration, not a red-tree or permission blocker.

## Deterministic derivation

The accepted H-R9-017 ownership partition is:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = uncertain_end - uncertain_start
```

For the current P2 fixture:

```text
count = 3
reliable_udp = true
recovery_enabled/migration_back = true

uncertain_start = 2
uncertain_end   = 2
controlled_tcp_replay = 0
```

Therefore the client executes zero iterations of the TCP replay loop. `tcp_active_sample` starts as `None` and is populated only inside that loop after a successful authenticated TCP application DeliveryAck. Immediately before sending the UDP recovery challenge, the recovery branch requires:

```text
let tcp_active_sample = tcp_active_sample
    .unwrap_or_else(|| fail("missing authenticated TCP application RTT sample"));
```

So the current `count=3` P2 process can never reach the nominal `udp_recovery_challenge_sent -> udp_recovery_validated -> migration-back -> post-return reliable Data` chain. Strengthening the test to require success without first correcting this fixture cardinality would only expose this deterministic precondition failure.

This also explains why the earlier conditional P2 evidence could remain vacuous without proving a missing UDP recovery transition.

## Smallest semantics-preserving repair

Do **not** weaken H-R9-017, invent a retry/sleep/deadline, or derive active TCP health from a new evidence domain.

Change the dedicated P2 migration-back fixture to `count=4`. Under the already accepted partition:

```text
records[0] offset 0   -> reliable UDP owned
records[1] offset 16  -> reliable UDP owned
records[2] offset 32  -> one genuine uncertain range replayed over TCP
records[3] offset 48  -> reserved for post-migration return to reliable UDP

uncertain_start = 2
uncertain_end   = 3
controlled_tcp_replay = 1
```

That one authenticated TCP replay supplies the existing `tcp_active_sample` without changing runtime semantics. The final reserved-record assertions must move from offset `32` to offset `48`.

The P2 test should then require both processes to exit successfully and prove the exact causal chain already described in `docs/CHATGPT_HANDOFF.md`, including the dedicated post-return terminal evidence point and both acknowledgement domains. The negative ownership assertion becomes: offset 48 must never appear in pre-promotion `udp_uncertain_range_sent` or TCP replay evidence.

If `count=4` still fails, classify the first missing structured milestone and repair only that exact transition. Do not restore the reverted recovery-challenge retry experiment.

## Evidence boundary

No developer-local gate was executed by this reviewer. Hosted checks on exact `b8d256b` are cross-evidence only and are green. No source/test modification is made by this review note. `READY_LIVE` remains none; R9-3 remains blocked on positive P2 plus P3/P4 and clean exact-tree provenance.
