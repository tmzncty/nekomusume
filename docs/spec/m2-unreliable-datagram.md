# Authenticated unreliable datagram candidate

Bounded 1200-byte datagrams use authenticated Noise record context, nonce and replay state. The API has no retransmission, delivery ACK, Session-ledger promotion, or carrier-ACK promotion. Loss is caller-visible absence. Malformed, oversize, tampered and replayed records fail closed. This is local research only: no 0-RTT, public service, WAN or protocol freeze.
