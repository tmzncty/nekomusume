# M3 bounded TCP fallback candidate

**Status: loopback research candidate; not a production tunnel or public listener.**

TCP records use a four-byte big-endian length followed by one bounded frame.
Only ephemeral `127.0.0.1` connections are constructed. TCP provides reliable,
ordered bytes, so the carrier capability explicitly disables packet-feedback
ACK machinery; Session `DataId` delivery remains separate.

`FailoverController` starts with UDP primary. A configurable count of consecutive
PTO observations is the deterministic hard-failure gate; one PTO alone is only
health evidence. On failure, bounded uncertain `(DataId, bytes)` entries are
resent over TCP. The receiver accepts a DataId once, treats exact duplicate bytes
as idempotent, and rejects conflicting reuse. Metrics count switches, recovery
events, unique delivered bytes and duplicate bytes.

The integration test models a UDP blackhole, moves the same authenticated Noise
Session data to TCP, proves no final byte loss, and sends a fresh-AEAD duplicate
with the same DataId to prove logical deduplication. TCP never gains a duplicate
packet ACK layer.

Migration back to UDP is deliberately not automatic in this slice. It requires
a newly validated UDP path generation plus Carrier Manager hysteresis and a
minimum hold period; the active TCP path remains stable until that later gate.


## Migration-back gate

Migration from active TCP to UDP is a separate atomic Carrier Manager
transition. The candidate must carry the same path generation as the active
TCP session, explicit path-challenge validation, a healthy sample, and a score
exceeding the active path by the configured margin. A minimum hold count is
required before commit. Old/mismatched generations, unvalidated, unhealthy,
low-margin, and early candidates do not mutate active path, switch metrics, or
Session evidence.


## Uncertain-state edge audit

Uncertain entries are keyed by `DataId`; exact duplicate tracking is idempotent,
conflicting payloads are rejected, and failed capacity/confirmation operations
do not mutate the ledger. Counter increments saturate, and recovery latency uses
checked subtraction so timestamps near `u64::MAX` remain deterministic.
