# Local wire/frozen-corpus bounded review — `525b61b`

Developer-run bounded review and execution against reachable exact tree `525b61be78c628fd153164d514401305c960b460`.

## Scope and result

The candidate v1 wire contract was checked against the separate frozen canonical corpus and the mutable legacy fuzz-seed boundary.

- frozen corpus identity remains exactly 42 vectors across 10 required domains with `freeze=true`
- the generated review artifact matches the fixture and every declared oracle retains a real implementation adapter
- `neko-wire` golden, malformed, short-prefix, varint canonicality/overflow, frame, datagram, negotiation, canonical-vector, deterministic property, and compatibility tests pass
- Session readiness fuzz seeds remain exact process messages
- one isolated 30-second decode fuzz smoke completed 9,262,082 executions without a crash; discoveries/artifacts remained temporary
- no contradiction was found between the candidate outer record/frame/negotiation text and the represented frozen corpus

The compatibility boundary remains explicit: corpus v1 freezes only its represented bytes/semantics and content identity. It does not freeze Noise transcripts, ciphertext, carrier packetization, failover/resume, the mutable fuzz corpus, or the global protocol; there is still no previous frozen release for previous/current interoperability.

## Commands

An initial invocation used two nonexistent script names and exited `2` before validation; it is not test evidence. The corrected bounded run was:

- `python3 scripts/validate-canonical-vectors.py fixtures/canonical-vectors.v1.json`: passed
- `python3 scripts/validate-canonical-vectors-test.py`: passed
- `python3 scripts/generate-canonical-review.py --check`: passed
- `python3 scripts/generate-canonical-review-test.py`: passed
- `cargo test -p neko-wire --all-targets`: passed
- `cargo test -p neko-session --test readiness_fuzz_seeds`: passed
- `bash scripts/fuzz-smoke.sh`: passed, 30-second bound
- `git diff --check`: passed

Run completed on `2026-09-10T17:18:13Z`–`2026-09-10T17:19:25Z` (bounded from command invocation records), host `Linux 6.8.0-137-generic x86_64`, Rust `1.98.0`. The source tree remained unchanged; a Python cache directory created by direct validator execution was removed.

This is developer-local candidate/fuzz evidence, not independent review, repository-wide freeze, interoperability approval, RC, release, public-listener, or production authorization.
