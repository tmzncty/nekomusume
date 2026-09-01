# Reviewer Follow-up C — equal-application HY2 prerequisite

**Parent:** `c7e0a211cbc74f065d03b374bd3cc1bbf2a97356`
**Scope:** bounded prerequisite only; no comparison result or superiority claim.

## Feasibility decision

The pinned Hysteria2 v2.9.3 forwarding seam can carry the same application
question as Nekomusume: send one exact payload and receive the identical bytes.
Doing so does not require disabling authentication, encryption, admission, or
any resource guard. The generic authenticated TCP path was missing only a way
to consume and attest an externally fixed payload; it always generated `x`
bytes internally. That exact code blocker is closed by this slice.

Nekomusume now reads one regular payload file through a 1201-byte bounded reader,
requires its size to equal `--bytes`, permits the mode only for authenticated
TCP / one record / JSON, verifies the authenticated echo byte-for-byte, and
reports the SHA-256. The HY2-side `echo-payload.py` adapter performs one finite
exact-byte exchange through a caller-owned TCP forwarding listener and reports
the same byte/hash/FD fields. The local methodology validator rejects a
right-length/wrong-hash result. No wire or parser behavior changed.

## Current execution status

The historical missing-command-channel blocker was superseded by the established
second self-owned path. Reviewer `3978f3f` subsequently implemented the separate
owned-lab orchestrator and reached a disposable HY2 server, but temporary HY2 UDP
connectivity timed out before forwarding readiness. See
`reviewer-3978f3f-followup-c-hy2-owned-lab-20260902.md`. No paired sample or
performance claim exists.
