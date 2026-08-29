# M0 candidate wire golden corpus

**Status: candidate audit asset, not a frozen normative vector set.**

`fuzz/corpus/decode/` contains 22 binary seed files corresponding to the 22
candidate record shapes previously exercised by `neko-wire`'s embedded golden
round-trip test. Each file is named with its record kind and payload length.
The nine-byte header is `NK`, version `0`, type, zero flags, and a big-endian
`u32` payload length. The corpus is seed input for `cargo fuzz`; it does not
replace the deterministic unit test or claim that the candidate format is
frozen. The 4096-byte entry is intentionally retained to exercise the current
payload limit.


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
