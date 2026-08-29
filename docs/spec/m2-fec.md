# Bounded XOR FEC candidate

**Status: candidate carrier optimization; not enabled by default.**

The isolated evidence currently proves deterministic recovery under 1/5/10%
first-send loss and a netem matrix, but does not demonstrate that FEC improves
any target metric. Therefore this slice records and tests only a bounded
systematic XOR block candidate; it is not a selection or performance claim.

A block contains 2–32 equal-size data symbols and one parity symbol. One missing
data symbol can be reconstructed by XOR. Two or more missing symbols fail as
`Unrecoverable`; no guessed or zero-filled bytes are produced. Configuration and
symbol sizes are bounded. Duplicate/index errors are explicit. Reordering and
loss scheduling are outside the block identity and do not affect recovery.

FEC remains separate from reliable recovery: parity does not acknowledge
Session delivery, replace retransmission, alter packet ACK evidence, or bypass
congestion control. No wire codec, negotiation, socket, runtime, WAN or
production behavior is introduced.


## Block identity bound

`max_blocks` bounds the block identity space: valid IDs are `0..=max_blocks`,
with the next ID rejected as `TooManyBlocks`. The identity check occurs
before symbol cloning or parity allocation, so an out-of-range request produces
no block state. This is a local candidate invariant; it introduces no wire field
or runtime block table.
