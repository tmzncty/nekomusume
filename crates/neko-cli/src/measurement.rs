//! Opt-in measurement-mode limit relaxation for limit research only.
//!
//! Default behavior is byte-for-byte identical to the plain CLI: nothing in
//! this module applies unless the environment variable `NEKO_MEASUREMENT`
//! is exactly `1`. When it is enabled, per-command ceilings rise to sit
//! exactly at the `neko_session` library hard limits — the research ceiling
//! is the library floor, not a new protocol surface:
//!
//! - session/workload duration: 30 s / 600 s -> 86_400 s (24 h)
//! - exchange count (`--count`): 64 -> 13_980, the largest value whose
//!   derived runtime limits (`count + 2` records, `bytes * (count + 1)`
//!   queue bytes at the full 1200-byte payload) still fit
//!   `HARD_MAX_RUNTIME_QUEUE_RECORDS` and `HARD_MAX_RUNTIME_QUEUE_BYTES`
//!   for every admissible payload size
//! - periodic count: 600 -> 65_530 (`count + 2 <= HARD_MAX_RUNTIME_QUEUE_
//!   RECORDS`); the periodic config additionally enforces the derived
//!   `bytes * (count + 1) <= HARD_MAX_RUNTIME_QUEUE_BYTES` ceiling, so large
//!   counts pair with small payloads
//! - periodic application bytes: 1 MiB -> 32 MiB. The periodic runtime sets
//!   `max_total_bytes = bytes * count * 2` and `total_bytes` is a lifetime
//!   accumulator that never decreases (ACK/pop release queue slots, not
//!   lifetime bytes), so 32 MiB of application data is the largest value
//!   that keeps the runtime limit inside `HARD_MAX_RUNTIME_TOTAL_BYTES`
//! - multistream shape: streams 16 -> 256 (inside `HARD_MAX_RUNTIME_
//!   STREAMS` = 4096; 256 is the review-agreed research ceiling), records
//!   per stream 64 -> min(8_192, 65_535 / streams) (the runtime declares
//!   `max_queue_records = streams * records + 1`, which must stay at or
//!   below `HARD_MAX_RUNTIME_QUEUE_RECORDS` = 65_536 as a declared value),
//!   record bytes 1024 -> 4000 (`PROCESS_DATA_HEADER_LEN + payload` must
//!   stay within `PROCESS_FRAME_MAX` = 4096), total payload 1 MiB ->
//!   `HARD_MAX_RUNTIME_TOTAL_BYTES` (64 MiB; lifetime semantics again).
//!
//! Every number below is pinned to the library constants by unit test, so a
//! library change that moves a hard limit fails the build here instead of
//! silently breaking the measurement contract.

/// True only when `NEKO_MEASUREMENT` is exactly `1`.
pub(super) fn enabled() -> bool {
    std::env::var_os("NEKO_MEASUREMENT").is_some_and(|v| v == "1")
}

/// Relaxed duration ceiling (24 h), replacing MAX_DURATION=30 and
/// MAX_WORKLOAD_DURATION=600 in measurement mode.
pub(super) const DURATION_MAX: u64 = 86_400;

/// Compile-time pin of the relaxed ceilings against the library constants:
/// if a neko_session hard limit moves, the build fails here instead of the
/// measurement contract silently drifting.
const _: () = {
    // Multistream record bytes: the exact data-frame header + payload must
    // fit PROCESS_FRAME_MAX, and the payload fits the per-record hard limit.
    assert!(
        neko_session::PROCESS_DATA_HEADER_LEN + MULTISTREAM_RECORD_BYTES_MAX
            <= neko_session::PROCESS_FRAME_MAX
    );
    assert!(MULTISTREAM_RECORD_BYTES_MAX <= neko_session::HARD_MAX_RUNTIME_RECORD_BYTES);
    // Multistream streams: inside the stream hard limit, and the paired
    // records value keeps the declared queue-record total inside the limit.
    assert!(MULTISTREAM_STREAMS_MAX <= neko_session::HARD_MAX_RUNTIME_STREAMS);
    assert!(
        MULTISTREAM_STREAMS_MAX
            * ((neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS - 1) / MULTISTREAM_STREAMS_MAX)
            < neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS
    );
    // Periodic application bytes: the runtime derives max_total_bytes =
    // 2 * app bytes (lifetime accumulator), so 32 MiB fits the 64 MiB limit.
    assert!(PERIODIC_TOTAL_BYTES_MAX * 2 <= neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES as u64);
    // Periodic count: count + 2 declared queue records.
    assert!(PERIODIC_COUNT_MAX + 2 <= neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS as u64);
    // Exchange count: every derived runtime limit at the maximum payload
    // (1200 B) — count + 2 records, 1200 * (count + 1) queue bytes,
    // 1200 * count lifetime bytes.
    assert!(EXCHANGE_COUNT_MAX + 2 <= neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS as u64);
    assert!(1200 * (EXCHANGE_COUNT_MAX + 1) <= neko_session::HARD_MAX_RUNTIME_QUEUE_BYTES as u64);
    assert!(1200 * EXCHANGE_COUNT_MAX <= neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES as u64);
};

/// Largest `--count` whose derived runtime limits fit the library hard
/// ceilings for every admissible payload (bytes <= 1200):
/// `count + 2 <= HARD_MAX_RUNTIME_QUEUE_RECORDS` and
/// `1200 * (count + 1) <= HARD_MAX_RUNTIME_QUEUE_BYTES` (at 13_981 the
/// queue-byte product overshoots 16 MiB by 1_184 bytes; 13_980 leaves 16).
pub(super) const EXCHANGE_COUNT_MAX: u64 = 13_980;

/// Largest periodic `--count`: `count + 2 <= HARD_MAX_RUNTIME_QUEUE_
/// RECORDS`. Pairing with the payload is enforced separately by the
/// periodic config (derived queue-bytes ceiling).
pub(super) const PERIODIC_COUNT_MAX: u64 = 65_530;

/// Relaxed periodic application-byte ceiling: the runtime derives
/// `max_total_bytes = bytes * count * 2`, so 32 MiB of application data
/// is the largest ceiling that stays inside HARD_MAX_RUNTIME_TOTAL_BYTES.
pub(super) const PERIODIC_TOTAL_BYTES_MAX: u64 = 32 << 20;

/// Multistream stream-count ceiling (review-agreed research value, inside
/// HARD_MAX_RUNTIME_STREAMS = 4096).
pub(super) const MULTISTREAM_STREAMS_MAX: usize = 256;

/// Multistream per-record payload ceiling: the exact process-data frame
/// header (PROCESS_DATA_HEADER_LEN = 30) plus the payload must stay within
/// PROCESS_FRAME_MAX (4096); 4000 leaves 66 bytes of encoding headroom on
/// top of the exact data-frame header.
pub(super) const MULTISTREAM_RECORD_BYTES_MAX: usize = 4_000;

/// Multistream total-payload ceiling: the runtime declares
/// `max_total_bytes = streams * records * bytes` (lifetime semantics),
/// which must stay at or below HARD_MAX_RUNTIME_TOTAL_BYTES.
pub(super) const MULTISTREAM_TOTAL_MAX: usize = neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES;

/// Records-per-stream ceiling for a given stream count: the runtime
/// declares `max_queue_records = streams * records + 1`, whose value must
/// stay at or below HARD_MAX_RUNTIME_QUEUE_RECORDS.
pub(super) fn multistream_records_max(streams: usize) -> usize {
    8_192.min(neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS.saturating_sub(1) / streams.max(1))
}

/// The one-line startup marker required whenever measurement mode relaxes
/// a command's ceilings (stdout contract line, non-JSON).
pub(super) fn emit_marker() {
    if enabled() {
        use std::io::Write as _;
        println!("measurement_mode=true relaxed_limits");
        let _ = std::io::stdout().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exchange-count ceiling must fit every derived runtime limit at
    /// the maximum payload, so any admissible (bytes, count) pair passes
    /// SessionRuntime::new without a per-command derived check.
    #[test]
    fn exchange_count_ceiling_fits_all_derived_runtime_limits() {
        assert!(
            EXCHANGE_COUNT_MAX + 2 <= neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS as u64,
            "count+2 must fit HARD_MAX_RUNTIME_QUEUE_RECORDS"
        );
        assert!(
            1200 * (EXCHANGE_COUNT_MAX as usize + 1) <= neko_session::HARD_MAX_RUNTIME_QUEUE_BYTES,
            "bytes*(count+1) must fit HARD_MAX_RUNTIME_QUEUE_BYTES at the 1200B payload"
        );
        assert!(
            1200 * EXCHANGE_COUNT_MAX as usize <= neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES,
            "bytes*count must fit HARD_MAX_RUNTIME_TOTAL_BYTES at the 1200B payload"
        );
    }

    /// The periodic ceilings must match the periodic runtime's derivation:
    /// queue records `count + 2`, lifetime total `bytes * count * 2`.
    #[test]
    fn periodic_ceilings_match_the_runtime_derivation() {
        assert!(PERIODIC_COUNT_MAX + 2 <= neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS as u64);
        assert!(
            PERIODIC_TOTAL_BYTES_MAX * 2 <= neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES as u64,
            "runtime max_total_bytes = bytes*count*2 must fit HARD_MAX_RUNTIME_TOTAL_BYTES"
        );
    }

    /// The multistream shape ceilings must match the multistream runtime's
    /// derivation: declared `max_queue_records = streams*records + 1`,
    /// per-record frames inside PROCESS_FRAME_MAX, lifetime total inside
    /// HARD_MAX_RUNTIME_TOTAL_BYTES.
    #[test]
    fn multistream_ceilings_match_the_runtime_derivation() {
        for streams in [1usize, 2, 8, 16, 64, 255, MULTISTREAM_STREAMS_MAX] {
            let records = multistream_records_max(streams);
            assert!(
                streams * records < neko_session::HARD_MAX_RUNTIME_QUEUE_RECORDS,
                "streams={streams} records={records} must fit the queue-record declaration"
            );
        }
        assert_eq!(
            multistream_records_max(MULTISTREAM_STREAMS_MAX),
            255,
            "256 streams pair with 255 records (65_535 total + 1)"
        );
    }
}
