# R-CLI/BND-DIFF — CLI contract / process portability / algorithmic boundedness owner-diff review

**Exact source/control-flow anchor:** `33b6a3c907ba0d37913def5cc98fd1402900bfec`.

**Review class:** bounded independent GitHub owner-diff/source review only. No reviewer-local Rust/check gate, hosted CI, non-Linux execution, WAN, fuzz, or performance execution is claimed.

## Reuse baselines

- current cross-platform/process evidence boundary: `416fd5e60f0b90155acf65ad33927d7accaecaf6` review note;
- current process-ownership sweep: `fcd7a08` review chain after H-I4-097..115;
- current algorithmic/resource boundedness reconciliation: `934c878164c3d417d5045672aeabef07d4235db2`;
- existing CLI machine/human output contract reviews, including the stable schema/exit/human-output split retained by the item-4 packet.

## Invariants challenged

1. No post-review CLI source/test drift may invalidate the exit-code / JSON / human-output contract or the bounded child/process ownership repairs.
2. Linux-scoped evidence must not be promoted to Windows/macOS/BSD execution support.
3. Post-`934c878` recovery/carrier repairs must not introduce a new unbounded retained collection, unbounded input-driven loop, or an invented capacity policy.
4. The known `SessionRuntime.events` retained-history bound remains a policy/value gate; no reviewer-selected history size is permitted.

## Owner-diff result

**No new dependency-ready finding in this bounded lane.**

- `934c878..33b6a3c` contains **no changes** to `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, `crates/neko-cli/tests/multistream.rs`, `crates/neko-session/src/lib.rs`, CLI manifests, or process/package scripts. The exact-current CLI output/process semantic owners therefore remain those already challenged by the 2026-09-24 process/cross-platform reviews.
- The current evidence boundary remains Linux-scoped: Bash gates/package scripts, Linux package target evidence, Unix-only descendant-pipe regression, signal handling and procfs-specific resource observation are not Windows/macOS runtime proof. No authoritative current file is promoted here into such a claim.
- The only post-`934c878` implementation changes are in `neko-carrier` / `neko-reliable` recovery-related owners already independently challenged as H-I4-116 (zero-loss Reno no-op), H-I4-117 (persistent-congestion wiring), H-I4-118 (stale-ACK loss eligibility), plus CarrierState/manager no-finding review support. Those repairs add no new retained collection or input-proportional unbounded iteration surface; their state remains under the pre-existing packet/history/maps and committed bounded algorithms.
- The existing `SessionRuntime.events` lifetime history question remains exactly `POLICY_BLOCKED_RESOURCE_BOUND`. This review does not choose TTL/LRU/history-size/capacity/security values and does not reinterpret D019.
- Because the CLI semantic owners have not moved since the current process/cross-platform challenge, there is no basis to reopen closed H-I4-097..115 or the machine/human output contract merely because review documentation advanced.

## Evidence boundaries / exclusions

- Historical exact-tree gates remain attached to their own SHAs and are not reattributed to `33b6a3c`.
- No capacity-pressure/adversarial-load benchmark was run or selected.
- No target-support, signal/process-group, output-schema, release-package, or service-policy decision is made.
- No decoder/parser/crypto-framing code moved in this lane; fuzz is not requested.
- No new real-network question arises. **`READY_LIVE: none`.**

## Classification

`R-CLI/BND-DIFF: CLOSED — bounded no-finding / exact-current semantic owners reuse the current process/output/boundedness reviews; existing policy gate preserved.`

After this fourth coherent reuse/refill slice, perform one grouped release/item-4 factual reconciliation rather than rewriting the packet after every note. Release items 3/4 remain incomplete.
