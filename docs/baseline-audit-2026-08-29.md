# Baseline audit — 2026-08-29

## Scope and provenance

This audit was performed on `tmzn-server` from `/media/tmzn/DATA5/nekomusume-research/repo`, without changing network services or configuration. It records repository facts at historical pre-implementation commit `0fbc68d4b28c9032f86e1bf46c5f2b983194f453`; this document is a historical baseline, not the current status snapshot on `candidate/g0-governance-status-repair6`.

## Verification

- `./scripts/check.sh`: passed (`cargo fmt`, workspace check, tests, clippy, governance/status/license checks).
- `FUZZ_TIME=5 FUZZ_MAX_LEN=1024 ./scripts/fuzz-smoke.sh`: passed; nightly `cargo-fuzz` decode target completed 3,493,123 executions with no crash or artifact.
- CLI invocation for the root and `client`, `server`, and `probe` subcommands: runs the documented M0 scaffold message; no transport is started.
- Git worktree was clean before and after the audit.

## Implemented evidence

- Four-crate Rust workspace: `neko-wire`, `neko-session`, `neko-carrier`, `neko-cli`.
- `neko-wire` has bounded candidate record encode/decode, deterministic fixed header, canonical varint helpers, malformed-input tests, and 20+ golden record vectors.
- `neko-session` has a pure in-memory delivery ledger with bounded streams/bytes/reorder/offsets, context and epoch checks, overlap/conflict handling, and state tests.
- `neko-carrier` has opaque bounded `MemoryPair`, loopback UDP carrier, path-generation/validation state, hysteresis, and evidence-separation tests.
- Carrier framing integration tests connect the candidate wire codec to memory and loopback UDP without adding production crypto or tunnel behavior.
- Decode fuzz target and corpus exist under `fuzz/`.

## Not implemented or not approved

- No authenticated handshake, AEAD, key management, trust store, authorization, or cryptographic dependency is selected or implemented.
- No production or public service exists. Candidate local/loopback slices now exist for encrypted UDP echo, TCP framing/failover, bounded reliable recovery, multi-stream scheduling, Carrier Manager, PLPMTUD, unreliable datagrams, key update, and isolated benchmark harness; these remain research candidates and do not constitute production deployment.
- G0 authorization was later amended for bounded local/loopback research, but G0 is not security-approved; Noise, pre-auth, key update, FEC, PLPMTUD, and multipath documents remain candidate gates. The provisional v0 document is not a frozen protocol.
- Reachability experiments and public-network exposure remain blocked and were not run.

## Recommended next executable slice

The earliest safe code slice after this audit is to improve the CLI scaffold with explicit, deterministic `--help` and error exit-code behavior while keeping it transport-free. Any cryptographic or public-network work must stop until the named G0/G2 review gates are resolved and recorded.
