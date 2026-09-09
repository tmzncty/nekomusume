# M0 candidate wire golden corpus

**Status: legacy fuzz-seed audit asset, distinct from the frozen canonical vector corpus v1.**

`fuzz/corpus/decode/` contains 22 binary seed files corresponding to the 22
candidate record shapes previously exercised by `neko-wire`'s embedded golden
round-trip test. Each file is named with its record kind and payload length.
The nine-byte header is `NK`, version `0`, type, zero flags, and a big-endian
`u32` payload length. This directory is mutable seed input for `cargo fuzz`; it
does not replace deterministic tests and carries no freeze claim. The 4096-byte
entry is intentionally retained to exercise the current payload limit.

The separate [`../../fixtures/canonical-vectors.v1.json`](../../fixtures/canonical-vectors.v1.json)
is the content-addressed frozen canonical corpus v1. Its 42 rows cover ten
required domains and execute declared real implementation oracles. That freeze
is corpus-specific: it does not make this fuzz-seed directory immutable, freeze
the entire protocol, or grant interoperability, release, security, or
production approval. See
[`canonical-vector-corpus-scope.md`](canonical-vector-corpus-scope.md).


## Boundary evidence update

The wire test suite now exercises every input length from zero through the
short-prefix boundary and explicit overflowing/non-canonical varint cases. The
fuzz target additionally asserts that decoding never panics for arbitrary bytes;
its return value remains ignored because malformed input is expected. Fuzz smoke
artifacts are disposable and must not be committed as protocol vectors.


## Smoke-run isolation

`./scripts/fuzz-smoke.sh` copies the tracked seed corpus into a unique temporary
directory and directs libFuzzer discoveries and crash artifacts there. An
EXIT/INT/TERM trap removes the temporary tree. Recurring smoke verification
therefore cannot add, rewrite, or delete repository corpus vectors.
`./scripts/fuzz-smoke-test.sh` executes a short smoke run and asserts that the
complete porcelain worktree status is byte-for-byte unchanged.
