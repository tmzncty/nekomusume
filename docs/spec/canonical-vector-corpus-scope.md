# Candidate canonical vector corpus scope

`fixtures/canonical-vectors.v1.json` is the machine-readable candidate wire corpus. It is deliberately `freeze=false`: these rows are review evidence, not yet a release, security approval, or compatibility freeze. `parent_commit` identifies the exact implementation parent against which the candidate was prepared.

## Executed wire coverage

Every non-`state_only` row carries real `bytes_hex`. The Rust adapter executes every oracle marked `true`: encoders compare emitted bytes, decoders consume the fixture bytes and compare values/errors, and round trips decode then re-encode (or perform the equivalent complete negotiation exchange) byte-exactly.

The candidate deliberately covers:

- negotiation hello, selected-version response, no overlap, duplicate offers, malformed responses, and unsupported selections;
- all outer record kinds (`Data`, `Ack`, `PathChallenge`), plus unknown version, truncation and trailing bytes;
- all current frame variants (`Data`, `Datagram`, `DeliveryAck`, `Close`, `PathChallenge`, `PathResponse`), unknown ignorable retention, unknown critical rejection, and reserved-type rejection;
- frame truncation, the 1024-byte frame payload maximum, oversized declared length, exactly 64 frames, and 65-frame rejection;
- canonical varint boundaries `0`, `1`, `127`, `128`, `16384`, and `u64::MAX`, plus non-canonical, truncated, and overflow failures.

`frame.datagram-small` is intentionally a small one-byte vector; `frame.datagram-max-1024` is the real maximum payload vector. `close.empty` is a real executed frame vector. Failure vectors preserve implementation semantics and are not rewritten into successes.

## Deliberate exclusions

The corpus freezes only public `neko-wire` record/frame/negotiation bytes. ACK-range state, reliable packet-number exhaustion, unreliable-datagram API policy, key-update state, and carrier-transition state are conceptual contracts in other crates rather than codecs in this wire layer. Their rows therefore use `bytes_hex: null`, all byte oracles are false, and `state_only` makes their non-wire status explicit.

Cryptographic ciphertext, Noise messages, carrier packetization, failover/resume state, and previous-release interoperability are excluded. Ciphertext is key/nonce dependent; carrier and Session state have separate executable tests; no previous frozen release exists. Empty frame lists, invalid flags/magic/type, record payload maximum, path-frame invalid length, and additional malformed permutations remain covered by ordinary unit/fuzz tests rather than compatibility vectors. These exclusions do not waive parser safety requirements and may be revisited before a future independent freeze decision.
