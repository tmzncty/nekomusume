//! R7 — one coherent local runtime/event-loop fixture for live reliable UDP.
//!
//! A single `ReliableUdpPeer` composes socket send/receive, authenticated
//! packet numbering, receiver-side ACK generation, ACK application / RTT /
//! loss / PTO / retransmit, the fresh-evidence health bridge, warm TCP standby
//! and the ConcurrentCarrierManager degrade/fail/promotion path — one actual
//! flow, not manual test choreography. Packet recovery evidence is layered
//! through `neko_observe` and never becomes Session delivery.
use neko_carrier::{
    ConcurrentPathKey, PathGeneration, PathId, UdpCarrier, UdpLimits, UdpLoopbackPair,
};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, SecureSession,
    TrustPolicy, TrustRecord, TrustStatus,
};
use neko_observe::Producer;
use neko_reliable::{AckRanges, FrameId};
use neko_session::SessionId;
use neko_wire::{Record, RecordType, decode, decode_ack, encode, encode_ack};
use std::{
    thread,
    time::{Duration, Instant},
};

const UDP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(1),
    generation: PathGeneration(1),
};
const TCP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(2),
    generation: PathGeneration(1),
};

fn context() -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 0,
        direction: 0,
    }
}
fn packet_number(record: &[u8]) -> u64 {
    u64::from_be_bytes(record[..8].try_into().unwrap())
}

/// One endpoint's reliable-UDP runtime: the socket/crypto harness wraps the
/// socket-agnostic `ReliableUdpRuntime` (recovery + ACK tracker + retransmit
/// ownership + freshness-enforced health + automatic manager fallback).
struct ReliableUdpPeer<'a> {
    sock: &'a dyn UdpCarrier,
    session: SecureSession,
    rt: neko_carrier::ReliableUdpRuntime,
    obs: Producer,
    /// Session-layer logical delivery owner. Bounded stream/offset delivery,
    /// exact-duplicate suppression and conflicting-duplicate fail-closed live
    /// here — above the Carrier, never inside packet recovery.
    session_rt: neko_session::SessionRuntime,
    /// Application deliveries accepted by the Session layer.
    session_delivered: u64,
    /// Exact logical duplicates suppressed by Session-layer semantics.
    session_duplicates: u64,
    /// Fail-closed Session rejections (e.g. conflicting duplicate bytes).
    session_conflicts: u64,
    /// Last Session rejection, retained so it is never silently swallowed.
    last_session_error: Option<neko_session::RuntimeError>,
    /// Application logical stream for stable delivery identity.
    stream: u32,
    /// Next application offset — the stable Session logical identity.
    next_offset: u64,
}

impl<'a> ReliableUdpPeer<'a> {
    fn new(sock: &'a dyn UdpCarrier, session: SecureSession, generation: u64) -> Self {
        Self {
            sock,
            session,
            rt: neko_carrier::ReliableUdpRuntime::new(generation, 1200).unwrap(),
            obs: Producer::new(SessionId(1), 256).unwrap(),
            session_rt: {
                let mut session_rt = neko_session::SessionRuntime::new(
                    SessionId(1),
                    neko_session::RuntimeLimits::default(),
                    0,
                )
                .unwrap();
                session_rt
                    .open_stream(neko_session::StreamId(1), 0)
                    .unwrap();
                session_rt
            },
            session_delivered: 0,
            session_duplicates: 0,
            session_conflicts: 0,
            last_session_error: None,
            stream: 1,
            next_offset: 0,
        }
    }

    /// Send an application payload under a stable logical identity as an
    /// authenticated packet-numbered Data record. The frame retains the stable
    /// `FrameId(offset)` while the packet carries a fresh AEAD nonce/number.
    /// Respects the cwnd admission gate before sealing.
    fn send_data(&mut self, now_us: u64, payload: &[u8]) -> Option<u64> {
        let offset = self.next_offset;
        self.next_offset += 1;
        self.send_data_at(now_us, offset, payload)
    }

    /// Send at an explicit stable logical offset — used by the conflict fixture
    /// so the same Session identity can be re-used with different bytes.
    fn send_data_at(&mut self, now_us: u64, offset: u64, payload: &[u8]) -> Option<u64> {
        self.send_data_frame(now_us, offset, FrameId(offset), payload)
    }

    /// Send with an explicit carrier-side stable frame identity. The Session
    /// logical identity in the payload is independent of the Carrier FrameId,
    /// which is what lets the conflict fixture reuse one logical identity with
    /// different bytes under a distinct frame.
    fn send_data_frame(
        &mut self,
        now_us: u64,
        offset: u64,
        frame: FrameId,
        payload: &[u8],
    ) -> Option<u64> {
        if !self.rt.can_send(1200) {
            return None; // congestion gate refused — nothing is sent/tracked
        }
        // Application data carries a stable Session logical identity inside
        // ProcessMessage::Data so retransmits dedup on (stream, offset).
        let msg = neko_session::ProcessMessage::Data {
            session: SessionId(1),
            record: neko_session::OutboundRecord {
                stream: neko_session::StreamId(u64::from(self.stream)),
                offset,
                data: payload.to_vec(),
            },
        };
        let payload_wire = msg.encode().unwrap();
        let record = encode(&Record {
            record_type: RecordType::Data,
            flags: 0,
            payload: payload_wire,
        })
        .unwrap();
        let sealed = self.session.seal(&record).unwrap();
        let n = packet_number(&sealed);
        self.rt
            .on_packet_sent(n, now_us, sealed.len() as u64, frame, payload)
            .unwrap();
        self.sock.send_datagram(&sealed).unwrap();
        Some(n)
    }

    /// Receive one datagram; on authenticated Data it records the packet for
    /// ACK and dedups the application payload by its stable logical identity;
    /// on authenticated Ack it applies recovery. Returns whether any Data was
    /// delivered (first time) this poll.
    fn recv_once(&mut self, now_us: u64) -> Option<u64> {
        let deadline = Instant::now() + Duration::from_millis(200);
        let bytes = loop {
            match self.sock.recv_datagram() {
                Ok(Some(b)) => break b,
                Ok(None) => {}
                Err(neko_carrier::UdpError::WouldBlock) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(1))
                }
                _ => return None,
            }
        };
        let plain = match self.session.open(&bytes) {
            Ok(p) => p,
            Err(_) => return None, // unauthenticated/tampered: never ACK/state
        };
        let rec = decode(&plain).ok()?;
        match rec.record_type {
            RecordType::Data => {
                let n = packet_number(&bytes);
                // Data is ack-eliciting: creates a pending ACK obligation.
                self.rt.on_packet_received(n, true).unwrap();
                // Decode the stable Session logical identity and dedup —
                // packet recovery/ACK is separate from Session delivery.
                if let Ok(neko_session::ProcessMessage::Data { record, .. }) =
                    neko_session::ProcessMessage::decode(&rec.payload)
                {
                    // Session-layer ownership: bounded stream/offset delivery.
                    // An exact duplicate of an already-delivered logical
                    // identity is suppressed by Session semantics; conflicting
                    // bytes for the same identity are a fail-closed error that
                    // is recorded here rather than swallowed with `let _`.
                    let before = self.session_rt.events().count();
                    match self.session_rt.receive(
                        neko_session::InboundRecord {
                            stream: record.stream,
                            offset: record.offset,
                            data: record.data.clone(),
                        },
                        now_us,
                    ) {
                        Ok(()) => {
                            let duplicate =
                                self.session_rt.events().skip(before).any(|e| {
                                    e.kind == neko_session::RuntimeEventKind::DuplicateDedup
                                });
                            if duplicate {
                                self.session_duplicates += 1;
                            } else {
                                self.session_delivered += 1;
                            }
                        }
                        Err(error) => {
                            self.session_conflicts += 1;
                            self.last_session_error = Some(error);
                        }
                    }
                }
                Some(n)
            }
            RecordType::Ack => {
                let ack = decode_ack(&rec.payload).ok()?;
                let ranges = AckRanges::from_ranges(
                    32,
                    &ack.ranges
                        .iter()
                        .map(|r| neko_reliable::AckRange {
                            start: r.start,
                            end: r.end,
                        })
                        .collect::<Vec<_>>(),
                )
                .ok()?;
                let out = self.rt.apply_ack(&ranges, now_us, ack.ack_delay_us).ok()?;
                self.obs.record_recovery_ack(
                    now_us / 1000,
                    1,
                    PathId(1),
                    self.rt.recovery_engine(),
                    &out.result,
                );
                None
            }
            _ => None,
        }
    }

    /// Emit a receiver ACK record sealing the current canonical ranges —
    /// consumes the pending obligation so a second poll without new eligible
    /// evidence emits nothing (no ACK-of-ACK).
    fn send_ack(&mut self) {
        if let Some(payload) = self.rt.poll_outgoing_ack(0) {
            let record = encode(&Record {
                record_type: RecordType::Ack,
                flags: 0,
                payload: encode_ack(&payload).unwrap(),
            })
            .unwrap();
            let sealed = self.session.seal(&record).unwrap();
            self.sock.send_datagram(&sealed).unwrap();
        }
    }

    /// Poll the freshness-enforced health bridge and drive the automatic
    /// fallback. Returns the runtime event (Idle / HealthSample /
    /// WarmFallback / FallbackFailed).
    fn poll_health(&mut self, now_ms: u64) -> neko_carrier::RuntimeEvent {
        self.rt.poll_health(now_ms)
    }

    /// Retransmit frames selected by a PTO into FRESH packets carrying the
    /// SAME stable FrameId (new packet number/nonce, stable identity).
    fn pto_retransmit(&mut self, now_us: u64) {
        for (frame, plaintext) in self.rt.pto_probe() {
            // Re-encode the retained plaintext under a fresh nonce; the frame
            // keeps its stable identity so overlapping-copy accounting holds.
            let msg = neko_session::ProcessMessage::Data {
                session: SessionId(1),
                record: neko_session::OutboundRecord {
                    stream: neko_session::StreamId(u64::from(self.stream)),
                    offset: frame.0, // frame id IS the stable logical offset
                    data: plaintext.clone(),
                },
            };
            let payload_wire = msg.encode().unwrap();
            let record = encode(&Record {
                record_type: RecordType::Data,
                flags: 0,
                payload: payload_wire,
            })
            .unwrap();
            let sealed = self.session.seal(&record).unwrap();
            let n = packet_number(&sealed);
            self.rt
                .on_retransmit_sent(n, now_us, sealed.len() as u64, frame)
                .unwrap();
            self.sock.send_datagram(&sealed).unwrap();
        }
    }
}

fn pair(client: &impl UdpCarrier, server: &impl UdpCarrier) -> (SecureSession, SecureSession) {
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"echo", b"rel").unwrap();
    client.send_datagram(&ih.first_message().unwrap()).unwrap();
    let rh = ResponderHandshake::new(&si, policy, b"rel").unwrap();
    let (resp, ss) = rh.receive_first(&recv(server), context()).unwrap();
    server.send_datagram(&resp).unwrap();
    let cs = ih.finish(&recv(client), context()).unwrap();
    (cs, ss)
}
fn recv(endpoint: &impl UdpCarrier) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match endpoint.recv_datagram() {
            Ok(Some(v)) => return v,
            Err(neko_carrier::UdpError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(1))
            }
            other => panic!("recv: {other:?}"),
        }
    }
}

#[test]
fn runtime_event_loop_recovers_loss_and_drives_degradation_only_on_new_evidence() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    // Ready UDP active + TCP standby ready (the runtime owns both paths).
    client.rt.ready_standby(0);
    // UDP must also prove readiness to be admissible active.
    for i in 0..3 {
        let _ = client
            .rt
            .manager_mut()
            .observe_readiness(UDP, true, true, i);
    }
    client.rt.activate_udp(2);

    // Round 1: send 4 data packets; server observes each authenticated packet
    // and ACKs; client applies — clean, in-flight drains, no retransmit.
    for i in 0..4u64 {
        client.send_data(10_000 + i * 1000, format!("d{i}").as_bytes());
    }
    for _ in 0..4 {
        server.recv_once(0);
    }
    server.send_ack();
    client.recv_once(50_000);
    assert_eq!(client.rt.in_flight(), 0);

    // Round 2: drop one packet on the wire (deterministic lab-only sender-side
    // suppression: seal a packet and record it in recovery but never send it)
    // while the rest transmit. The receiver's ACK for the delivered set then
    // declares the suppressed packet lost — controlled loss, not natural.
    let dropped_num = packet_number(
        &client
            .session
            .seal(
                &encode(&Record {
                    record_type: RecordType::Data,
                    flags: 0,
                    payload: b"lost".to_vec(),
                })
                .unwrap(),
            )
            .unwrap(),
    );
    client
        .rt
        .on_packet_sent(dropped_num, 60_000, 400, FrameId(9001), b"lost")
        .unwrap();
    // never sent on the wire
    for i in 0..3u64 {
        client.send_data(61_000 + i * 1000, format!("e{i}").as_bytes());
    }
    for _ in 0..3 {
        server.recv_once(0);
    }
    server.send_ack();
    client.recv_once(90_000);
    assert!(client.rt.packets_lost() > 0);

    // Fresh-evidence bridge: first poll yields the new bad sample; a second
    // poll on the same epoch yields nothing — one loss is one bad obs.
    let first = client.poll_health(100);
    assert!(
        matches!(first, neko_carrier::RuntimeEvent::HealthSample(_)),
        "resolved loss outcome emits one fresh health observation"
    );
    assert!(
        matches!(client.poll_health(101), neko_carrier::RuntimeEvent::Idle),
        "same epoch must not replay"
    );

    // Distinct bad evidence epochs drive degradation: each round sends a real
    // outstanding packet and fires a PTO with no ACK (blackhole), so pto_count
    // climbs past the >=3 bad threshold; each resolved PTO epoch yields one
    // fresh bad observation, and the bridge auto-promotes the warm TCP standby.
    let mut attempts = 0u64;
    loop {
        client.send_data(
            200_000 + attempts * 50_000,
            format!("bh{attempts}").as_bytes(),
        );
        client.pto_retransmit(201_000 + attempts * 50_000);
        if matches!(
            client.poll_health(300 + attempts),
            neko_carrier::RuntimeEvent::WarmFallback(_)
        ) {
            break;
        }
        attempts += 1;
        if attempts > 16 {
            panic!("no degradation under distinct new bad evidence");
        }
    }
    assert!(
        attempts >= 1,
        "degrade needs >= degrade_after new bad epochs"
    );
    // The fresh-evidence health bridge already drove the automatic fallback
    // inside poll_health: UDP failed and the warm TCP standby is now active.
    assert_eq!(client.rt.manager().active(), Some(TCP));
}

#[test]
fn session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    // First delivery of logical identity (stream 1, offset 0).
    client.send_data_at(0, 0, b"alpha");
    server.recv_once(0);
    assert_eq!(server.session_delivered, 1, "first delivery accepted");
    assert_eq!(server.session_conflicts, 0);

    // An exact duplicate of the same logical identity is suppressed by Session
    // semantics: delivered once, counted once, never an error.
    client.send_data_at(1000, 0, b"alpha");
    server.recv_once(1000);
    assert_eq!(
        server.session_delivered, 1,
        "exact duplicate of a delivered identity is not redelivered"
    );
    assert_eq!(server.session_duplicates, 1, "exact duplicate counted once");
    assert_eq!(server.session_conflicts, 0);

    // Conflicting bytes for the same logical identity fail closed and are
    // retained for inspection — never swallowed. A distinct Carrier frame keeps
    // carrier plaintext ownership legal while the Session identity collides.
    client.send_data_frame(2000, 0, FrameId(5000), b"tampered");
    server.recv_once(2000);
    assert_eq!(
        server.session_conflicts, 1,
        "conflicting duplicate fails closed at the Session layer"
    );
    assert_eq!(
        server.session_delivered, 1,
        "a conflicting duplicate is never delivered"
    );
    assert!(
        server.last_session_error.is_some(),
        "the Session rejection is retained, not swallowed with `let _`"
    );

    // Packet recovery/ACK evidence remains separate from Session delivery.
    server.send_ack();
    client.recv_once(3000);
    assert_eq!(server.session_delivered, 1);
    assert_eq!(server.session_conflicts, 1);
}
