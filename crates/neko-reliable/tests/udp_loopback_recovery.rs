use neko_carrier::{UdpCarrier, UdpError, UdpLimits, UdpLoopbackPair};
use neko_reliable::{AckRanges, FrameId, Recovery, SentPacket};
use std::{
    thread,
    time::{Duration, Instant},
};

fn recv(endpoint: &impl UdpCarrier) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match endpoint.recv_datagram() {
            Ok(Some(bytes)) => return bytes,
            Err(UdpError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(1))
            }
            other => panic!("loopback receive failed: {other:?}"),
        }
    }
}

fn packet(number: u64, frame: u64) -> SentPacket {
    SentPacket {
        number,
        sent_at_us: 0,
        bytes: 8,
        ack_eliciting: true,
        frames: vec![FrameId(frame)],
    }
}

#[test]
fn loopback_drop_then_retransmit_preserves_recovery_evidence() {
    let (sender, receiver) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 64,
    })
    .unwrap();
    let mut recovery = Recovery::new(8, 1).unwrap();
    for number in 0..=3 {
        recovery.on_sent(packet(number, number)).unwrap();
        sender.send_datagram(&number.to_be_bytes()).unwrap();
    }
    assert_eq!(recv(&receiver), 0u64.to_be_bytes());
    for expected in 1u64..=3 {
        assert_eq!(recv(&receiver), expected.to_be_bytes());
    }

    let mut ack = AckRanges::new(1).unwrap();
    ack.insert(1).unwrap();
    ack.insert(2).unwrap();
    ack.insert(3).unwrap();
    let first = recovery.on_ack(&ack, 10_000, 0).unwrap();
    assert_eq!(first.acked_packets, vec![1, 2, 3]);
    assert_eq!(first.lost_packets, vec![0]);
    assert_eq!(first.retransmit_frames, vec![FrameId(0)]);

    recovery.on_sent(packet(4, 0)).unwrap();
    sender.send_datagram(&4u64.to_be_bytes()).unwrap();
    assert_eq!(recv(&receiver), 4u64.to_be_bytes());
    let mut retransmit_ack = AckRanges::new(1).unwrap();
    retransmit_ack.insert(4).unwrap();
    let second = recovery.on_ack(&retransmit_ack, 20_000, 0).unwrap();
    assert_eq!(second.acked_packets, vec![4]);
    assert!(second.retransmit_frames.is_empty());
    assert_eq!(recovery.in_flight(), 0);
}

#[test]
fn packet_numbers_fail_closed_at_lifetime_boundary() {
    let mut numbers = neko_reliable::PacketNumbers::new(u64::MAX - 1);
    assert_eq!(numbers.allocate(), Ok(u64::MAX - 1));
    assert_eq!(numbers.allocate(), Ok(u64::MAX));
    assert!(numbers.exhausted());
    assert_eq!(numbers.allocate(), Err(neko_reliable::Error::Exhausted));
}

#[test]
fn loss_bytes_and_persistent_congestion_are_observable_and_bounded() {
    let mut recovery = Recovery::new(8, 1).unwrap();
    for n in 0..=3 {
        recovery
            .on_sent(SentPacket {
                number: n,
                sent_at_us: 0,
                bytes: 100,
                ack_eliciting: true,
                frames: vec![FrameId(n)],
            })
            .unwrap();
    }
    let mut ack = AckRanges::new(1).unwrap();
    ack.insert(3).unwrap();
    let result = recovery.on_ack(&ack, 10_000, 0).unwrap();
    assert_eq!(result.retransmit_bytes, 100);
    for _ in 0..3 {
        recovery.on_pto(1).unwrap();
    }
    assert_eq!(recovery.persistent_congestion_events, 1);
    let mut congestion = neko_reliable::Reno::new(1200).unwrap();
    congestion.on_persistent_congestion();
    assert_eq!(congestion.cwnd, 2400);
    assert!(congestion.persistent_congestion(3));
}
