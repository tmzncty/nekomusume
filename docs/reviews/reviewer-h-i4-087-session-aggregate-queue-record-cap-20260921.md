# Reviewer HIGH H-I4-087 — SessionRuntime aggregate queued-record cap is enforced per direction

**Reviewed current tree:** reachable `main` at `6b24a1671ea3645f8bffe68ea86988b4897e0030` before this note.

**Developer commits reviewed:** `da1314b`, `ca629d7`, `3acba04`, and the later handoff/provenance/navigation commits through `6b24a16`.

**Classification:** correctness/security resource-limit HIGH for release-item-4 review support. This is a bounded local source-decided defect; it does not require a maintainer policy choice.

## H-I4-086 closure accepted

The ACK-before-drain defect is repaired by `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` plus companion tests at `3acba049fee5f9297cc8a4c3867250635c255717`:

- `queue_send` continues to reserve stream/session send-window credit;
- `pop_send` advances per-stream drained/sent ownership;
- `delivery_ack` rejects `end > sent` before releasing any credit or confirmation evidence;
- tests cover ACK-before-drain, a mixed drained/queued prefix, cross-stream drain isolation, resumed-session ordering, and terminal cleanup of the new bookkeeping;
- developer-reported clean exact-tree provenance for `3acba04` is retained in `docs/notes/h-i4-086-provenance-3acba04-20260921.md`.

No reviewer-local execution is claimed here. No hosted CI run was visible for exact `3acba04`; absence of hosted evidence is not a failure.

## New counterexample — configured `max_queue_records` can be exceeded

`SessionRuntime` exposes one runtime-level `RuntimeLimits::max_queue_records` and `queued_records()` reports the aggregate `send.len() + recv.len()`. However the two admission paths enforce the limit independently:

- `queue_send` rejects only when `self.send.len() >= max_queue_records`;
- `receive` rejects only when `self.recv.len() >= max_queue_records`.

Therefore the configured record cap is not an aggregate cap over the runtime-owned queued-record surface.

A deterministic minimal counterexample with byte/window limits chosen large enough not to interfere:

1. set `max_queue_records = 1`;
2. open one stream;
3. `queue_send(stream, b"a", ...)` succeeds, so `send.len() == 1`;
4. `receive(InboundRecord { same stream, offset: 0, data: b"b" }, ...)` also succeeds because `recv.len() == 0` at its own check;
5. now `queued_records() == 2` while the configured `max_queue_records == 1`.

The reverse order (`receive` first, then `queue_send`) has the same defect. This contradicts the current runtime-level resource-limit surface and the existing independent/developer review claims that `max_queue_records` is enforced before mutation. It is bounded to at most roughly two directional queues, but it still defeats the configured per-Session record cap and therefore cannot remain a truthful release/security-review closure.

## Required closure contract

Use the smallest repair compatible with current semantics; do not invent a second policy value or separate send/recv cap.

1. Both `queue_send` and `receive` must reject before mutation when the aggregate queued-record count would exceed `max_queue_records`.
2. Add deterministic negatives for both admission orders:
   - send fills the single aggregate slot, then receive fails closed;
   - receive fills the single aggregate slot, then send fails closed.
3. Assert rejection does not change queues, `queued_bytes`, send/receive window accounting, offsets/watermarks, or event state beyond behavior already explicitly documented for the rejected operation.
4. Add a positive slot-release control: after `pop_send` or `pop_receive` removes one queued record, the other direction may consume the freed aggregate record slot if all other limits permit it.
5. Keep existing per-stream/session byte-window behavior unchanged. `pop_send` must still not release delivery credit; `pop_receive` must still release receive-window credit as currently defined.
6. No decoder/parser/crypto-framing change is involved; do not mechanically run fuzz.
7. After repair, run focused deterministic tests and the final pushed SHA clean exact-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist provenance.

## Queue impact

Front H-I4-087 before continuing the I4-FS2 remainder. Once closed, continue the existing deep queue: FS2 remainder -> MemoryCarrier -> UDP/TCP adapters -> cross-platform process/CLI -> machine-output contract -> human-output contract -> boundedness -> item-4 factual reconciliation -> repository-wide refill -> conditional live.

`READY_LIVE: none`; release items 3/4 remain incomplete. Governance flags remain unchanged.
