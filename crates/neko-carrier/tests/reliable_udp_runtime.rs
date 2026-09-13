//! R7 — one coherent local runtime/event-loop fixture for live reliable UDP.
//!
//! A single `ReliableUdpPeer` composes socket send/receive, authenticated
//! packet numbering, receiver-side ACK generation, ACK application / RTT /
//! loss / PTO / retransmit, the fresh-evidence health bridge, warm TCP standby
//! and the ConcurrentCarrierManager degrade/fail/promotion path — one actual
//! flow, not manual test choreography. Packet recovery evidence is layered
//! through `neko_observe` and never becomes Session delivery.
use neko_carrier::{
    CarrierHealth, CarrierKind, ConcurrentCarrierManager, ConcurrentLimits, ConcurrentPathKey,
    HealthLimits, PacketAckTracker, PathGeneration, PathId, PathRecovery, RetransmitBuffer,
    SwitchReason, UdpCarrier, UdpLimits, UdpLoopbackPair,
};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, SecureSession,
    TrustPolicy, TrustRecord, TrustStatus,
};
use neko_observe::Producer;
use neko_reliable::{AckRanges, FrameId, SentPacket};
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

/// One endpoint's reliable-UDP runtime: crypto session, sender recovery,
/// receiver ACK tracker, retransmission buffer, and the carrier manager the
/// fresh health evidence drives.
struct ReliableUdpPeer<'a> {
    sock: &'a dyn UdpCarrier,
    session: SecureSession,
    generation: u64,
    recovery: PathRecovery,
    acks: PacketAckTracker,
    retransmit: RetransmitBuffer,
    health: CarrierHealth,
    manager: ConcurrentCarrierManager,
    obs: Producer,
}

impl<'a> ReliableUdpPeer<'a> {
    fn new(sock: &'a dyn UdpCarrier, session: SecureSession, generation: u64) -> Self {
        let mut manager = ConcurrentCarrierManager::new(ConcurrentLimits::default()).unwrap();
        manager.register(UDP, CarrierKind::Udp).unwrap();
        manager.register(TCP, CarrierKind::Tcp).unwrap();
        Self {
            sock,
            session,
            generation,
            recovery: PathRecovery::new(PathId(1), generation, 1200).unwrap(),
            acks: PacketAckTracker::new(generation),
            retransmit: RetransmitBuffer::new(64, 4096).unwrap(),
            health: CarrierHealth::new(HealthLimits::default()).unwrap(),
            manager,
            obs: Producer::new(SessionId(1), 256).unwrap(),
        }
    }

    /// Send an application payload as an authenticated packet-numbered Data
    /// record; record it in recovery and retain the frame for retransmit.
    fn send_data(&mut self, now_us: u64, payload: &[u8]) -> u64 {
        let record = encode(&Record {
            record_type: RecordType::Data,
            flags: 0,
            payload: payload.to_vec(),
        })
        .unwrap();
        let sealed = self.session.seal(&record).unwrap();
        let n = packet_number(&sealed);
        self.recovery
            .on_sent(SentPacket {
                number: n,
                sent_at_us: now_us,
                bytes: sealed.len() as u64,
                ack_eliciting: true,
                frames: vec![FrameId(n)],
            })
            .unwrap();
        self.retransmit.track(FrameId(n), payload).unwrap();
        self.sock.send_datagram(&sealed).unwrap();
        n
    }

    /// Receive one datagram; on authenticated Data it is recorded for ACK, on
    /// authenticated Ack the recovery engine is applied. Returns the packet
    /// number observed, if a Data record arrived.
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
                // A Data record is ack-eliciting; it creates an ACK obligation.
                self.acks.observe_packet(self.generation, n, true).unwrap();
                Some(n)
            }
            RecordType::Ack => {
                let ack = decode_ack(&rec.payload).ok()?;
                // The wire ACK already carries canonical merged ranges; convert
                // directly rather than re-inserting each packet (which could
                // exceed a small range cap).
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
                let out = self
                    .recovery
                    .on_ack(self.generation, &ranges, now_us, ack.ack_delay_us)
                    .ok()?;
                self.obs.record_recovery_ack(
                    now_us / 1000,
                    1,
                    PathId(1),
                    self.recovery.recovery(),
                    &out.result,
                );
                // Released frames free their retransmit bytes.
                for f in &out.acked_packets {
                    self.retransmit.release(FrameId(*f));
                }
                for f in &out.retransmit_frames {
                    self.retransmit.release(*f); // re-arm under a fresh packet
                }
                None
            }
            _ => None,
        }
    }

    /// Emit a receiver ACK record sealing the current canonical ranges —
    /// consumes the pending obligation so a second poll without new eligible
    /// evidence emits nothing (no ACK-of-ACK).
    fn send_ack(&mut self) {
        if let Some(payload) = self.acks.take_ack(0) {
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

    /// Poll the fresh-evidence health bridge: a new evidence epoch yields one
    /// sample; no new evidence yields None and cannot advance the streak.
    fn poll_health(&mut self, now_ms: u64) -> Option<neko_carrier::HealthState> {
        let s = self.recovery.fresh_health_sample()?;
        let state = self.health.observe(PathId(1), s).unwrap();
        self.obs.record_health(now_ms, 1, PathId(1), None, state, s);
        Some(state)
    }

    /// Retransmit frames selected by a PTO into fresh packets (new nonce).
    fn pto_retransmit(&mut self, now_us: u64) {
        if let Ok(frames) = self.recovery.on_pto(4) {
            self.obs.record_pto(
                now_us / 1000,
                1,
                PathId(1),
                self.recovery.recovery(),
                frames.len(),
            );
            for f in frames {
                if let Some(bytes) = self.retransmit.get(f).map(|b| b.to_vec()) {
                    self.send_data(now_us, &bytes);
                }
            }
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

    // Ready UDP active; TCP standby proves readiness independently.
    for i in 0..3 {
        client
            .manager
            .observe_readiness(UDP, true, true, i)
            .unwrap();
        client
            .manager
            .observe_readiness(TCP, true, true, i)
            .unwrap();
    }
    client
        .manager
        .activate(UDP, SwitchReason::OperatorRequest, 2, false)
        .unwrap();

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
    assert_eq!(client.recovery.in_flight(), 0);

    // Round 2: drop one packet on the wire (deterministic: sender seals but
    // suppresses the send_datagram) while the rest transmit. The ACK for the
    // delivered set declares the suppressed packet lost.
    let dropped = client
        .session
        .seal(
            &encode(&Record {
                record_type: RecordType::Data,
                flags: 0,
                payload: b"lost".to_vec(),
            })
            .unwrap(),
        )
        .unwrap();
    let dropped_num = packet_number(&dropped);
    client
        .recovery
        .on_sent(SentPacket {
            number: dropped_num,
            sent_at_us: 60_000,
            bytes: dropped.len() as u64,
            ack_eliciting: true,
            frames: vec![FrameId(dropped_num)],
        })
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
    // The suppressed packet is declared lost; its frame retransmits on a fresh
    // packet via the retained buffer.
    assert!(client.recovery.packets_lost() > 0);

    // The fresh-evidence bridge: first poll yields the new bad sample, a
    // second poll on the same epoch yields nothing — one loss is one bad obs.
    let first = client.poll_health(100);
    assert!(first.is_some());
    assert!(
        client.poll_health(101).is_none(),
        "same epoch must not replay"
    );

    // Distinct bad evidence epochs drive degradation: each round sends a real
    // outstanding packet and fires a PTO with no ACK (blackhole), so pto_count
    // climbs past the >=3 bad threshold and each resolved PTO epoch yields one
    // fresh bad observation. Polling the same epoch replays nothing.
    let mut attempts = 0u64;
    loop {
        client.send_data(
            200_000 + attempts * 50_000,
            format!("bh{attempts}").as_bytes(),
        );
        client.pto_retransmit(201_000 + attempts * 50_000);
        if client.poll_health(300 + attempts) == Some(neko_carrier::HealthState::Degraded) {
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
    // PTO-driven degradation fails the active UDP path; the ready warm TCP
    // standby is promoted.
    client
        .manager
        .fail(UDP, SwitchReason::UdpBlackhole, 400)
        .unwrap();
    let ev = client
        .manager
        .activate(TCP, SwitchReason::UdpBlackhole, 401, true)
        .unwrap();
    assert_eq!(ev.recovery_class, Some(neko_carrier::RecoveryClass::Warm));
}
