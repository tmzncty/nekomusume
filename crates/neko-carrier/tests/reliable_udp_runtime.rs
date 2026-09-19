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
use neko_carrier::{PathRecoveryError, RetransmitError};
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
    /// Congestion probe size consulted before sealing a send.
    cwnd_probe: u64,
    /// Recorded Carrier `FrameId` -> Session byte offset. Frame identity and
    /// Session logical offset are separate identities; retransmission must read
    /// the offset from this map instead of assuming `FrameId.0` is the offset.
    frame_offsets: std::collections::BTreeMap<FrameId, u64>,
    /// Application logical stream for stable delivery identity.
    stream: u32,
    /// Next application offset — the stable Session logical identity.
    next_offset: u64,
}

impl<'a> ReliableUdpPeer<'a> {
    fn new(sock: &'a dyn UdpCarrier, session: SecureSession, generation: u64) -> Self {
        Self::with_mss(sock, session, generation, 1200)
    }

    /// Same composition with an explicit MSS, so a test can reach a small
    /// congestion window deterministically without long send loops.
    fn with_mss(
        sock: &'a dyn UdpCarrier,
        session: SecureSession,
        generation: u64,
        mss: u64,
    ) -> Self {
        Self::with_limits(sock, session, generation, mss, 1200)
    }

    /// Same composition with an explicit MSS and admitted-send probe size.
    fn with_limits(
        sock: &'a dyn UdpCarrier,
        session: SecureSession,
        generation: u64,
        mss: u64,
        cwnd_probe: u64,
    ) -> Self {
        Self {
            sock,
            session,
            rt: neko_carrier::ReliableUdpRuntime::new(generation, mss).unwrap(),
            cwnd_probe,
            frame_offsets: std::collections::BTreeMap::new(),
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
        // Session logical identity is (SessionId, stream, byte_offset): the next
        // identity advances by the payload length, not by one record. The new
        // offset is committed only AFTER the send passed the Carrier admission
        // boundary, so a refused send consumes no Session byte space.
        let offset = self.next_offset;
        let number = self.send_data_at(now_us, offset, payload)?;
        // The send already happened: advancing the identity must not turn a
        // completed send into a `None` that callers read as "refused".
        self.next_offset = self
            .next_offset
            .checked_add(payload.len() as u64)
            .expect("fixture Session byte offset overflowed u64");
        Some(number)
    }

    /// Record which Session byte offset a Carrier frame carries. Re-binding one
    /// `FrameId` to a different offset fails loudly instead of silently keeping
    /// the last value.
    fn note_frame_offset(&mut self, frame: FrameId, offset: u64) {
        if let Some(existing) = self.frame_offsets.get(&frame) {
            assert_eq!(
                *existing, offset,
                "FrameId {frame:?} is already bound to Session offset {existing}, not {offset}"
            );
            return;
        }
        self.frame_offsets.insert(frame, offset);
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
        if !self.rt.can_send(self.cwnd_probe) {
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
        // Remember the Session identity this Carrier frame carries. A frame id
        // may not be re-bound to a different Session offset: that would make a
        // later retransmission re-encode the wrong logical identity.
        self.note_frame_offset(frame, offset);
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
            // keeps its stable Carrier identity. The Session byte offset comes
            // from the recorded mapping — never from `FrameId.0`, which need not
            // be the offset for every frame.
            let offset = *self
                .frame_offsets
                .get(&frame)
                .expect("retransmitted frame has a recorded Session byte offset");
            let msg = neko_session::ProcessMessage::Data {
                session: SessionId(1),
                record: neko_session::OutboundRecord {
                    stream: neko_session::StreamId(u64::from(self.stream)),
                    offset,
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
            .map(|m| m.observe_readiness(UDP, true, true, i));
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
    // The dropped packet is built through the raw runtime API, so its Carrier
    // frame gets no offset from the send path: record the Session identity it
    // would carry, otherwise a PTO selecting it would fail loudly.
    let dropped_frame = FrameId(9001);
    let dropped_offset = client.next_offset;
    client.note_frame_offset(dropped_frame, dropped_offset);
    client
        .rt
        .on_packet_sent(dropped_num, 60_000, 400, dropped_frame, b"lost")
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

// ---------------------------------------------------------------------------
// Q4 coherent R7 scenario suite.
//
// One real flow per scenario, all on the same composition (UDP socket pair,
// authenticated packet numbering, bounded plaintext ownership, resolved-outcome
// health, bounded Session-layer delivery above the Carrier). Packet recovery,
// health, switch and Session-delivery evidence stay distinct throughout.
// ---------------------------------------------------------------------------

/// Bring one peer to "UDP active with a ready warm TCP standby".
fn activate_with_ready_standby(peer: &mut ReliableUdpPeer) {
    peer.rt.ready_standby(0);
    for i in 0..3 {
        let _ = peer
            .rt
            .manager_mut()
            .map(|m| m.observe_readiness(UDP, true, true, i));
    }
    peer.rt.activate_udp(2);
}

#[test]
fn scenario_no_loss_round_trip_drains_in_flight_and_reports_zero_loss() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);
    activate_with_ready_standby(&mut client);

    // Variable-length records: the byte offsets used are 0, 2, 5, 6. Under
    // message-index offsets the second record would land below next_receive and
    // be rejected by the Session layer.
    let payloads: [&[u8]; 4] = [b"aa", b"bbb", b"c", b"dddd"];
    for (i, payload) in payloads.iter().enumerate() {
        assert!(
            client
                .send_data(10_000 + i as u64 * 1000, payload)
                .is_some(),
            "cwnd admits a fresh send"
        );
    }
    for i in 0..4 {
        server.recv_once(1000 + i * 100);
    }
    server.send_ack();
    client.recv_once(50_000);

    assert_eq!(client.rt.in_flight(), 0, "all packets retired");
    assert_eq!(client.rt.packets_lost(), 0, "no loss is declared");
    assert_eq!(
        server.session_delivered, 4,
        "four byte-accurate logical deliveries"
    );
    assert_eq!(
        server.session_duplicates, 0,
        "no duplicate suppression on a clean flow"
    );
    assert_eq!(
        server.session_conflicts, 0,
        "no Session rejection: the byte offsets are contiguous"
    );
    assert!(
        server.last_session_error.is_none(),
        "a clean flow retains no Session error"
    );
    match client.poll_health(100) {
        neko_carrier::RuntimeEvent::HealthSample(state) => assert!(
            !matches!(
                state,
                neko_carrier::HealthState::Degraded | neko_carrier::HealthState::Failed
            ),
            "a clean resolved interval must not degrade or fail the path"
        ),
        other => panic!("expected one fresh health sample, got {other:?}"),
    }
}

#[test]
fn scenario_lost_ack_recovers_via_pto_replacement_without_duplicate_delivery() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    client.send_data(0, b"ackloss");
    server.recv_once(0);
    assert_eq!(server.session_delivered, 1);
    // The receiver's ACK is produced but never reaches the sender: consumed
    // here and discarded, which is the deterministic lab-only ACK-loss seam.
    assert!(
        server.rt.poll_outgoing_ack(0).is_some(),
        "an ack-eliciting packet creates exactly one pending ACK"
    );
    assert_eq!(client.rt.in_flight(), 1, "sender still awaits resolution");

    // PTO selects the stable frame and the caller re-encodes it under a FRESH
    // packet number/nonce; the frame identity is unchanged.
    client.pto_retransmit(1000);
    server.recv_once(1000);
    assert_eq!(
        server.session_delivered, 1,
        "the replacement must not re-deliver the same logical identity"
    );
    assert_eq!(server.session_duplicates, 1, "duplicate suppressed once");
    server.send_ack();
    client.recv_once(5000);
    assert_eq!(
        client.rt.in_flight(),
        0,
        "replacement ACK retires the frame"
    );
    assert_eq!(client.rt.packets_lost(), 0, "no loss was declared");
}

#[test]
fn scenario_replacement_delivered_first_suppresses_the_late_original() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    client.send_data_at(0, 0, b"late");
    // The original never arrives: it is deterministically dropped on the wire
    // before the receiver can read it.
    assert!(server_sock.recv_datagram().unwrap().is_some());
    client.pto_retransmit(1000);
    server.recv_once(1000);
    assert_eq!(
        server.session_delivered, 1,
        "the retransmission delivers the logical identity first"
    );

    // A late byte-identical copy of the same logical identity (fresh packet
    // number) is suppressed by Session semantics, not redelivered.
    client.send_data_at(2000, 0, b"late");
    server.recv_once(2000);
    assert_eq!(
        server.session_delivered, 1,
        "late original not re-delivered"
    );
    assert_eq!(
        server.session_duplicates, 1,
        "late original suppressed once"
    );
    assert_eq!(
        server.session_conflicts, 0,
        "identical bytes are not a conflict"
    );
}

#[test]
fn scenario_plaintext_bound_refusal_is_atomic_and_typed() {
    // Oversized frame: the bounded owner refuses before any recovery packet,
    // Reno charge or packet->frame entry exists. Exit `FrameTooLarge` is
    // distinct from `Conflict` and `Capacity`.
    let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
    assert_eq!(
        rt.on_packet_sent(0, 0, 400, FrameId(1), &vec![b'x'; 8193]),
        Err(PathRecoveryError::Retransmit(
            RetransmitError::FrameTooLarge
        ))
    );
    assert_eq!(rt.in_flight(), 0, "refused send recorded no packet");
    assert_eq!(
        rt.recovery_bytes_in_flight(),
        0,
        "refused send charged no bytes"
    );
    assert!(rt.pto_probe().is_empty(), "refused frame owns no plaintext");
}

#[test]
fn scenario_clean_recovery_after_old_loss_does_not_replay_it() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    // Six single-byte records: enough spread for the recovery engine to declare
    // the older packets lost when only the newest is acknowledged.
    let mut numbers = Vec::new();
    for i in 0..6u64 {
        numbers.push(client.send_data(10_000 + i * 1000, b"r").unwrap());
    }
    for _ in 0..6 {
        server.recv_once(0);
    }
    // ACK only the newest packet: the older ones are newly declared lost.
    let mut only_newest = AckRanges::new(8).unwrap();
    only_newest.insert(*numbers.last().unwrap()).unwrap();
    client.rt.apply_ack(&only_newest, 50_000, 0).unwrap();
    let lost_after_bad = client.rt.packets_lost();
    assert!(lost_after_bad >= 1, "the older packets are declared lost");
    assert!(
        matches!(
            client.poll_health(100),
            neko_carrier::RuntimeEvent::HealthSample(_)
        ),
        "the newly resolved loss is fresh health evidence"
    );

    // A later ACK retires everything still outstanding: the interval has no new
    // loss, so the old loss must not be replayed as fresh bad evidence.
    let mut everything = AckRanges::new(8).unwrap();
    for n in &numbers {
        everything.insert(*n).unwrap();
    }
    client.rt.apply_ack(&everything, 60_000, 0).unwrap();
    assert_eq!(client.rt.in_flight(), 0, "recovery completed cleanly");
    assert_eq!(client.rt.packets_lost(), lost_after_bad, "no new loss");
    match client.poll_health(101) {
        neko_carrier::RuntimeEvent::HealthSample(state) => assert!(
            !matches!(
                state,
                neko_carrier::HealthState::Degraded | neko_carrier::HealthState::Failed
            ),
            "a clean resolved interval must not replay old loss as bad evidence"
        ),
        other => panic!("expected a fresh health sample, got {other:?}"),
    }
    assert_eq!(server.session_delivered, 6);
}

#[test]
fn scenario_invalid_standby_never_produces_a_false_switch() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, _ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    // Deliberately do NOT mark the TCP standby ready.
    for i in 0..3 {
        let _ = client
            .rt
            .manager_mut()
            .map(|m| m.observe_readiness(UDP, true, true, i));
    }
    client.rt.activate_udp(2);

    let mut switched = false;
    let mut degradation_attempted = false;
    for i in 0..12u64 {
        client.send_data(100_000 + i * 50_000, b"is");
        client.pto_retransmit(101_000 + i * 50_000);
        match client.poll_health(200 + i) {
            neko_carrier::RuntimeEvent::WarmFallback(_) => {
                switched = true;
                break;
            }
            neko_carrier::RuntimeEvent::FallbackFailed => degradation_attempted = true,
            _ => {}
        }
    }
    assert!(
        !switched,
        "no switch may be fabricated without a ready standby"
    );
    assert!(
        degradation_attempted,
        "the bad-evidence epochs must actually attempt the promotion"
    );
    assert_ne!(
        client.rt.manager().active(),
        Some(TCP),
        "TCP must never become active without a ready standby"
    );
}

#[test]
fn scenario_cwnd_refusal_consumes_no_session_byte_offset() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    // Small MSS -> small congestion window, so refusal is reached quickly.
    let mut client = ReliableUdpPeer::with_limits(&client_sock, cs, 1, 120, 32);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);

    // Fill the window with single-byte records; the first refused send must not
    // consume Session byte space.
    let mut admitted = 0u64;
    for i in 0..64u64 {
        if client.send_data(i * 100, b"p").is_none() {
            break;
        }
        admitted += 1;
    }
    assert!(admitted > 0, "some sends are admitted");
    assert!(admitted < 64, "the congestion window refuses eventually");
    assert!(
        client.send_data(9_000, b"p").is_none(),
        "a refused send returns None and records nothing"
    );

    // Free the window: the receiver ACKs everything it actually saw.
    for _ in 0..admitted {
        server.recv_once(0);
    }
    server.send_ack();
    client.recv_once(20_000);
    assert_eq!(
        server.session_delivered, admitted,
        "every admitted record was delivered exactly once"
    );

    // The next admitted send must reuse the byte offset the refused attempt
    // would have used. If the refusal had consumed the offset, this record would
    // arrive above the Session next_receive and be rejected as a protocol gap.
    let number = client.send_data(25_000, b"q");
    assert!(number.is_some(), "the freed window admits the next send");
    server.recv_once(25_000);
    assert_eq!(
        server.session_delivered,
        admitted + 1,
        "the post-refusal record is delivered, not skipped"
    );
    assert_eq!(
        server.session_conflicts, 0,
        "a refused send must not create a Session byte-offset gap"
    );
}

#[test]
fn scenario_pto_only_sample_does_not_erase_a_later_resolved_loss_in_the_coherent_path() {
    let (client_sock, server_sock) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (cs, ss) = pair(&client_sock, &server_sock);
    let mut client = ReliableUdpPeer::new(&client_sock, cs, 1);
    let mut server = ReliableUdpPeer::new(&server_sock, ss, 1);
    // A ready warm standby makes degradation observable as a real switch event.
    activate_with_ready_standby(&mut client);

    let mut degraded = false;
    // How many resolved-loss intervals were observed WITHOUT reaching Degraded.
    // `CarrierHealth` leaves the state unchanged for a single bad observation
    // while `consecutive_bad < degrade_after`, so the state itself cannot show
    // whether an interval was counted as bad. What it does show is that the
    // second consecutive bad interval crosses `degrade_after`: with the fix the
    // first interval is counted and the second degrades, so exactly one
    // non-degraded sample precedes it. The removed send-time denominator erases
    // the first interval to 0/mille Progress, which resets the bad streak and
    // makes the second interval non-degrading too — two non-degraded samples
    // and no degradation inside the bound.
    let mut non_degraded_samples = 0u32;
    for round in 0..2u64 {
        let base = 1_000 + round * 3_000;
        // Six small records give the recovery engine enough spread to declare
        // the older packets lost when only the newest one is acknowledged.
        let mut numbers = Vec::new();
        for i in 0..6u64 {
            numbers.push(
                client
                    .send_data(base + i * 10, b"L")
                    .expect("cwnd admits a fresh send"),
            );
        }
        for _ in 0..6 {
            server.recv_once(base);
        }

        if round == 0 {
            // One PTO-only interval keeps pto_count = 1 < 3, so this sample is a
            // loss-based observation, never a pto-threshold failure.
            client.pto_retransmit(base + 100);
            match client.poll_health(base + 110) {
                neko_carrier::RuntimeEvent::HealthSample(state) => assert!(
                    !matches!(
                        state,
                        neko_carrier::HealthState::Degraded | neko_carrier::HealthState::Failed
                    ),
                    "a single PTO interval must not degrade the path by itself"
                ),
                other => panic!("expected a fresh PTO health sample, got {other:?}"),
            }
        }

        // With no intervening send, this ACK newly declares the older packets
        // lost. A PTO-only sample must not have consumed this loss interval: if
        // the resolved loss were erased to zero, consecutive_bad would never
        // accumulate and Degraded would be unreachable.
        let mut only_newest = AckRanges::new(8).unwrap();
        only_newest.insert(*numbers.last().unwrap()).unwrap();
        client.rt.apply_ack(&only_newest, base + 200, 0).unwrap();
        match client.poll_health(base + 210) {
            neko_carrier::RuntimeEvent::Idle => {
                panic!("the resolved-loss epoch must remain observable after the PTO sample")
            }
            neko_carrier::RuntimeEvent::HealthSample(state) => {
                if matches!(
                    state,
                    neko_carrier::HealthState::Degraded | neko_carrier::HealthState::Failed
                ) {
                    degraded = true;
                    break;
                }
                non_degraded_samples += 1;
            }
            neko_carrier::RuntimeEvent::WarmFallback(_)
            | neko_carrier::RuntimeEvent::FallbackFailed => {
                degraded = true;
                break;
            }
        }
    }
    // Both consecutive loss intervals must actually be counted by the health
    // model: exactly one non-degraded sample precedes the degrading second
    // interval. Under the removed send-time denominator the first interval is
    // erased to Progress, so this count is 2 and nothing degrades.
    assert_eq!(
        non_degraded_samples, 1,
        "the loss branch must count the first interval and degrade on the second \
         (pto_count stays < 3, so this is not the pto-threshold branch)"
    );
    assert!(
        degraded,
        "two consecutive resolved-loss intervals must reach Degraded"
    );
}
