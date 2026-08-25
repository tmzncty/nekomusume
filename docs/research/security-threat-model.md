# Security Threat Model and Candidate Handshakes

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**
**Status: research and gates, not a security audit or cryptographic decision.**

## Assets and attacker

Assets are session identity, authenticated payload/control state, key material, delivery state, availability, and endpoint privacy. The network attacker can observe, delay, drop, reorder, duplicate, inject, replay and steer packets between candidate paths; it may induce NAT rebinding and resource exhaustion. It cannot break selected primitives or compromise endpoints in the baseline model. Endpoint compromise, malicious peer authorization and traffic-analysis resistance are outside the baseline and must not be implied.

Trust boundaries are the peer authentication boundary, the Session/Carrier boundary, and the application/transport boundary. An unauthenticated carrier is never proof of peer identity.

## Candidate comparison

| Candidate | Strength | Cost/risk | M0 position |
|---|---|---|---|
| TLS 1.3 over a custom record carrier ([RFC 8446](https://www.rfc-editor.org/rfc/rfc8446) §§5.2–5.5,7–8) | Mature handshake, transcript/key schedule, peer authentication options | Record integration, identity policy and resumption must be specified | Candidate only |
| Noise Framework ([noiseprotocol.org/noise.html](https://noiseprotocol.org/noise.html) §§2–6) | Explicit patterns, CipherState, channel binding and rekey concepts | Pattern/identity choice is application responsibility; misuse risk | Candidate only |
| QUIC-TLS ([RFC 9001](https://www.rfc-editor.org/rfc/rfc9001) §§4–5) | Strong deployed reference | Tightly coupled to QUIC packet spaces and rules | Reference, not adoption |

No algorithm or library is selected in M0. No self-designed cryptography is allowed.

## Required security gates

- **Nonce/key:** select a mature AEAD only after nonce construction is specified. [RFC 5116](https://www.rfc-editor.org/rfc/rfc5116) §§3.1–3.2 requires a defined nonce-generation/formation contract. [RFC 8446](https://www.rfc-editor.org/rfc/rfc8446) §§5.3,5.5 and §7 are references for sequence-dependent record protection, usage limits and key schedule. Never reuse a key/nonce pair; define key phase, epoch and update limits.
- **Replay/early data:** default M0/M1 is no 0-RTT or unauthenticated early data. RFC 8446 §8 and Noise’s application responsibilities make replay policy an endpoint decision. Resumption must bind Session identity, epoch and transcript; replay acceptance must be bounded and idempotent.
- **Path:** RFC 9000 §8 Address Validation, especially §8.1, limits an unvalidated sender to at most three times the bytes received from that address. RFC 9000 §8.2 and §§8.2.1–8.2.4 define path validation initiation, response, success and failure. ACK entropy is insufficient to prove reverse reachability; use an independent authenticated challenge/response. These are references, not a claim of QUIC compliance.
- **Resource:** cap record/frame/stream/session memory, CPU, handshake work, amplification, rate and queue growth before parsing attacker-controlled lengths. Reject unknown versions, truncation and impossible offsets without unbounded allocation.
- **Logging/pcap:** never log keys, PSKs, private material or plaintext payload. Redact addresses/topology where unnecessary; test fixtures must be synthetic. Capture files require an explicit retention and redaction policy.

## M0 prohibition list

No self-made cipher, nonce shortcut, 0-RTT, unauthenticated control state change, unbounded decode allocation, production proxy mode, or claim of audit/compliance. TLS/Noise, identity model and key update remain decision gates; licensing is resolved as `MIT OR Apache-2.0`.
