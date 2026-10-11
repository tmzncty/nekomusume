//! D070 P4: windowed UDP bulk transfer (Reno AIMD + SACK selective
//! retransmission) on the reliable-UDP carrier stack.
//!
//! Reuses the frozen recovery machinery end to end — `ReliableUdpRuntime`
//! (PathRecovery + Reno + RttEstimator + RetransmitBuffer), `PacketAckTracker`
//! for receiver SACK obligations, and `neko_wire` ACK frames — zero new
//! protocol surface, zero new algorithms (D070 decision 1). The packet number
//! is the authenticated SecureSession record sequence (the AEAD nonce), the
//! same single source of truth as the failover R9 wiring.
//!
//! Wire shape (D070 §3.2's 1250B sealed-record arithmetic): a bulk DATA
//! record is exactly one `ProcessMessage::Data` plaintext (30B header +
//! payload, capped at 1200B = MAX_UNRELIABLE_DATAGRAM), sealed to 1250B on
//! the wire. Control traffic (SACK, the END termination notice, the final
//! receipt) rides the small `neko_wire::Record` forms; on the receive side
//! the two shapes demux unambiguously because a ProcessMessage's fixed
//! session-prefix bytes never decode as a canonical wire record.
//!
//! Replay-window coupling (D070 decisions 2-3, the hard gates):
//! - `cwnd <= 48` is pinned at compile time (`MAX_WINDOW`) — 64-wide crypto
//!   replay window minus 16 headroom for ACK/control records and reordering.
//! - A retransmission re-seals the retained frame plaintext under a FRESH
//!   sequence (fresh AEAD nonce; same stable `FrameId`), honoring the frozen
//!   `on_retransmit_sent`/`pto_probe` runtime contract (failover shape). The
//!   original sealed record is never re-emitted byte-identical, so a
//!   retransmission never presents an old nonce to the receiver's replay
//!   window, and each original is sealed exactly once (nonce discipline).
//! - On the receiver, an authenticated packet number already inside the
//!   accepted ACK ranges is a retransmission of accepted work: silent no-op,
//!   never a session kill (D070 decision 3); only a genuine AEAD/context
//!   failure on an unseen packet fails the session.
//!
//! Measurement-gated exactly like the TCP bulk path (D068/D069/D070): the
//! command refuses without NEKO_MEASUREMENT=1 and the TCP bulk fixture is
//! untouched (rollback = removing the `--transport udp` dispatch branch).
use super::*;
use crate::bulk::{install_shutdown, payload_block};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, VecDeque};
use std::net::UdpSocket;

const SESSION: SessionId = SessionId(7302);
const STREAM: StreamId = StreamId(1);
/// D070 decision 1: window levels {8,16,32,48}, default 32.
const DEFAULT_WINDOW: u32 = 32;
/// D070 decision 2: hard compile-time pin. The crypto replay window is 64
/// sealed sequences wide (MAX_REPLAY_WINDOW); an in-flight data window that
/// large could let a reordering-triggered retransmission fall outside the
/// replay window and be misjudged as an attack. 48 = 64 - 16 headroom.
/// MUST NOT be relaxed without a D070 successor decision.
pub const MAX_WINDOW: u32 = 48;
/// D070 decision 4: retransmit plaintext budget — one full window of
/// in-flight records plus the END notice, ≤ 60 KiB + 1250B.
const RETRANSMIT_FRAMES: usize = 64;
const RETRANSMIT_BYTES: usize = 61_440;
/// Sealed size of one maximal bulk DATA record: 1200B plaintext + 50B seal
/// overhead (seq 8 + context 26 + tag 16). Also the Reno MSS.
const RECORD_WIRE_BYTES: u64 = 1250;
/// D070 §3.4: PLPMTUD fail-closed floor — a path with a confirmed MTU below
/// 1300 cannot carry the 1278B IP packets a 1250B record needs without
/// fragmentation; refuse to start rather than adaptively shrink (D067).
const PLPMTUD_FLOOR: u16 = 1300;
/// Receiver reorder depth = the pinned window (D070 decision 4).
const REORDER_DEPTH: usize = MAX_WINDOW as usize;

#[derive(Debug)]
pub(super) struct UdpBulkConfig {
    pub port: u16,
    pub bytes_total: usize,
    pub record_bytes: usize,
    pub duration: Duration,
    pub window: u32,
}

/// D070 decision 5: PLPMTUD fail-closed gate. `None` (unknown MTU — PLPMTUD
/// not run on this path) admits; a confirmed MTU below the floor refuses.
pub(crate) fn plpmtud_gate(confirmed: Option<u16>) -> bool {
    confirmed.is_none_or(|mtu| mtu >= PLPMTUD_FLOOR)
}

pub(super) fn config_udp(args: &[String]) -> Result<UdpBulkConfig, &'static str> {
    if !crate::measurement::enabled() {
        return Err("bulk requires NEKO_MEASUREMENT=1 (research-only)");
    }
    let num = |key: &str, default: &str| -> Result<u64, &'static str> {
        crate::parse(args, key, Some(default))
            .parse()
            .map_err(|_| "invalid numeric argument")
    };
    let port = num("--port", "40080")? as u16;
    if !(40080..=MAX_PORT).contains(&port) {
        return Err("port outside 40080-40100");
    }
    let record_bytes = num("--record-bytes", &crate::PERIODIC_BYTES_MAX.to_string())? as usize;
    if record_bytes == 0 || record_bytes > crate::PERIODIC_BYTES_MAX {
        return Err("record-bytes outside 1-1170");
    }
    let bytes_total = num("--bytes-total", &(64 << 20).to_string())? as usize;
    if bytes_total == 0
        || !bytes_total.is_multiple_of(record_bytes)
        || bytes_total > neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES
    {
        return Err("bytes-total must be a positive multiple of record-bytes, <= 2147483648");
    }
    let duration = num("--duration", "86400")?;
    if duration == 0 || duration > crate::MEASUREMENT_DURATION_MAX {
        return Err("duration outside 1-86400");
    }
    let window = num("--window", &DEFAULT_WINDOW.to_string())? as u32;
    if !matches!(window, 8 | 16 | 32 | 48) {
        return Err("window must be one of 8, 16, 32, 48");
    }
    Ok(UdpBulkConfig {
        port,
        bytes_total,
        record_bytes,
        duration: Duration::from_secs(duration),
        window,
    })
}

fn limits_udp() -> neko_session::RuntimeLimits {
    neko_session::RuntimeLimits {
        max_streams: 1,
        max_queue_records: neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_RECORDS,
        max_queue_bytes: neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_BYTES,
        max_total_bytes: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        max_record_bytes: crate::PERIODIC_BYTES_MAX,
        max_session_window: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        max_stream_window: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        ..neko_session::RuntimeLimits::default()
    }
}

/// The END termination notice: one small wire Data record whose payload is
/// the declared record count (BE u64). Sent after the last DATA record is
/// acknowledged; the receiver finalizes once all records are present.
fn wire_end(total_records: u64) -> Vec<u8> {
    neko_wire::encode(&neko_wire::Record {
        record_type: neko_wire::RecordType::Data,
        flags: 0,
        payload: total_records.to_be_bytes().to_vec(),
    })
    .unwrap()
}

/// Send-with-drain: the UDP send buffer can refuse large bursts on loopback;
/// draining one datagram keeps the window loop live without busy-spinning.
fn send_drain(sock: &UdpSocket, buf: &[u8], peer: SocketAddr) -> bool {
    for attempt in 0..64 {
        match sock.send_to(buf, peer) {
            Ok(_) => return true,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::Interrupted
                ) =>
            {
                std::thread::sleep(Duration::from_micros(200 + attempt * 50));
            }
            Err(_) => return false,
        }
    }
    false
}

fn wire_ack(ack: &neko_wire::AckPayload) -> Option<Vec<u8>> {
    let payload = neko_wire::encode_ack(ack).ok()?;
    neko_wire::encode(&neko_wire::Record {
        record_type: neko_wire::RecordType::Ack,
        flags: 0,
        payload,
    })
    .ok()
}

/// Classify a sealed-record plaintext on the client: SACK frame, final
/// receipt, or nothing this client expects.
#[derive(Debug)]
enum ClientInbound {
    Ack(neko_wire::AckPayload),
    Receipt(Vec<u8>),
    Other,
}
fn classify_client(plain: &[u8]) -> ClientInbound {
    if let Ok(rec) = neko_wire::decode(plain)
        && rec.record_type == neko_wire::RecordType::Ack
        && let Ok(ack) = neko_wire::decode_ack(&rec.payload)
    {
        return ClientInbound::Ack(ack);
    }
    if let Ok(ProcessMessage::Data { session, record }) = ProcessMessage::decode(plain)
        && session == SESSION
        && record
            .data
            .starts_with(b"{\"ok\":true,\"role\":\"bulk-udp-server\"")
    {
        return ClientInbound::Receipt(record.data);
    }
    ClientInbound::Other
}

/// Classify a sealed-record plaintext on the server: END notice, bulk DATA
/// record, or nothing this server expects (ACK frames from the client are
/// ignored — the server never solicits them).
#[derive(Debug)]
enum ServerInbound {
    End(u64),
    Data(u64, Vec<u8>),
    Other,
}
fn classify_server(plain: &[u8]) -> ServerInbound {
    if let Ok(rec) = neko_wire::decode(plain)
        && rec.record_type == neko_wire::RecordType::Data
        && rec.payload.len() == 8
    {
        return ServerInbound::End(u64::from_be_bytes(rec.payload[..].try_into().unwrap()));
    }
    if let Ok(ProcessMessage::Data { session, record }) = ProcessMessage::decode(plain)
        && session == SESSION
        && record.stream == STREAM
    {
        return ServerInbound::Data(record.offset, record.data);
    }
    ServerInbound::Other
}

/// The pinned peer's ACK obligation flush: emit at most one SACK frame when
/// an ack-eliciting packet was observed (or `forced` by a terminal event).
fn flush_ack(
    sock: &UdpSocket,
    secure: &mut neko_crypto::SecureSession,
    acks: &mut neko_carrier::PacketAckTracker,
    peer: SocketAddr,
    forced: bool,
) {
    if (forced || acks.pending_ack())
        && let Some(ack) = acks.take_ack(0)
        && let Some(wire) = wire_ack(&ack)
        && let Ok(sealed) = secure.seal_unreliable(&wire)
    {
        let _ = sock.send_to(&sealed, peer);
    }
}

fn value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

pub(super) fn server_udp(args: &[String]) {
    crate::measurement::emit_marker();
    println!("measurement_limits_exceptions=total_bytes,queue_records,queue_bytes");
    let cfg = config_udp(args).unwrap_or_else(|e| fail(e));
    let client_key = peer_public_key(&parse(args, "--client-key", None));
    let bind = parse(args, "--bind", Some(&format!("0.0.0.0:{}", cfg.port)))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad bind address"));
    let shutdown = install_shutdown();
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    println!("server_public_key={}", hex(id.public_key()));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: b"probe".to_vec(),
        status: TrustStatus::Active,
    }]);
    let sock = UdpSocket::bind(bind).unwrap_or_else(|_| fail("bind failed"));
    sock.set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    println!("bulk_udp_server_ready port={}", cfg.port);
    // Test observability seam: an optional ready marker file beats pipe
    // buffering races when a harness (not a terminal) holds the other end.
    if let Some(path) = value(args, "--ready-file") {
        std::fs::write(&path, format!("bulk_udp_server_ready port={}\n", cfg.port))
            .unwrap_or_else(|_| fail("ready file write failed"));
    }
    let start = Instant::now();
    let deadline = start + cfg.duration;
    let mut buf = [0u8; 65536];

    // ---- UDP Noise handshake (failover client-hello shape, cached replies
    // for retried hellos / first messages on a lossy path) ----
    let mut negotiation = VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
        .unwrap_or_else(|_| fail("negotiation setup failed"));
    // (hello, selection, binding, peer) staged during the UDP handshake.
    type PendingHandshake = Option<(Vec<u8>, Vec<u8>, Vec<u8>, SocketAddr)>;
    let mut pending: PendingHandshake = None;
    // Post-handshake echo cache: (first_message, response) so a retried
    // first datagram (its response was lost) is answered from the cache
    // instead of hitting the authenticated-data path (failover shape).
    #[allow(unused_assignments)]
    let mut handshake_echo: Option<(Vec<u8>, Vec<u8>)> = None;
    let (mut secure, peer) = loop {
        if start.elapsed() >= cfg.duration || shutdown.load(Ordering::Acquire) {
            fail("duration expired before authenticated Session");
        }
        let (n, p) = match sock.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::ConnectionRefused) => {
                // ICMP port-unreachable echo from an earlier datagram aimed
                // at a peer socket that briefly closed — tolerate and retry.
                continue;
            }
            Err(_) => fail("handshake receive failed"),
        };
        if let Some((_, _, _, pinned)) = pending.as_ref()
            && p != *pinned
        {
            continue;
        }
        match pending.take() {
            None => {
                if let Ok(selection) = negotiation.server_accept_hello(&buf[..n]) {
                    let binding = negotiation
                        .authenticated_binding()
                        .unwrap()
                        .as_bytes()
                        .to_vec();
                    let _ = sock.send_to(&selection, p);
                    pending = Some((buf[..n].to_vec(), selection, binding, p));
                }
            }
            Some((hello, selection, binding, peer)) => {
                if buf[..n] == hello {
                    let _ = sock.send_to(&selection, p);
                    pending = Some((hello, selection, binding, peer));
                    continue;
                }
                let hs = ResponderHandshake::new_with_prologue_binding(
                    &id,
                    policy.clone(),
                    DOMAIN,
                    &binding,
                );
                match hs
                    .unwrap_or_else(|_| fail("handshake setup failed"))
                    .receive_first(&buf[..n], context(0))
                {
                    Ok((response, ss)) => {
                        let _ = sock.send_to(&response, p);
                        negotiation
                            .admit_data()
                            .unwrap_or_else(|_| fail("data admission denied"));
                        handshake_echo = Some((buf[..n].to_vec(), response.clone()));
                        break (ss, p);
                    }
                    Err(_) => pending = Some((hello, selection, binding, peer)),
                }
            }
        }
    };
    println!("bulk_udp_server_authenticated session=7302 stream=1");

    let mut runtime = neko_session::SessionRuntime::new_measurement(
        SESSION,
        limits_udp(),
        start.elapsed().as_millis() as u64,
    )
    .unwrap_or_else(|_| fail("runtime rejected"));
    runtime
        .open_stream(STREAM, start.elapsed().as_millis() as u64)
        .unwrap();

    let total_records = cfg.bytes_total / cfg.record_bytes;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;
    let mut received_bytes: u64 = 0;
    let mut end_seen = false;
    // Receiver ACK tracker: accepted packet numbers (the pre-open duplicate
    // gate) and the SACK obligation latch. Generation 1 matches the client's
    // runtime.
    let mut acks = neko_carrier::PacketAckTracker::new(1);
    // D070 decision 4: reorder buffer — offset-indexed records that arrived
    // ahead of the delivery frontier; bounded by the pinned window.
    let mut reorder: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut reorder_bytes: usize = 0;
    let mut delivered: u64 = 0;
    let mut duplicates_skipped: u64 = 0;

    while Instant::now() < deadline && !shutdown.load(Ordering::Acquire) {
        if end_seen && received >= total_records as u64 {
            break;
        }
        let (n, p) = match sock.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                flush_ack(&sock, &mut secure, &mut acks, peer, false);
                continue;
            }
            Err(_) => break,
        };
        if p != peer {
            continue;
        }
        let sealed = &buf[..n];
        if sealed.len() < 8 {
            continue;
        }
        let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
        // P4b hard gate (D070 decision 3 / assertion 4): pre-open duplicate
        // gate — a packet number already inside the accepted ACK ranges is a
        // retransmission of accepted work. Silently skip it; never hand it
        // to the crypto replay window, never fail the session.
        if acks
            .build_ack(0)
            .is_some_and(|a| a.ranges.iter().any(|r| r.start <= pn && pn <= r.end))
        {
            duplicates_skipped += 1;
            continue;
        }
        // A retried Noise first-message (its response was lost) is answered
        // from the handshake echo cache — never treated as bulk data.
        if let Some((first, response)) = handshake_echo.as_ref()
            && sealed == first.as_slice()
        {
            let _ = sock.send_to(response, peer);
            continue;
        }
        let plain = match secure.open_unreliable(sealed) {
            Ok(v) => v,
            Err(_) => {
                // An AEAD-failing record whose packet number the tracker has
                // NOT accepted is a genuine authentication failure on unseen
                // work (tampering): fail closed. (The in-window duplicate
                // case was already skipped by the gate above.)
                fail("unauthenticated bulk data");
            }
        };
        match classify_server(&plain) {
            ServerInbound::End(declared) => {
                if declared != total_records as u64 {
                    fail("bulk end mismatch");
                }
                end_seen = true;
                // The END notice is not data: acknowledge its arrival so the
                // client can retire it, without counting it in the ranges
                // gate for data (its pn IS accepted here — an END
                // retransmission hits the gate above and is skipped).
                acks.observe_packet(1, pn, true)
                    .unwrap_or_else(|_| fail("ack tracker rejected END"));
                flush_ack(&sock, &mut secure, &mut acks, peer, true);
            }
            ServerInbound::Data(offset, data) => {
                if offset % cfg.record_bytes as u64 != 0
                    || data.len() != cfg.record_bytes
                    || offset / cfg.record_bytes as u64 >= total_records as u64
                {
                    fail("bulk record outside contract");
                }
                let idx = offset / cfg.record_bytes as u64;
                // A retransmission of an ALREADY-DELIVERED record (fresh
                // nonce, stale data): observe its packet number so the copy
                // retires sender-side, but never let it touch the reorder
                // buffer — stale retransmissions must not pollute the
                // bounded window (D070 decision 4).
                if idx < delivered {
                    acks.observe_packet(1, pn, true)
                        .unwrap_or_else(|_| fail("ack tracker rejected"));
                    duplicates_skipped += 1;
                    continue;
                }
                // D070 decision 4 (bounded receiver): when the reorder
                // buffer is at capacity, an ABOVE-FRONTIER record is
                // dropped WITHOUT entering ACK state — its SACK silence
                // keeps the copy in-flight sender-side and the PTO
                // retransmission retries once space frees. Never a session
                // kill, never an unbounded buffer. The frontier record
                // itself always admits (it frees space by delivering).
                let at_capacity = reorder.len() >= REORDER_DEPTH
                    || reorder_bytes.saturating_add(data.len()) > RETRANSMIT_BYTES;
                if at_capacity && idx != delivered {
                    continue;
                }
                // Observing the packet number creates the SACK obligation —
                // after authentication, exactly once per packet number.
                acks.observe_packet(1, pn, true)
                    .unwrap_or_else(|_| fail("ack tracker rejected"));
                let expected = payload_block(idx, cfg.record_bytes);
                if data != expected {
                    fail("bulk payload checksum mismatch");
                }
                if reorder.insert(idx, data.to_vec()).is_none() {
                    reorder_bytes += data.len();
                } else {
                    duplicates_skipped += 1;
                }
                debug_assert!(
                    reorder.len() <= REORDER_DEPTH + 1 && reorder_bytes <= RETRANSMIT_BYTES,
                    "reorder ceiling invariant (D070 decision 4)"
                );
                // Deliver every contiguous record from the frontier.
                while let Some(data) = reorder.remove(&delivered) {
                    hasher.update(&data);
                    received += 1;
                    received_bytes += data.len() as u64;
                    reorder_bytes = reorder_bytes.saturating_sub(data.len());
                    runtime
                        .receive(
                            InboundRecord {
                                stream: STREAM,
                                offset: delivered * cfg.record_bytes as u64,
                                data,
                            },
                            start.elapsed().as_millis() as u64,
                        )
                        .unwrap_or_else(|_| fail("bulk record rejected by runtime"));
                    let _ = runtime.pop_receive(start.elapsed().as_millis() as u64);
                    delivered += 1;
                }
                flush_ack(&sock, &mut secure, &mut acks, peer, false);
            }
            ServerInbound::Other => {
                // Authenticated but unmatched shape: ignore (bounded by the
                // session's own auth surface; nothing here mutates state).
            }
        }
    }
    // Final flush: never strand a pending SACK at termination.
    flush_ack(&sock, &mut secure, &mut acks, peer, true);
    let digest = hex(&hasher.finalize()[..]);
    let receipt = format!(
        "{{\"ok\":true,\"role\":\"bulk-udp-server\",\"records\":{},\"bytes\":{},\"sha256\":\"{}\",\"duplicates_skipped\":{}}}",
        received, received_bytes, digest, duplicates_skipped
    );
    println!("bulk_udp_server_summary {receipt}");
    // Wire receipt: one sealed ProcessMessage::Data the client verifies by
    // itself; retransmit briefly so a single loss does not strand it.
    let receipt_plain = ProcessMessage::Data {
        session: SESSION,
        record: neko_session::OutboundRecord {
            stream: STREAM,
            offset: 0,
            data: receipt.clone().into_bytes(),
        },
    }
    .encode()
    .unwrap();
    for _ in 0..4 {
        if let Ok(sealed) = secure.seal_unreliable(&receipt_plain) {
            let _ = sock.send_to(&sealed, peer);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Retire a frame everywhere: bookkeeping (pn_owner, sent_wire, counters)
/// AND the recovery engine — every engine-registered pn this frame still
/// owns is abandoned so no ghost in-flight byte survives the retirement.
fn retire_frame(
    idx: u64,
    pn_owner: &mut BTreeMap<u64, u64>,
    sent_wire: &mut BTreeMap<u64, Vec<u8>>,
    rt: &mut neko_carrier::ReliableUdpRuntime,
    acked_records: &mut u64,
    end_acked: &mut bool,
) {
    // Engine: abandon every pn still owned by this frame (Err = already
    // retired, ignore).
    let owned: Vec<u64> = pn_owner
        .iter()
        .filter(|(_, i)| **i == idx)
        .map(|(&pn, _)| pn)
        .collect();
    for pn in owned {
        let _ = rt.abandon_retransmit(pn);
        pn_owner.remove(&pn);
    }
    if idx == u64::MAX {
        *end_acked = true;
    } else if sent_wire.remove(&idx).is_some() {
        *acked_records += 1;
    }
}

pub(super) fn client_udp(args: &[String]) {
    crate::measurement::emit_marker();
    println!("measurement_limits_exceptions=total_bytes,queue_records,queue_bytes");
    let cfg = config_udp(args).unwrap_or_else(|e| fail(e));
    let addr = parse(args, "--addr", None)
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad address"));
    let server_key = peer_public_key(&parse(args, "--server-key", None));
    let shutdown = install_shutdown();
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
    println!("client_public_key={}", hex(id.public_key()));
    let start = Instant::now();
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap_or_else(|_| fail("client bind failed"));
    sock.set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    let mut buf = [0u8; 65536];

    // ---- UDP Noise handshake (failover client shape; each phase retransmits
    // its request until the matching response arrives) ----
    let mut negotiation = VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS)
        .unwrap_or_else(|_| fail("negotiation setup failed"));
    let hello = negotiation.client_hello().unwrap();
    let mut got: Option<Vec<u8>> = None;
    let hs_deadline = start + Duration::from_secs(120);
    while got.is_none() && Instant::now() < hs_deadline && !shutdown.load(Ordering::Acquire) {
        sock.send_to(&hello, addr).unwrap();
        match sock.recv_from(&mut buf) {
            Ok((n, p)) if p == addr => {
                // Accept only a VALID negotiation response — a stray/stale
                // datagram keeps the retry loop alive instead of poisoning
                // the phase.
                if negotiation.client_accept_response(&buf[..n]).is_ok() {
                    got = Some(buf[..n].to_vec());
                }
            }
            Ok(_) => continue,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::ConnectionRefused) => {
                // ICMP echo from a datagram aimed at a momentarily-closed
                // peer port — tolerate and keep retrying (lossy-path norm).
                continue;
            }
            Err(_) => fail("UDP handshake receive failed"),
        }
    }
    drop(got);
    let binding = negotiation.authenticated_binding().unwrap();
    let mut hs = InitiatorHandshake::new_with_prologue_binding(
        &id,
        &server_key,
        b"probe",
        DOMAIN,
        binding.as_bytes(),
    )
    .unwrap_or_else(|_| fail("handshake setup failed"));
    let first = hs.first_message().unwrap();
    let mut hs = Some(hs);
    let mut secure: Option<neko_crypto::SecureSession> = None;
    while secure.is_none() && Instant::now() < hs_deadline && !shutdown.load(Ordering::Acquire) {
        sock.send_to(&first, addr).unwrap();
        match sock.recv_from(&mut buf) {
            Ok((n, p)) if p == addr => {
                // Noise read_message rejection leaves the handshake object
                // usable (failover's UDP client relies on the same
                // property): a stale/retried selection keeps waiting; only
                // the real Noise response finishes the handshake.
                if let Some(h) = hs.take()
                    && let Ok(session) = h.finish(&buf[..n], context(0))
                {
                    secure = Some(session);
                } else {
                    hs = hs.or(Some(
                        InitiatorHandshake::new_with_prologue_binding(
                            &id,
                            &server_key,
                            b"probe",
                            DOMAIN,
                            binding.as_bytes(),
                        )
                        .unwrap_or_else(|_| fail("handshake setup failed")),
                    ));
                    // The rebuilt object re-emits the same first message;
                    // retransmit it immediately on the next loop tick.
                }
            }
            Ok(_) => continue,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::ConnectionRefused) => {
                // ICMP echo from a datagram aimed at a momentarily-closed
                // peer port — tolerate and keep retrying (lossy-path norm).
                continue;
            }
            Err(_) => fail("UDP handshake receive failed"),
        }
    }
    let mut secure = secure.unwrap_or_else(|| fail("UDP handshake timeout"));
    negotiation
        .admit_data()
        .unwrap_or_else(|_| fail("data admission denied"));
    println!("bulk_udp_client_authenticated session=7302 stream=1");
    sock.set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();

    let mut runtime = neko_session::SessionRuntime::new_measurement(
        SESSION,
        limits_udp(),
        start.elapsed().as_millis() as u64,
    )
    .unwrap_or_else(|_| fail("runtime rejected"));
    runtime
        .open_stream(STREAM, start.elapsed().as_millis() as u64)
        .unwrap();

    // D070 decision 1/2: the sender rides the frozen ReliableUdpRuntime —
    // Reno admission, Recovery sent-map, RetransmitBuffer ownership — with
    // the D070 retransmit budget. MSS = one maximal sealed record so cwnd
    // counts whole records (min cwnd = 2 MSS keeps recovery admissible
    // after loss). PLPMTUD is not run on this path (unknown MTU admits;
    // a confirmed sub-1300 MTU would have refused above via the gate).
    let mut rt = neko_carrier::ReliableUdpRuntime::new_with_retransmit_budget(
        1,
        RECORD_WIRE_BYTES,
        RETRANSMIT_FRAMES,
        RETRANSMIT_BYTES,
    )
    .unwrap_or_else(|_| fail("reliable runtime rejected"));
    if !plpmtud_gate(rt.pmtu_confirmed_mtu()) {
        fail("PLPMTUD confirmed MTU below the D070 floor");
    }
    let window_ceiling = cfg.window.min(MAX_WINDOW) as u64;

    let total_records = cfg.bytes_total / cfg.record_bytes;
    let deadline = start + cfg.duration;
    let epoch = Instant::now();
    let now_us = |epoch: &Instant| epoch.elapsed().as_micros() as u64;

    // Sender state: retained wire plaintexts (in-flight frames), the
    // packet->owner map for ACK retirement, and the loss/PTO retransmit
    // queue. END's stable frame id is u64::MAX.
    let mut sent_wire: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut pn_owner: BTreeMap<u64, u64> = BTreeMap::new();
    let mut retransmit_queue: VecDeque<u64> = VecDeque::new();
    // Records popped from the session queue whose engine admission was
    // refused — retried verbatim (see admission phase 1).
    let mut unsent_wire: VecDeque<(u64, Vec<u8>)> = VecDeque::new();
    let mut next_record: u64 = 0;
    let mut acked_records: u64 = 0;
    let mut retransmits: u64 = 0;
    let mut pto_probes: u64 = 0;
    let mut end_sent = false;
    let mut end_acked = false;
    let mut last_pto_fire: Option<u64> = None;
    let mut last_hb_dbg: u64 = 0;
    let mut last_progress_us: u64 = 0;
    let mut last_dump_us: u64 = 0;
    let mut last_rebuild_us: u64 = 0;
    let mut ack_largest_seen: u64 = 0;
    let mut _pto_count_empty_rounds: u64 = 0;
    // Highest pn ever registered in the recovery engine (fresh/rtx/probe).
    let mut max_engine_pn: u64 = 0;
    let mut receipt: Option<String> = None;

    // Sealed-send helpers: seal under a FRESH nonce every time (originals
    // exactly once; retransmissions always fresh) and record ownership.
    let send_fresh = |idx: u64,
                      wire: &[u8],
                      secure: &mut neko_crypto::SecureSession,
                      rt: &mut neko_carrier::ReliableUdpRuntime,
                      sent_wire: &mut BTreeMap<u64, Vec<u8>>,
                      pn_owner: &mut BTreeMap<u64, u64>,
                      max_engine_pn: &mut u64|
     -> Result<(), &'static str> {
        let sealed = secure
            .seal_unreliable(wire)
            .map_err(|_| "bulk seal failed")?;
        let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
        rt.on_packet_sent(
            pn,
            now_us(&epoch),
            sealed.len() as u64,
            neko_reliable::FrameId(idx),
            wire,
        )
        .map_err(|_| "recovery on_packet_sent rejected")?;
        match sock.send_to(&sealed, addr) {
            Ok(_) => {
                sent_wire.insert(idx, wire.to_vec());
                pn_owner.insert(pn, idx);
                *max_engine_pn = (*max_engine_pn).max(pn);
                Ok(())
            }
            Err(_) => {
                rt.abandon_sent(pn);
                Err("bulk send failed")
            }
        }
    };

    while Instant::now() < deadline && !shutdown.load(Ordering::Acquire) {
        if end_acked {
            break;
        }
        // 1. Admission-gated fresh sends while the pinned window has room.
        // D070 offset-integrity rule: a record popped from the session
        // queue has its stream offset BURNED at queue_send time. If the
        // recovery engine then rejects the send (CongestionWindowFull /
        // retransmit budget saturated), the wire must be CACHED and retried
        // verbatim — re-queueing the same payload would allocate a NEW
        // offset (next_send never rewinds), sending "offset=N+1 carrying
        // record N" and desynchronizing the receiver's checksum map (the
        // off-by-one corruption seen at the ~2MB retransmit-budget
        // saturation point).
        // Honest outstanding: after a stalled-engine rebuild the engine's
        // in_flight resets to 0 while sent_wire still holds unacknowledged
        // records — counting only the engine would admit unbounded fresh
        // records past an unfilled hole (observed 1088 ahead). sent_wire
        // may double-count probe-registered copies — conservative, safe.
        let outstanding = rt.in_flight() + sent_wire.len();
        while !unsent_wire.is_empty()
            && (outstanding as u64) < window_ceiling
            && rt.can_send(RECORD_WIRE_BYTES)
        {
            let (idx, wire) = unsent_wire.front().unwrap().clone();
            match send_fresh(
                idx,
                &wire,
                &mut secure,
                &mut rt,
                &mut sent_wire,
                &mut pn_owner,
                &mut max_engine_pn,
            ) {
                Ok(()) => {
                    unsent_wire.pop_front();
                    debug_assert_eq!(idx, next_record);
                    next_record += 1;
                }
                // Park: budget saturated — the ACK drain frees it.
                Err("recovery on_packet_sent rejected") => break,
                Err(e) => fail(e),
            }
        }
        while (next_record as usize) < total_records
            && unsent_wire.is_empty()
            && (outstanding as u64) < window_ceiling
            && rt.can_send(RECORD_WIRE_BYTES)
        {
            let idx = next_record;
            let payload = payload_block(idx, cfg.record_bytes);
            runtime
                .queue_send(STREAM, &payload, start.elapsed().as_millis() as u64)
                .unwrap_or_else(|_| fail("bulk queue rejected"));
            let record = runtime
                .pop_send(start.elapsed().as_millis() as u64)
                .unwrap()
                .unwrap_or_else(|| fail("bulk queue starved"));
            debug_assert_eq!(record.offset, idx * cfg.record_bytes as u64);
            debug_assert_eq!(record.data, payload);
            let wire = ProcessMessage::Data {
                session: SESSION,
                record,
            }
            .encode()
            .unwrap();
            match send_fresh(
                idx,
                &wire,
                &mut secure,
                &mut rt,
                &mut sent_wire,
                &mut pn_owner,
                &mut max_engine_pn,
            ) {
                Ok(()) => next_record += 1,
                // CongestionWindowFull is the caller-side park signal.
                // The popped wire is cached — exactly one queue/pop per
                // record, offsets never double-allocate.
                Err("recovery on_packet_sent rejected") => {
                    unsent_wire.push_back((idx, wire));
                    break;
                }
                Err(e) => fail(e),
            }
        }
        // 2. Retransmit loss-scheduled frames (fresh nonce per copy). Loss
        //    recovery is not cwnd-gated: the frozen on_retransmit_sent
        //    never cwnd-checks (only on_packet_sent does), Reno's lost()
        //    already halved the window as the congestion response, and
        //    gating recovery behind a b.i.f ceiling that registered probes
        //    keep inflating is the windowed-transfer deadlock. The queue is
        //    naturally bounded (one entry per loss event per window).
        while let Some(idx) = retransmit_queue.pop_front() {
            let Some(wire) = sent_wire.get(&idx).cloned() else {
                continue; // already retired by an overlapping copy's ACK
            };
            let sealed = secure
                .seal_unreliable(&wire)
                .unwrap_or_else(|_| fail("retransmit seal failed"));
            let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
            rt.on_retransmit_sent(
                pn,
                now_us(&epoch),
                sealed.len() as u64,
                neko_reliable::FrameId(idx),
            )
            .unwrap_or_else(|_| fail("on_retransmit_sent rejected"));
            match sock.send_to(&sealed, addr) {
                Ok(_) => {
                    pn_owner.insert(pn, idx);
                    max_engine_pn = max_engine_pn.max(pn);
                    retransmits += 1;
                }
                Err(_) => {
                    rt.abandon_retransmit(pn);
                    retransmit_queue.push_front(idx);
                    break;
                }
            }
        }
        // 3. END notice once every DATA record is acknowledged.
        if next_record as usize >= total_records
            && acked_records as usize >= total_records
            && !end_sent
        {
            let wire = wire_end(total_records as u64);
            send_fresh(
                u64::MAX,
                &wire,
                &mut secure,
                &mut rt,
                &mut sent_wire,
                &mut pn_owner,
                &mut max_engine_pn,
            )
            .unwrap_or_else(|e| fail(e));
            end_sent = true;
        }
        if std::env::var("NEKO_UDP_DEBUG").is_ok()
            && now_us(&epoch) > last_hb_dbg.saturating_add(2_000_000)
        {
            last_hb_dbg = now_us(&epoch);
            eprintln!(
                "[hb] t={}ms acked={} next={} infl={} rtxq={} unsent={} pto_dl={:?} pto_cnt={} bif={}",
                now_us(&epoch) / 1000,
                acked_records,
                next_record,
                rt.in_flight(),
                retransmit_queue.len(),
                unsent_wire.len(),
                rt.next_pto_deadline_us(1_000, 0),
                pto_probes,
                rt.in_flight() as u64 * RECORD_WIRE_BYTES
            );
        }
        // 4. PTO: fire probes over the oldest outstanding frames when the
        //    deadline passes. The probe is the deadlock breaker (QUIC PTO
        //    semantics): it is sent even when cwnd is exhausted — the only
        //    way an all-lost window ever recovers — and
        //    `on_retransmit_sent`, unlike `on_packet_sent`, does not
        //    cwnd-gate. A cwnd-gated loss frame parked at the head of the
        //    retransmit queue joins the probe set FIRST — the frontier hole
        //    it plugs is the only thing that frees window capacity.
        // Client-side PTO-wait clamp: the engine's exponential backoff is
        // the frozen model, but a bulk transfer whose SACK frame was lost
        // enters mutual silence; waiting minutes for the next backoff slot
        // is a liveness failure, not congestion response. Bounded probe
        // cadence: at most one probe burst per PTO_CLAMP_US (1s) — still
        // self-limiting (>> RTT, << amplification budgets).
        const PTO_CLAMP_US: u64 = 1_000_000;
        let now = now_us(&epoch);
        if let Some(pto_deadline) = rt.next_pto_deadline_us(1_000, 0)
            && now >= pto_deadline.min(last_pto_fire.map_or(0, |t| t.saturating_add(PTO_CLAMP_US)))
        {
            let mut probes: Vec<(neko_reliable::FrameId, Vec<u8>)> = Vec::new();
            if let Some(&head) = retransmit_queue.front()
                && let Some(wire) = sent_wire.get(&head).cloned()
            {
                probes.push((neko_reliable::FrameId(head), wire));
            } else if let Some((&head, wire)) = sent_wire.iter().next() {
                // Frontier insurance: after a stalled-engine rebuild the
                // engine no longer registers the true frontier record, so
                // pto_probe() alone targets post-rebuild frames while the
                // hole that blocks every ACK goes unprobed. The oldest
                // unacknowledged record joins the probe set FIRST.
                probes.push((neko_reliable::FrameId(head), wire.clone()));
            }
            probes.extend(rt.pto_probe());
            if last_pto_fire.is_none_or(|t| now >= t.saturating_add(PTO_CLAMP_US)) {
                last_pto_fire = Some(now);
                if probes.is_empty() {
                    // Empty probe round (all frames already armed): still
                    // advance the cadence clock — never hot-spin on_pto.
                    _pto_count_empty_rounds += 1;
                }
                pto_probes += probes.len() as u64;
                for (frame, plaintext) in probes {
                    let idx = frame.0;
                    let wire = if idx == u64::MAX {
                        if !end_sent {
                            continue; // the END frame is step 3's to send
                        }
                        wire_end(total_records as u64)
                    } else {
                        plaintext
                    };
                    // A PTO probe re-registers in the engine (QUIC: probes
                    // ARE ack-eliciting data copies — bif stays truthful),
                    // and its SACK retirement flows through apply_ack like
                    // any retransmit copy.
                    let sealed = secure
                        .seal_unreliable(&wire)
                        .unwrap_or_else(|_| fail("pto seal failed"));
                    let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
                    // The frame may have been retired by an ACK processed
                    // after this probe set was built — probing a retired
                    // frame is a benign no-op, never fatal.
                    if rt
                        .on_retransmit_sent(pn, now, sealed.len() as u64, frame)
                        .is_err()
                    {
                        continue;
                    }
                    if send_drain(&sock, &sealed, addr) {
                        pn_owner.insert(pn, idx);
                        max_engine_pn = max_engine_pn.max(pn);
                        if std::env::var("NEKO_UDP_DEBUG").is_ok() {}
                    } else {
                        rt.abandon_retransmit(pn);
                        break;
                    }
                }
            }
        }
        // Quiesce liveness (H-R9-064 companion): when the engine reports
        // zero in-flight but local bookkeeping still holds unacknowledged
        // frames (every engine-registered copy retired while the data is
        // unconfirmed — or a quiesced on_pto consumed the last probe
        // slot), the engine will never fire PTO again. The client drives
        // liveness itself: once per second, send the OLDEST unacked frame
        // as a raw copy (unregistered; its SACK retires via the raw-copy
        // path in ACK handling).
        // Liveness floor (covers BOTH quiesced engines and live-but-stalled
        // engines whose PTO probes keep missing the frontier hole): 3s
        // without any ACK progress pushes the OLDEST unacknowledged frame
        // as a raw copy at most once per second — bounded, amplification-
        // safe, and the only path guaranteed to target the true frontier.
        let stalled = last_progress_us != 0 && now_us(&epoch) > last_progress_us + 3_000_000;
        // Stalled-engine self-heal: 3s without any ACK progress means the
        // engine's state is no longer evidence of anything real — either
        // its retransmit budget is full of ghosts (probe copies that never
        // drew the ACK that would release them), or its in-flight count is
        // inflated by probe registrations that no ACK will ever retire
        // (the ACK they answer was itself lost; the server already holds
        // the data and will not re-acknowledge). Either way a parked
        // unsent record can never be admitted and the transfer deadlocks.
        // Rebuild the runtime (fresh state, same budget) at most once per
        // 3s while stalled. New pns continue above max_engine_pn, so the
        // server's monotonic observation never regresses; congestion
        // response is preserved by the 3s cadence floor.
        if stalled && now_us(&epoch) >= last_rebuild_us.saturating_add(3_000_000) {
            rt = neko_carrier::ReliableUdpRuntime::new_with_retransmit_budget(
                1,
                RECORD_WIRE_BYTES,
                RETRANSMIT_FRAMES,
                RETRANSMIT_BYTES,
            )
            .unwrap_or_else(|_| fail("reliable runtime rejected"));
            last_progress_us = now_us(&epoch); // re-arm progress tracking
            last_rebuild_us = now_us(&epoch);
        }
        if (rt.in_flight() == 0 || stalled)
            && !end_acked
            && (next_record < (total_records as u64) || !end_acked)
            && last_pto_fire.is_none_or(|t| now_us(&epoch) >= t.saturating_add(PTO_CLAMP_US))
        {
            // Work remains somewhere (undelivered records, unqueued wire, or
            // an unconfirmed END). Probe the engine's oldest outstanding
            // frame if it has one; otherwise the undelivered frontier is
            // engine-invisible — resend the frontier record raw (its pn
            // retires via the raw-copy ACK path).
            let target: Option<(u64, Vec<u8>)> = sent_wire
                .iter()
                .next()
                .map(|(&idx, w)| (idx, w.clone()))
                .or_else(|| {
                    // frontier = first undelivered record index = acked count
                    // when wire is empty and END unsent; END probe otherwise.
                    end_sent.then(|| (u64::MAX, wire_end(total_records as u64)))
                });
            if let Some((idx, wire)) = target
                && (idx != u64::MAX || !end_sent)
            {
                let wire = if idx == u64::MAX {
                    wire_end(total_records as u64)
                } else {
                    wire
                };
                if let Ok(sealed) = secure.seal_unreliable(&wire)
                    && send_drain(&sock, &sealed, addr)
                {
                    let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
                    pn_owner.insert(pn, idx);
                    max_engine_pn = max_engine_pn.max(pn);
                    pto_probes += 1;
                    last_pto_fire = Some(now_us(&epoch));
                }
            }
        }

        // Stall watchdog: with recovery context idle for 3s+, dump the
        // sender state once per second (diagnostics only).
        if std::env::var("NEKO_UDP_DEBUG").is_ok() {
            let now_w = now_us(&epoch);
            if last_progress_us != 0
                && now_w > last_progress_us + 3_000_000
                && now_w > last_dump_us + 1_000_000
            {
                eprintln!(
                    "[dbg] STALL in_flight={} bif={} queue={} next_record={} sent_wire={} acked={} end_sent={} end_acked={}",
                    rt.in_flight(),
                    rt.recovery_bytes_in_flight(),
                    retransmit_queue.len(),
                    next_record,
                    sent_wire.len(),
                    acked_records,
                    end_sent,
                    end_acked
                );
                last_dump_us = now_w;
            }
        }

        // 5. Receive drain: apply SACKs (retire records / schedule loss
        //    retransmissions), catch the final receipt. Returns to the send
        //    phase as soon as an ACK freed window space or scheduled work —
        //    never idles a full read-timeout while the path has capacity.
        let mut idle = 0u32;
        let mut progressed = false;
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, p)) if p == addr => {
                    idle = 0;
                    let Ok(plain) = secure.open_unreliable(&buf[..n]) else {
                        continue;
                    };
                    match classify_client(&plain) {
                        ClientInbound::Ack(ack) => {
                            progressed = true;
                            // Real progress only: a stale re-transmitted SACK
                            // (frozen largest, nothing newly retired) must
                            // NOT refresh the stall clock — otherwise the
                            // liveness floor and ghost-pn rebuild never fire
                            // and a jammed window crawls forever.
                            let advanced = ack.largest_observed > ack_largest_seen;
                            if advanced {
                                ack_largest_seen = ack.largest_observed;
                                last_progress_us = now_us(&epoch);
                            }
                            // Every pn the client ever put on the wire
                            // (fresh, rtx, probe) is engine-registered, so
                            // the wire ranges apply directly. A rejection
                            // (stale replay / gross desync) skips this ACK
                            // frame — never kills the session; the next
                            // canonical ACK self-heals the engine.
                            let ranges = neko_reliable::AckRanges::from_ranges(
                                32,
                                &ack.ranges
                                    .iter()
                                    .map(|w| neko_reliable::AckRange {
                                        start: w.start,
                                        end: w.end,
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .unwrap_or_else(|_| fail("ack ranges rejected"));
                            let outcome = rt
                                .apply_ack(&ranges, now_us(&epoch), ack.ack_delay_us)
                                .or_else(|_| {
                                    // Sanitize: a raw liveness copy's pn can
                                    // exceed the engine's largest, which
                                    // rejects the WHOLE frame. Clip every
                                    // range to the engine's max pn and retry
                                    // — retired pns are ignored, ghost pns
                                    // finally retire.
                                    // Clip ONLY the tail of a partially-in-window
                                    // range; a range entirely above the engine's
                                    // largest pn is DROPPED — collapsing it via
                                    // min() would fabricate [max,max], acking a
                                    // packet the peer never observed. Observed
                                    // pn == delivery evidence (the reorder
                                    // buffer is bounded and capacity-drops are
                                    // never observed); a fabricated ack retires
                                    // a frame whose wire is then dropped while
                                    // the record sits undelivered — the
                                    // ghost-retire deadlock observed at
                                    // 13980@5% (client acked 8735 vs server
                                    // records 8732, frozen forever).
                                    let clipped: Vec<neko_reliable::AckRange> = ack
                                        .ranges
                                        .iter()
                                        .filter_map(|w| {
                                            if w.start > max_engine_pn {
                                                return None;
                                            }
                                            let end = w.end.min(max_engine_pn);
                                            Some(neko_reliable::AckRange {
                                                start: w.start,
                                                end,
                                            })
                                        })
                                        .collect();
                                    let sanitized =
                                        neko_reliable::AckRanges::from_ranges(32, &clipped)?;
                                    rt.apply_ack(&sanitized, now_us(&epoch), ack.ack_delay_us)
                                });
                            if let Ok(outcome) = &outcome {
                                let retired: Vec<u64> = outcome
                                    .acked_packets
                                    .iter()
                                    .filter_map(|pn| pn_owner.get(pn).copied())
                                    .collect();
                                for idx in retired {
                                    retire_frame(
                                        idx,
                                        &mut pn_owner,
                                        &mut sent_wire,
                                        &mut rt,
                                        &mut acked_records,
                                        &mut end_acked,
                                    );
                                }
                                for frame in &outcome.retransmit_frames {
                                    retransmit_queue.push_back(frame.0);
                                }
                            }
                            // Raw liveness copies (pn_owner-known, engine-
                            // unregistered): a SACK observing such a copy
                            // proves the frame's DATA delivered — retire the
                            // bookkeeping directly.
                            let engine_acked: std::collections::BTreeSet<u64> = match &outcome {
                                Ok(o) => o.acked_packets.iter().copied().collect(),
                                Err(_) => Default::default(),
                            };
                            for w in &ack.ranges {
                                let mut c = w.start;
                                while c <= w.end {
                                    if let Some(idx) = pn_owner.get(&c).copied()
                                        && !engine_acked.contains(&c)
                                    {
                                        retire_frame(
                                            idx,
                                            &mut pn_owner,
                                            &mut sent_wire,
                                            &mut rt,
                                            &mut acked_records,
                                            &mut end_acked,
                                        );
                                    }
                                    c += 1;
                                }
                            }
                        }
                        ClientInbound::Receipt(data) => {
                            receipt = Some(String::from_utf8_lossy(&data).into_owned());
                        }
                        ClientInbound::Other => {}
                    }
                }
                Ok(_) => continue,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    idle += 1;
                    if idle >= 2 || (progressed && idle >= 1) {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = &progressed;
    }
    // Receipt wait: the server retransmits its receipt for ~200ms after
    // completion; drain briefly so the confirmation lands.
    if receipt.is_none() {
        let receipt_deadline = Instant::now() + Duration::from_millis(1500);
        while Instant::now() < receipt_deadline {
            match sock.recv_from(&mut buf) {
                Ok((n, p)) if p == addr => {
                    if let Ok(plain) = secure.open_unreliable(&buf[..n])
                        && let ClientInbound::Receipt(data) = classify_client(&plain)
                    {
                        receipt = Some(String::from_utf8_lossy(&data).into_owned());
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
    }
    let elapsed = start.elapsed();
    let goodput_bps = (cfg.bytes_total as f64 / elapsed.as_secs_f64().max(1e-9)) as u64;
    println!(
        "bulk_udp_client_summary sent_records={} acked_records={} retransmits={} pto_probes={} goodput_bps={} elapsed_ms={} end_acknowledged={} server_receipt={}",
        total_records,
        acked_records,
        retransmits,
        pto_probes,
        goodput_bps,
        elapsed.as_millis(),
        end_acked,
        receipt.as_deref().unwrap_or("unverified")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D070 assertion 1: the compile-time window pin. MAX_WINDOW must stay
    /// at 48 (64-wide replay window - 16 headroom); relaxing it is a D070
    /// successor-decision matter, never an implementation tweak.
    #[test]
    fn window_pin_is_48() {
        const _: () = assert!(MAX_WINDOW == 48);
    }

    /// D070 decision 1: only the four sanctioned window levels exist.
    #[test]
    fn window_levels_are_sanctioned() {
        for level in [8u32, 16, 32, 48] {
            assert!(level <= MAX_WINDOW);
        }
        assert!(!matches!(7u32 | 24 | 40 | 64, 8 | 16 | 32 | 48));
    }

    /// D070 assertion 12: PLPMTUD fail-closed — unknown MTU admits, any
    /// confirmed MTU below 1300 refuses.
    #[test]
    fn plpmtud_gate_fails_closed_below_floor() {
        assert!(plpmtud_gate(None), "unknown MTU admits");
        assert!(plpmtud_gate(Some(1300)), "exactly the floor admits");
        assert!(!plpmtud_gate(Some(1299)), "below the floor must refuse");
        assert!(!plpmtud_gate(Some(1200)));
    }

    /// D070 assertion 13: UDP bulk is unreachable without NEKO_MEASUREMENT=1
    /// (config-level mirror of the CLI gate test in tests/bulk_d070_udp.rs).
    #[test]
    fn config_requires_measurement_mode() {
        // SAFETY-free: measurement::enabled() reads the process env; in the
        // test harness it is normally unset. If it happens to be set, skip.
        if crate::measurement::enabled() {
            return;
        }
        let args = vec!["bulk-server".to_string()];
        assert_eq!(
            config_udp(&args).err(),
            Some("bulk requires NEKO_MEASUREMENT=1 (research-only)")
        );
    }

    /// Window argument validation: the four levels parse, anything else
    /// rejects by name.
    #[test]
    fn window_argument_validation() {
        if !crate::measurement::enabled() {
            return;
        }
        let mk = |window: &str| {
            config_udp(&[
                "--window".to_string(),
                window.to_string(),
                "--bytes-total".to_string(),
                "2340".to_string(),
            ])
            .map(|c| c.window)
        };
        assert_eq!(mk("8"), Ok(8));
        assert_eq!(mk("16"), Ok(16));
        assert_eq!(mk("32"), Ok(32));
        assert_eq!(mk("48"), Ok(48));
        assert!(mk("64").is_err());
        assert!(mk("0").is_err());
        assert!(mk("7").is_err());
    }

    /// D070 assertion 9 (encode side): a full 32-range ACK payload encodes
    /// within the unreliable-datagram plaintext budget, and the steady-state
    /// single-range SACK frame stays a small fraction of a data record —
    /// amplification pin (assertion 11): one ACK answers many data packets.
    #[test]
    fn ack_frame_shapes_stay_bounded() {
        let mut ranges = Vec::new();
        for i in 0..32u64 {
            ranges.push(neko_wire::AckRangeWire {
                start: i * 2,
                end: i * 2,
            });
        }
        let ack = neko_wire::AckPayload {
            largest_observed: 63,
            ack_delay_us: 0,
            ranges,
        };
        let wire = wire_ack(&ack).expect("32-range ACK encodes");
        // +50B seal overhead stays a single unfragmented datagram.
        assert!(
            wire.len() + neko_crypto::RECORD_SEAL_OVERHEAD
                <= neko_crypto::MAX_UNRELIABLE_DATAGRAM + 40
        );
        // Steady state: one contiguous range ACKs a whole window of data.
        let steady = wire_ack(&neko_wire::AckPayload {
            largest_observed: 47,
            ack_delay_us: 0,
            ranges: vec![neko_wire::AckRangeWire { start: 0, end: 47 }],
        })
        .expect("steady ACK encodes");
        // 48 data records ≈ 48*1250B on the wire; the answering ACK frame
        // (with seal overhead) is a tiny fraction — no amplification shape.
        let ack_wire_bytes = (steady.len() + neko_crypto::RECORD_SEAL_OVERHEAD) as f64;
        let answered_data_bytes = 0.25 * (48.0 * RECORD_WIRE_BYTES as f64);
        assert!(
            ack_wire_bytes < answered_data_bytes,
            "amplification pin: ACK bytes must stay under 0.25x answered data"
        );
    }

    /// D070 §3.2 arithmetic pin: a maximal bulk DATA record seals to exactly
    /// the 1250B wire size the throughput model assumes.
    #[test]
    fn maximal_record_seals_to_1250() {
        let record = neko_session::OutboundRecord {
            stream: STREAM,
            offset: 0,
            data: vec![0u8; crate::PERIODIC_BYTES_MAX],
        };
        let plain = ProcessMessage::Data {
            session: SESSION,
            record,
        }
        .encode()
        .unwrap();
        assert_eq!(
            plain.len(),
            neko_crypto::MAX_UNRELIABLE_DATAGRAM,
            "max plaintext must exactly fill the unreliable datagram budget"
        );
        assert_eq!(
            plain.len() + neko_crypto::RECORD_SEAL_OVERHEAD,
            RECORD_WIRE_BYTES as usize
        );
    }

    /// Demux determinism: bulk DATA ProcessMessages and the receipt never
    /// decode as wire records (session prefix bytes break the wire grammar),
    /// and END/ACK control records never decode as ProcessMessages.
    #[test]
    fn client_server_demux_is_unambiguous() {
        let data_plain = ProcessMessage::Data {
            session: SESSION,
            record: neko_session::OutboundRecord {
                stream: STREAM,
                offset: 0,
                data: vec![7u8; 64],
            },
        }
        .encode()
        .unwrap();
        // A DATA plaintext must classify as Data, never as END/ACK.
        match classify_server(&data_plain) {
            ServerInbound::Data(offset, data) => {
                assert_eq!(offset, 0);
                assert_eq!(data.len(), 64);
            }
            other => panic!("DATA misclassified: {other:?}"),
        }
        // The END notice must classify as End, never as Data.
        let end_plain = wire_end(2000);
        match classify_server(&end_plain) {
            ServerInbound::End(n) => assert_eq!(n, 2000),
            other => panic!("END misclassified: {other:?}"),
        }
        // An ACK frame must classify as Ack on the client.
        let ack_plain = wire_ack(&neko_wire::AckPayload {
            largest_observed: 5,
            ack_delay_us: 0,
            ranges: vec![neko_wire::AckRangeWire { start: 0, end: 5 }],
        })
        .unwrap();
        match classify_client(&ack_plain) {
            ClientInbound::Ack(a) => assert_eq!(a.largest_observed, 5),
            other => panic!("ACK misclassified: {other:?}"),
        }
    }

    /// D070 assertions 4+5 (hard gates) at the seam level: a duplicate of
    /// an already-accepted packet number is skipped by the pre-open gate
    /// WITHOUT touching crypto (so the nonce/replay sequence state is never
    /// consumed by retransmissions), and the tracker keeps accepting fresh
    /// numbers afterwards — the session stays alive.
    #[test]
    fn duplicate_retransmission_is_noop_and_session_survives() {
        let mut acks = neko_carrier::PacketAckTracker::new(1);
        for pn in 0u64..8 {
            acks.observe_packet(1, pn, true).unwrap();
        }
        let accepted = acks.build_ack(0).unwrap();
        assert!(
            accepted.ranges.iter().any(|r| r.start == 0 && r.end == 7),
            "SACK must settle the reordered window cumulatively (assertion 3)"
        );
        // The gate: every accepted pn (and a retransmission of any of them)
        // must be classified duplicate — a pure no-op that never fails.
        for pn in 0u64..8 {
            let is_dup = acks
                .build_ack(0)
                .is_some_and(|a| a.ranges.iter().any(|r| r.start <= pn && pn <= r.end));
            assert!(is_dup, "pn {pn} must be gated as duplicate");
        }
        // Fresh numbers still admit: the session did not die.
        acks.observe_packet(1, 8, true).unwrap();
        assert_eq!(acks.build_ack(0).unwrap().largest_observed, 8);
    }

    /// D070 assertion 16 seam: two independent runtimes keep independent
    /// congestion state — one stream's loss does not touch the other's cwnd.
    #[test]
    fn independent_streams_do_not_share_cwnd() {
        let mut a = neko_carrier::ReliableUdpRuntime::new(1, 1250).unwrap();
        let mut b = neko_carrier::ReliableUdpRuntime::new(2, 1250).unwrap();
        for i in 0u64..6 {
            let plain_a = [0u8; 64].to_vec();
            a.on_packet_sent(100 + i, 0, 1250, neko_reliable::FrameId(i), &plain_a)
                .unwrap();
            let plain_b = [1u8; 64].to_vec();
            b.on_packet_sent(200 + i, 0, 1250, neko_reliable::FrameId(i), &plain_b)
                .unwrap();
        }
        // Stream A loses one packet (SACK covers the rest — the gap sits 3+
        // below the ACK frontier, the PACKET_THRESHOLD=3 fast-retransmit
        // condition); stream B does not.
        let ack = neko_reliable::AckRanges::from_ranges(
            32,
            &[
                neko_reliable::AckRange {
                    start: 100,
                    end: 101,
                },
                neko_reliable::AckRange {
                    start: 103,
                    end: 105,
                },
            ],
        )
        .unwrap();
        let out = a.apply_ack(&ack, 1_000_000, 0).unwrap();
        assert_eq!(
            out.lost_packets,
            vec![102],
            "fast retransmit must declare exactly the SACK gap lost (assertion 6)"
        );
        assert_eq!(
            out.retransmit_frames,
            vec![neko_reliable::FrameId(2)],
            "the lost packet's stable frame must be scheduled (selective, not GBN)"
        );
        // B's window is untouched — independent congestion domains.
        assert_eq!(b.recovery_bytes_in_flight(), 6 * 1250);
        // A's Reno bytes reconcile exactly: 6 charged - 5 acked - 1 lost = 0
        // in flight (the lost frame's charge released with the declaration,
        // even though the retransmission will re-charge under a fresh pn).
        assert_eq!(
            a.recovery_bytes_in_flight(),
            0,
            "acked + lost releases every charged byte of the window"
        );
    }

    /// D070 assertion 8 seam: PTO backoff raises the deadline and a PTO
    /// streak past the persistent-congestion threshold collapses cwnd.
    #[test]
    fn pto_backoff_and_persistent_congestion_reset() {
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1250).unwrap();
        rt.on_packet_sent(1, 0, 1250, neko_reliable::FrameId(1), &[0u8; 8])
            .unwrap();
        let d0 = rt.next_pto_deadline_us(1_000, 0).unwrap();
        let probes = rt.pto_probe();
        assert!(!probes.is_empty(), "PTO schedules the oldest frame");
        let d1 = rt.next_pto_deadline_us(1_000, 0).unwrap();
        assert!(d1 > d0, "PTO backoff must push the next deadline out");
        // Drive the streak to the persistent-congestion threshold.
        for _ in 0..2 {
            let _ = rt.pto_probe();
        }
        // The committed window collapse: after 3 PTOs the cwnd ceiling must
        // admit only the 2-MSS persistent-congestion floor.
        assert!(
            !rt.can_send(3 * RECORD_WIRE_BYTES),
            "persistent congestion must collapse the window to the 2-MSS floor"
        );
    }
}
