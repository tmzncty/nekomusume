# Protocol Standards Research

**Protocol research: 伊冯；Security research: 阿米娅；Transport research: 符玄；Plan & implementation: 庄方宜；Coordination/Review: 佩丽卡**
**检索日期：2026-08-26**
**Status: research input, not a frozen wire specification.**

## Evidence rules

RFC Editor copies are the external source of truth. “Fact” means the cited standard says it; “Mapping” is a proposed analogy for Nekomusume; “Do not copy” records semantic boundaries; “Open” requires an explicit ADR or experiment.

## Comparison

| Source | Fact | Mapping | Do not copy | Open |
|---|---|---|---|---|
| [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000) §§4,8,8.1,8.2,13 | QUIC separates connection/path behavior, validates paths, and protects against amplification. §8.2.1–§8.2.4 define path validation lifecycle. | Use separate Session delivery and Carrier/Path feedback domains; use an authenticated challenge for a new path. | Do not call a TCP↔UDP resume “QUIC migration”; QUIC packet-number spaces and connection IDs are not Nekomusume wire decisions. | Challenge transcript, epoch and address-change policy. |
| [RFC 9001](https://www.rfc-editor.org/rfc/rfc9001) §§5–6 | TLS is integrated with QUIC packet protection and key phases. | Treat mature handshake/key schedule as a candidate boundary for M1. | Do not reuse QUIC packet protection or claim QUIC compliance. | TLS-over-record vs Noise. |
| [RFC 9002](https://www.rfc-editor.org/rfc/rfc9002) §§5–7 | ACK delay, RTT, loss detection and congestion-control guidance are defined for QUIC recovery. | Candidate inputs for UDP Carrier recovery in M1/M2. | No congestion control or PTO algorithm is frozen by M0. | Reno/CUBIC baseline and parameter selection. |
| [RFC 8684](https://www.rfc-editor.org/rfc/rfc8684) §§3–5 | MPTCP uses tokens, subflows, path management and a connection-level data sequence. | Evidence that logical data identity can outlive a path. | Do not import MPTCP scheduler, simultaneous multipath, or token format. | Failover first; aggregation is later research. |
| [RFC 9260](https://www.rfc-editor.org/rfc/rfc9260) §§1,6,7 | SCTP has associations, streams, chunks and acknowledgements. | Compare stream/association separation. | SCTP chunks and multihoming are not Nekomusume framing. | Whether SCTP remains experimental only. |
| [RFC 4340](https://www.rfc-editor.org/rfc/rfc4340) §§1,4,11 | DCCP provides congestion-controlled unreliable datagrams. | Reference for an unreliable Carrier candidate. | No DCCP semantics in M0. | Reachability experiment only. |
| [RFC 9297](https://www.rfc-editor.org/rfc/rfc9297) §§1–3, [RFC 9298](https://www.rfc-editor.org/rfc/rfc9298) §§1–3, [RFC 9484](https://www.rfc-editor.org/rfc/rfc9484) §§1–3 | HTTP Datagrams/Capsules and CONNECT-UDP/IP proxy datagrams are application/proxy mechanisms. | Useful vocabulary for later proxy carriers. | They do not define Nekomusume Session identity or delivery ACK. | Experimental proxy Carrier scope. |

## M0 model

`Session` owns identity, epoch, authenticated logical records and delivery state. `Carrier` names a mechanism (UDP/TCP/etc.). `Path` names one concrete Carrier instance and address. `active_path_epoch` changes only after authenticated path validation. M0 records three evidence domains: `packet_feedback` (Carrier-local), `session_delivery` (logical received/delivered), and `path_validated` (independent reverse reachability).

## Questions retained

Magic/version, canonical integers, record limits, ACK proof object, challenge transcript, key selection, and license remain governed by `docs/m0-spec-plan.md` and ADRs to be added only when implementation starts.
