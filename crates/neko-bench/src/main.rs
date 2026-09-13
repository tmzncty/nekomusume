//! Bounded, local-only Era-4-I performance evidence. This is measurement, not
//! a comparison or release claim. Run with `cargo run -p neko-bench --release`.
use neko_carrier::{FairScheduler, FlowLimits, StreamId, StreamPriority};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, SecureSession,
    TrustPolicy, TrustRecord, TrustStatus,
};
use neko_reliable::{AckRanges, FrameId, Recovery, SentPacket};
use neko_wire::{Frame, Record, RecordType, decode, decode_frames, encode, encode_frames};
use std::time::Instant;

const DEFAULT_ITERS: usize = 1_000;
const MAX_ITERS: usize = 10_000;

#[derive(Clone, Copy)]
struct Stat {
    n: usize,
    median: u128,
    p95: u128,
    failures: usize,
}
fn stat(mut xs: Vec<u128>, failures: usize) -> Stat {
    xs.sort_unstable();
    let n = xs.len();
    // Align with the repository's existing netns P95 convention: the order
    // statistic index is `round((n - 1) * 0.95)` on sorted successful samples
    // (index 94 for n=100, 949 for n=1000), not `n * 95 / 100` (index 95).
    // Median stays the lower-middle `n / 2`, matching the same convention.
    let p95_index = if n == 0 {
        0
    } else {
        ((n - 1) as f64 * 0.95).round() as usize
    };
    Stat {
        n,
        median: xs.get(n / 2).copied().unwrap_or(0),
        p95: xs.get(p95_index).copied().unwrap_or(0),
        failures,
    }
}
fn sample<F: FnMut() -> Result<(), ()>>(iters: usize, mut f: F) -> Stat {
    let mut xs = Vec::with_capacity(iters);
    let mut failures = 0;
    for _ in 0..iters {
        let t = Instant::now();
        if f().is_ok() {
            xs.push(t.elapsed().as_nanos());
        } else {
            failures += 1;
        }
    }
    stat(xs, failures)
}
fn emit(name: &str, s: Stat) {
    println!(
        "{{\"benchmark\":\"{name}\",\"iterations\":{},\"median_ns\":{},\"p95_ns\":{},\"failures\":{}}}",
        s.n, s.median, s.p95, s.failures
    );
}
fn context() -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 1,
        direction: 0,
    }
}
fn sessions() -> Result<(SecureSession, SecureSession), ()> {
    let i = LocalIdentity::generate().map_err(|_| ())?;
    let r = LocalIdentity::generate().map_err(|_| ())?;
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: i.public_key().to_vec(),
        scope: b"bench".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut ih =
        InitiatorHandshake::new(&i, r.public_key(), b"bench", b"era4-i").map_err(|_| ())?;
    let first = ih.first_message().map_err(|_| ())?;
    let rh = ResponderHandshake::new(&r, policy, b"era4-i").map_err(|_| ())?;
    let (reply, rs) = rh.receive_first(&first, context()).map_err(|_| ())?;
    let is = ih.finish(&reply, context()).map_err(|_| ())?;
    Ok((is, rs))
}
fn main() {
    let iters = std::env::var("NEKO_BENCH_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_ITERS)
        .clamp(1, MAX_ITERS);
    println!(
        "{{\"schema\":\"era4-i-performance.v1\",\"iterations\":{},\"unit\":\"nanoseconds per operation\",\"claims\":\"bounded local evidence only; no superiority claim\"}}",
        iters
    );
    let frames = vec![
        Frame::Data(vec![7; 128]),
        Frame::Datagram(vec![3; 64]),
        Frame::PathChallenge([9; 8]),
    ];
    let wire = encode_frames(&frames).unwrap();
    emit(
        "encode_decode",
        sample(iters, || {
            let x = encode_frames(&frames).map_err(|_| ())?;
            let y = decode_frames(&x).map_err(|_| ())?;
            if y == frames { Ok(()) } else { Err(()) }
        }),
    );
    let rec = Record {
        record_type: RecordType::Data,
        flags: 0,
        payload: vec![4; 256],
    };
    let _encoded = encode(&rec).unwrap();
    emit(
        "outer_record_encode_decode",
        sample(iters, || {
            let x = encode(&rec).map_err(|_| ())?;
            decode(&x).map_err(|_| ()).map(|_| ())
        }),
    );
    let (mut tx, mut rx) = sessions().expect("crypto setup");
    let payload = vec![5; 256];
    let _sealed = tx.seal(&payload).unwrap();
    emit(
        "crypto_seal",
        sample(iters, || tx.seal(&payload).map(|_| ()).map_err(|_| ())),
    );
    let mut records = Vec::with_capacity(iters);
    for _ in 0..iters {
        records.push(tx.seal(&payload).map_err(|_| ()).unwrap());
    }
    let mut index = 0usize;
    emit(
        "crypto_open",
        sample(iters, || {
            let x = rx.open(&records[index]).map_err(|_| ())?;
            index += 1;
            if x == payload { Ok(()) } else { Err(()) }
        }),
    );
    emit(
        "scheduler_next_frame",
        sample(iters, || {
            let mut s = FairScheduler::new(FlowLimits::default()).map_err(|_| ())?;
            s.open(StreamId(1), StreamPriority::Interactive)
                .map_err(|_| ())?;
            s.open(StreamId(2), StreamPriority::Bulk).map_err(|_| ())?;
            s.enqueue(StreamId(1), &wire).map_err(|_| ())?;
            s.enqueue(StreamId(2), &wire).map_err(|_| ())?;
            s.next_frame().map(|_| ()).ok_or(())
        }),
    );
    emit(
        "recovery_ack_loss",
        sample(iters, || {
            let mut r = Recovery::new(64, 8).map_err(|_| ())?;
            for n in 0..4 {
                r.on_sent(SentPacket {
                    number: n,
                    sent_at_us: n * 1000,
                    bytes: 256,
                    ack_eliciting: true,
                    frames: vec![FrameId(n)],
                })
                .map_err(|_| ())?;
            }
            let mut a = AckRanges::new(8).map_err(|_| ())?;
            a.insert(3).map_err(|_| ())?;
            r.on_ack(&a, 10_000, 0).map(|_| ()).map_err(|_| ())
        }),
    );
    // Instrumentation proxy: same scheduler operation with an explicit counter; the
    // delta is overhead of this harness counter only, not a runtime-wide claim.
    let mut count = 0usize;
    emit(
        "instrumentation_counter",
        sample(iters, || {
            count = count.wrapping_add(1);
            Ok(())
        }),
    );
    println!(
        "{{\"resource_note\":\"CPU/memory process-wide accounting is intentionally not reported: this slice has no stable cross-platform sampler.\"}}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_operations_are_excluded_from_latency_distribution() {
        // median/P95 must come from successful samples only; a failing op must
        // count in `failures` but never depress the latency distribution.
        let mut call = 0usize;
        let s = sample(10, || {
            call += 1;
            // First half succeed after a tiny busy window; second half fail fast.
            if call <= 5 { Ok(()) } else { Err(()) }
        });
        assert_eq!(s.n, 5, "only successful samples are timed");
        assert_eq!(s.failures, 5);
    }

    #[test]
    fn empty_distribution_reports_zero_not_panic() {
        let s = stat(Vec::new(), 3);
        assert_eq!(s.n, 0);
        assert_eq!(s.median, 0);
        assert_eq!(s.p95, 0);
        assert_eq!(s.failures, 3);
    }

    #[test]
    fn p95_order_statistic_matches_repository_convention() {
        // P95 index is round((n-1) * 0.95) on sorted samples — the same order
        // statistic used by run-netns.sh — so the convention is reproducible.
        for (n, expected_index) in [(1usize, 0usize), (2, 1), (20, 18), (100, 94), (1000, 949)] {
            // Distinct ascending values let the chosen index be read back.
            let xs: Vec<u128> = (0..n as u128).map(|v| v + 1).collect();
            let s = stat(xs, 0);
            assert_eq!(
                s.p95,
                (expected_index + 1) as u128,
                "n={n} expected index {expected_index}"
            );
            assert_eq!(s.median, (n / 2 + 1) as u128, "n={n} median index n/2");
        }
    }
}
