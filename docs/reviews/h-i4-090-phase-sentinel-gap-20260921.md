# H-I4-090 follow-up — phase sentinel still permits post-churn baseline

**Severity:** HIGH (evidence-oracle closure; no new runtime transport defect observed)

**Reviewed anchor:** current `main` at `390be04e54e854d9670f967453ada333aa4b0d36`.

**Current source/test owner:** `de223a5128223d3667961e772002dcc6d75c563c` (`crates/neko-cli/tests/probe.rs`).

## What de223a5 fixed

The test now has an explicit `churn_phase` sentinel around the malformed-UDP resource observation, and developer-reported exact-tree provenance is retained in `docs/notes/h-i4-090-provenance-de223a5-20260921.md`. The actual Linux measurement order on the committed tree remains READY -> affirmative pre-churn `/proc` snapshot -> malformed sends -> settle -> affirmative post snapshot. H-I4-089's Linux-only `/proc` boundary remains intact.

## Remaining mutation-oracle defect

The sentinel changes to phase `2` only **after all malformed `send_to` calls have already completed**:

```rust
let mut churn_phase = 0u8;
let before = {
    assert_eq!(churn_phase, 0, "baseline must precede the churn");
    process_resource_snapshot(pid)
};
...
for sender in &senders {
    sender.send_to(&malformed, ("127.0.0.1", udp)).unwrap();
}
churn_phase = 2;
```

Therefore the exact regression mutation this finding is meant to detect can still stay green: move the `before` block to immediately after the `for sender in &senders { ... send_to ... }` loop but before `churn_phase = 2`. At that point every malformed datagram has already been sent, yet `churn_phase` is still `0`, so the baseline assertion passes. The test again compares a post-churn baseline with a later post-churn sample and can hide persistent FD/RSS growth.

The later `assert_eq!(churn_phase, 2)` only proves the assignment occurred; it does not prove the baseline preceded the first malformed send. Thus `de223a5` does not yet satisfy the prior closure contract: moving the baseline after malformed-send start must fail even when FD/RSS values themselves do not expose the ordering mistake.

## Minimal closure contract

1. Keep the current actual measurement order and all existing FD/RSS margins and Linux-only `/proc` policy.
2. Make the sentinel transition observable **before the first malformed `send_to` can occur** (for example, enter a `churn_started` phase immediately before the send loop, or use an equally small test-local helper whose first action changes the phase before issuing any send). The baseline assertion must require the pre-churn phase.
3. Preserve an explicit post-send/post-settle phase if useful, but do not build a general sequencing framework/checker.
4. Add/retain a mutation-sensitive guard such that cut/pasting the baseline block to immediately after the malformed-send loop fails deterministically even if FD/RSS are unchanged.
5. Run the focused affected test(s), then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify a clean tree, and persist exact developer-local provenance (SHA, UTC start/end, exit codes, OS/arch, Rust stable, clean state).
6. No decoder/parser/crypto-framing change is involved, so no mechanical decode fuzz run is required.

After the repair, continue immediately with the already-planned bounded independent I4-PORT-RES re-challenge, PORT evidence reconciliation, release-packet factual consistency challenge, and repository-wide 13-surface refill. Do not wait for the next reviewer cadence.

`READY_LIVE: none`; release items 3/4 and all release/governance flags remain unchanged.
