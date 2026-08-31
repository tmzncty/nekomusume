# Canonical protocol vector corpus v1 (N9 frozen corpus)

This is the **frozen canonical conformance corpus v1**, not a repository-wide protocol freeze. The
authoritative fixture is `fixtures/canonical-vectors.v1.json`; its schema is
`schema/canonical-vector.v1.json` and the structural gate is
`scripts/validate-canonical-vectors.py`.

## Vector contract

Each vector has a stable `id`, one `domain` and `operation`, typed JSON
`input`, exact lowercase `bytes_hex`, an `expected` success value or stable
error code, classifications, and three mandatory oracle assertions:

1. `encode_equals_bytes`: implementation encoding is byte-for-byte equal to
   `bytes_hex`.
2. `decode_bytes_equals_expected`: decoding exactly `bytes_hex` equals
   `expected.value` (or the named deterministic error).
3. `roundtrip_equals_bytes`: encoding the decoded value reproduces the exact
   original bytes, including canonical integer representation.

The validator rejects missing/false oracle assertions. It is intentionally
conservative: the JSON gate proves that a vector declares all three checks;
an execution adapter must set them only after running the corresponding
implementation calls. No unchecked vector is conformance evidence.

## Coverage

The initial bounded corpus includes negotiation, outer wire records, frames,
ACK/ranges, reliable UDP packet boundaries, unreliable datagrams, synchronized
key phase, carrier/path transitions, close, and error handling. It includes
malformed, truncated, trailing, oversized, unknown enum/version,
unauthenticated, range, duplicate/late, noncanonical integer, overflow, and
minimum/maximum integer classifications.

All lengths, IDs, bytes, vector count, error names, and enums are bounded by the
JSON Schema and executable validator. Corpus provenance is content-addressed:
`schema_revision` selects the identity algorithm and `corpus_sha256` is
recomputed over deterministic JSON content with only the hash field excluded.
It is intentionally independent of mutable branch or HEAD names. The validator
also checks a fixed required-domain set, including `close` and `error`, rather
than deriving coverage requirements from fixture contents. `freeze` is required
to remain `true`; the validator and generator reject a reversion to `false`, and
the content identity rejects stale or silently changed rows.

## Freeze boundary

`freeze=true` freezes exactly this 42-vector, 10-domain corpus identity and its represented bytes and semantics. It does not freeze Noise transcripts, cryptographic ciphertext, carrier packetization, failover/resume behavior, or the global protocol/release state. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, repository-wide `FREEZE=false`, and `RELEASED=false` remain unchanged in `docs/status.md`.
