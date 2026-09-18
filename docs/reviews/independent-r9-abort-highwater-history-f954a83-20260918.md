# Independent bounded R9 abort/high-water review — exact `f954a83`

**Classification:** `HIGH` — H-R9-053 Recovery/ACK-history correctness. R9-3 remains blocked.

**Reviewed developer source/test anchor:** exact reachable pushed SHA `f954a83a5556e77f50871410dc481e5b83d7bbac`, on top of `4488d09a79ed150f4a1c7d732f4f0ecffedfb381`.

**Current reviewer anchor before this note:** `dcb4fb150162660f7b810ff613fe02394250b9ee`.

## Accepted progress

The H-R9-051 transaction work is materially useful and must be retained: the executable retransmit callers check the socket result; a socket error routes through the complete `abandon_retransmit` owner, reverses Recovery/Reno/packet-frame ownership, does not emit positive send evidence, and does not advance caller packet ownership. The partial rollback bypass was removed and `packets_sent` is rolled back.

The new regression also proves the narrow case where the previous committed packet is still present in `Recovery.sent`: after reserving a higher packet and aborting it, the aborted packet number is no longer ACK-valid.

## H-R9-053 finding — rollback reconstructs committed history from the in-flight map

`Recovery::largest_sent` is not merely the largest packet currently in flight. `on_ack` explicitly treats it as the largest packet the sender has ever successfully sent, so an ACK for a genuinely sent-and-already-retired packet may remain a legitimate late/duplicate accepted-empty observation, while `on_sent` uses the same watermark to prevent packet-number regression/reuse.

Exact `4488d09` changes `Recovery::abandon_sent` so that, when the aborted reservation equals `largest_sent`, it restores the watermark using:

```rust
self.largest_sent = self.sent.keys().next_back().copied();
```

That is not the previous committed high-water. It is only the largest packet still present in the outstanding `sent` map. ACK/loss processing removes committed packets from that map while intentionally leaving their send history represented by `largest_sent`.

Therefore a normal sequence can corrupt committed history without any WAN behavior:

1. successfully send packet `N`;
2. ACK/retire `N`, so `N` is no longer in `sent` but `largest_sent == N`;
3. reserve higher packet `M` for a retransmit;
4. socket send fails and `abandon_sent(M)` executes;
5. current code sets `largest_sent` to the largest *currently outstanding* packet, or `None` if none remain, rather than restoring `N`.

Two established invariants are then violated:

- a legitimate late/duplicate ACK whose largest is `N` can now be rejected as future/never-sent even though `N` really was handed to the socket;
- `Recovery::on_sent` can accept a packet number at or below the previously committed historical high-water after rollback, weakening the monotonic packet-number / nonce-identity guard.

This is the exact failure mode the previous contract warned against when it required abort rollback to restore the **previous committed high-water** and explicitly preserve late/duplicate ACK semantics for packets that were actually sent and later retired. Reconstructing that value from `sent` cannot satisfy the contract.

## Required discriminator

Add a focused deterministic test that does **not** leave the previous high-water outstanding:

1. send committed packet `N`;
2. ACK/retire `N` completely;
3. reserve higher packet `M` and abort it as a socket-send failure;
4. verify a duplicate/late ACK for `N` is still accepted under the existing empty/late semantics and does not mutate unrelated RTT/PTO/loss/cwnd/Session state;
5. verify ACK for aborted `M` is rejected atomically;
6. verify `on_sent` still rejects packet numbers `<= N`, while retrying a valid packet number `> N` remains possible.

A variant with another older packet still outstanding should also prove the rollback never lowers the committed watermark to `max(sent.keys())` and thereby enables historical packet-number reuse.

## Smallest repair contract

Preserve H-R9-050 exact-wire admission and H-R9-051 socket-result transaction semantics. Do not require ACK membership in the current `sent` map, do not add an unbounded abandoned-PN history, and do not invent TTL/LRU/capacity policy.

Use a bounded reservation/commit/abort shape, or equivalent local state, that can restore the actual pre-reservation committed watermark on abort even when the prior committed packet has already been ACKed/lost and removed from `sent`. The reservation state must remain local/bounded and must not change Session/Carrier ACK architecture, crypto framing, or wire format.

After repair, run focused tests plus the required clean exact-tree developer-local gate and persist exact provenance. No decoder/framing change is required by this finding, so fuzz is not mechanical unless the implementation touches those surfaces.

## CI/evidence boundary

The developer-reported local exact-tree gate for `f954a83` is recorded in the handoff. Separately, GitHub-hosted Rust CI run `35323003081` for the same SHA failed in `scripts/check.sh` at `cargo fmt --check` under rustc/rustfmt 1.98.1; the nightly decode fuzz-smoke job passed. The docs-only descendant `dcb4fb` also has a failed hosted Rust CI run (`35323142232`) because it inherits the same source tree. Hosted CI is cross-evidence, not the primary local gate, but the current branch is objectively not green under the hosted stable toolchain and the formatting delta should be repaired in the next source/test slice rather than described as green.

No live/WAN experiment was run. `READY_LIVE` remains `none`; this is deterministic local Recovery/evidence correctness. D019, capacity/security policy values, signing/SBOM/key custody, core Session/Carrier/ACK/crypto/wire architecture, destructive migration, and release/freeze/production authority are untouched.
