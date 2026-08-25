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
