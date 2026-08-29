//! Deterministic, socket-free UDP packet recovery candidate.
//! Packet ACK evidence in this crate is carrier-local and never Session delivery evidence.
use std::collections::{BTreeMap, BTreeSet};

pub const PACKET_THRESHOLD: u64 = 3;
pub const LOSS_TIME_NUMERATOR: u64 = 9;
pub const LOSS_TIME_DENOMINATOR: u64 = 8;
pub const DEFAULT_MSS: u64 = 1200;
pub const DEFAULT_MAX_SENT_PACKETS: usize = 4096;
pub const DEFAULT_MAX_FRAMES_PER_PACKET: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Exhausted,
    InvalidLimit,
    InvalidRange,
    TooManyRanges,
    UnknownPacket,
    Arithmetic,
    Capacity,
    EmptyPacket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketNumbers {
    next: u64,
    exhausted: bool,
}
impl PacketNumbers {
    pub const fn new(first: u64) -> Self {
        Self {
            next: first,
            exhausted: false,
        }
    }
    pub fn allocate(&mut self) -> Result<u64, Error> {
        if self.exhausted {
            return Err(Error::Exhausted);
        }
        let n = self.next;
        match n.checked_add(1) {
            Some(v) => self.next = v,
            None => self.exhausted = true,
        };
        Ok(n)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckRange {
    pub start: u64,
    pub end: u64,
}
impl AckRange {
    pub fn new(start: u64, end: u64) -> Result<Self, Error> {
        (start <= end)
            .then_some(Self { start, end })
            .ok_or(Error::InvalidRange)
    }
    pub fn contains(self, n: u64) -> bool {
        n >= self.start && n <= self.end
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckRanges {
    max_ranges: usize,
    ranges: Vec<AckRange>,
}
impl AckRanges {
    pub fn new(max_ranges: usize) -> Result<Self, Error> {
        if max_ranges == 0 {
            return Err(Error::InvalidLimit);
        }
        Ok(Self {
            max_ranges,
            ranges: Vec::new(),
        })
    }
    pub fn ranges(&self) -> &[AckRange] {
        &self.ranges
    }
    pub fn largest(&self) -> Option<u64> {
        self.ranges.last().map(|r| r.end)
    }
    pub fn contains(&self, n: u64) -> bool {
        self.ranges.iter().any(|r| r.contains(n))
    }
    pub fn insert(&mut self, n: u64) -> Result<(), Error> {
        if self.contains(n) {
            return Ok(());
        }
        let mut candidate = self.ranges.clone();
        candidate.push(AckRange { start: n, end: n });
        candidate.sort_by_key(|r| r.start);
        let mut merged: Vec<AckRange> = Vec::with_capacity(candidate.len());
        for r in candidate {
            if let Some(last) = merged.last_mut()
                && r.start <= last.end.saturating_add(1)
            {
                last.end = last.end.max(r.end);
                continue;
            }
            merged.push(r)
        }
        if merged.len() > self.max_ranges {
            return Err(Error::TooManyRanges);
        }
        self.ranges = merged;
        Ok(())
    }
    pub fn from_ranges(max_ranges: usize, ranges: &[AckRange]) -> Result<Self, Error> {
        let mut out = Self::new(max_ranges)?;
        let mut candidate = ranges.to_vec();
        if candidate.iter().any(|r| r.start > r.end) {
            return Err(Error::InvalidRange);
        }
        candidate.sort_by_key(|r| r.start);
        for range in candidate {
            if let Some(last) = out.ranges.last_mut()
                && range.start <= last.end.saturating_add(1)
            {
                last.end = last.end.max(range.end);
                continue;
            }
            if out.ranges.len() == out.max_ranges {
                return Err(Error::TooManyRanges);
            }
            out.ranges.push(range);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RttEstimator {
    pub latest_us: u64,
    pub min_us: u64,
    pub smoothed_us: u64,
    pub variance_us: u64,
    initialized: bool,
}
impl RttEstimator {
    pub fn update(&mut self, sample_us: u64, ack_delay_us: u64) {
        if sample_us == 0 {
            return;
        }
        self.latest_us = sample_us;
        self.min_us = if self.initialized {
            self.min_us.min(sample_us)
        } else {
            sample_us
        };
        let adjusted = if sample_us.saturating_sub(self.min_us) >= ack_delay_us {
            sample_us - ack_delay_us
        } else {
            sample_us
        };
        if !self.initialized {
            self.smoothed_us = adjusted;
            self.variance_us = adjusted / 2;
            self.initialized = true;
            return;
        }
        let delta = self.smoothed_us.abs_diff(adjusted);
        self.variance_us = (3 * self.variance_us + delta) / 4;
        self.smoothed_us = (7 * self.smoothed_us + adjusted) / 8;
    }
    pub fn loss_delay_us(&self) -> u64 {
        let base = self.latest_us.max(self.smoothed_us);
        base.saturating_mul(LOSS_TIME_NUMERATOR)
            .div_ceil(LOSS_TIME_DENOMINATOR)
    }
    pub fn pto_us(&self, granularity_us: u64, max_ack_delay_us: u64, pto_count: u32) -> u64 {
        let base = if self.initialized {
            self.smoothed_us
                .saturating_add((4 * self.variance_us).max(granularity_us))
                .saturating_add(max_ack_delay_us)
        } else {
            1_000_000
        };
        base.saturating_mul(1u64.checked_shl(pto_count.min(63)).unwrap_or(u64::MAX))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameId(pub u64);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentPacket {
    pub number: u64,
    pub sent_at_us: u64,
    pub bytes: u64,
    pub ack_eliciting: bool,
    pub frames: Vec<FrameId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryResult {
    pub acked_packets: Vec<u64>,
    pub lost_packets: Vec<u64>,
    pub retransmit_frames: Vec<FrameId>,
    pub acked_bytes: u64,
    pub lost_bytes: u64,
}
#[derive(Debug)]
pub struct Recovery {
    sent: BTreeMap<u64, SentPacket>,
    outstanding_frames: BTreeSet<FrameId>,
    max_sent_packets: usize,
    max_frames_per_packet: usize,
    pub rtt: RttEstimator,
    pub pto_count: u32,
}
impl Default for Recovery {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_SENT_PACKETS, DEFAULT_MAX_FRAMES_PER_PACKET)
            .expect("valid recovery defaults")
    }
}
impl Recovery {
    pub fn new(max_sent_packets: usize, max_frames_per_packet: usize) -> Result<Self, Error> {
        if max_sent_packets == 0 || max_frames_per_packet == 0 {
            return Err(Error::InvalidLimit);
        }
        Ok(Self {
            sent: BTreeMap::new(),
            outstanding_frames: BTreeSet::new(),
            max_sent_packets,
            max_frames_per_packet,
            rtt: RttEstimator::default(),
            pto_count: 0,
        })
    }
    pub fn on_sent(&mut self, p: SentPacket) -> Result<(), Error> {
        if self.sent.contains_key(&p.number) {
            return Err(Error::InvalidRange);
        }
        if self.sent.len() >= self.max_sent_packets || p.frames.len() > self.max_frames_per_packet {
            return Err(Error::Capacity);
        }
        if p.bytes == 0 || (p.ack_eliciting && p.frames.is_empty()) {
            return Err(Error::EmptyPacket);
        }
        for f in &p.frames {
            self.outstanding_frames.insert(*f);
        }
        self.sent.insert(p.number, p);
        Ok(())
    }
    pub fn in_flight(&self) -> usize {
        self.sent.len()
    }
    pub fn on_ack(
        &mut self,
        ack: &AckRanges,
        now_us: u64,
        ack_delay_us: u64,
    ) -> Result<RecoveryResult, Error> {
        let largest = ack.largest().ok_or(Error::InvalidRange)?;
        let mut out = RecoveryResult::default();
        if let Some(p) = self.sent.get(&largest) {
            self.rtt.update(
                now_us.checked_sub(p.sent_at_us).ok_or(Error::Arithmetic)?,
                ack_delay_us,
            )
        }
        let acked: Vec<u64> = self
            .sent
            .keys()
            .copied()
            .filter(|n| ack.contains(*n))
            .collect();
        for n in acked {
            let p = self.sent.remove(&n).ok_or(Error::UnknownPacket)?;
            out.acked_bytes = out.acked_bytes.saturating_add(p.bytes);
            out.acked_packets.push(n);
            for f in p.frames {
                self.outstanding_frames.remove(&f);
            }
        }
        let delay = self.rtt.loss_delay_us();
        let lost: Vec<u64> = self
            .sent
            .iter()
            .filter_map(|(&n, p)| {
                ((largest.saturating_sub(n) >= PACKET_THRESHOLD)
                    || (delay > 0 && now_us.saturating_sub(p.sent_at_us) >= delay))
                    .then_some(n)
            })
            .collect();
        let mut frames = BTreeSet::new();
        for n in lost {
            let p = self.sent.remove(&n).ok_or(Error::UnknownPacket)?;
            out.lost_bytes = out.lost_bytes.saturating_add(p.bytes);
            out.lost_packets.push(n);
            for f in p.frames {
                if self.outstanding_frames.remove(&f) {
                    frames.insert(f);
                }
            }
        }
        out.retransmit_frames = frames.into_iter().collect();
        if !out.acked_packets.is_empty() {
            self.pto_count = 0;
        }
        Ok(out)
    }
    /// PTO schedules at most `max_probe_frames` oldest outstanding frames. It
    /// does not declare packets lost and does not create Session delivery evidence.
    pub fn on_pto(&mut self, max_probe_frames: usize) -> Result<Vec<FrameId>, Error> {
        if max_probe_frames == 0 {
            return Err(Error::InvalidLimit);
        }
        self.pto_count = self.pto_count.saturating_add(1);
        Ok(self
            .outstanding_frames
            .iter()
            .copied()
            .take(max_probe_frames)
            .collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reno {
    pub cwnd: u64,
    pub ssthresh: u64,
    pub bytes_in_flight: u64,
    mss: u64,
}
impl Reno {
    pub fn new(mss: u64) -> Result<Self, Error> {
        if mss == 0 {
            return Err(Error::InvalidLimit);
        }
        Ok(Self {
            cwnd: 10u64.checked_mul(mss).ok_or(Error::Arithmetic)?,
            ssthresh: u64::MAX,
            bytes_in_flight: 0,
            mss,
        })
    }
    pub fn can_send(&self, bytes: u64) -> bool {
        self.bytes_in_flight.saturating_add(bytes) <= self.cwnd
    }
    pub fn sent(&mut self, b: u64) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_add(b)
    }
    pub fn acked(&mut self, b: u64) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(b);
        self.cwnd = if self.cwnd < self.ssthresh {
            self.cwnd.saturating_add(b)
        } else {
            self.cwnd
                .saturating_add(self.mss.saturating_mul(b) / self.cwnd.max(1))
        }
    }
    pub fn lost(&mut self, b: u64) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(b);
        self.ssthresh = (self.cwnd / 2).max(2 * self.mss);
        self.cwnd = self.ssthresh
    }
    pub fn pacing_interval_us(&self, rtt_us: u64, bytes: u64) -> u64 {
        rtt_us.saturating_mul(bytes).div_ceil(self.cwnd.max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationResult {
    pub sent: u64,
    pub delivered: u64,
    pub retransmitted: u64,
    pub rounds: u64,
}

/// Monotonic socket-free clock used by deterministic acceptance tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VirtualClock {
    now_us: u64,
}
impl VirtualClock {
    pub const fn new() -> Self {
        Self { now_us: 0 }
    }
    pub const fn now_us(self) -> u64 {
        self.now_us
    }
    pub fn advance(&mut self, delta_us: u64) -> Result<u64, Error> {
        self.now_us = self.now_us.checked_add(delta_us).ok_or(Error::Arithmetic)?;
        Ok(self.now_us)
    }
}

/// Deterministic loss, burst, reorder and blackhole faults for local recovery tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaultProfile {
    pub drop_every: u64,
    pub burst_start: u64,
    pub burst_len: u64,
    pub reorder: bool,
    pub blackhole: bool,
}
pub fn simulate_fault_profile(
    total_frames: u64,
    profile: FaultProfile,
) -> Result<SimulationResult, Error> {
    if total_frames == 0 || total_frames > 100_000 {
        return Err(Error::InvalidLimit);
    }
    let mut clock = VirtualClock::new();
    let mut pending: Vec<u64> = (0..total_frames).collect();
    let mut delivered = BTreeSet::new();
    let mut sent = 0;
    let mut retransmitted = 0;
    let mut rounds = 0;
    while !pending.is_empty() {
        rounds = rounds.checked_add(1).ok_or(Error::Arithmetic)?;
        if rounds > total_frames.saturating_add(2) {
            return Err(Error::Capacity);
        }
        let current = std::mem::take(&mut pending);
        let mut arrivals = Vec::new();
        for frame in current {
            sent = sent.checked_add(1).ok_or(Error::Arithmetic)?;
            let first = !delivered.contains(&frame);
            let burst = profile.burst_len > 0
                && frame >= profile.burst_start
                && frame < profile.burst_start.saturating_add(profile.burst_len);
            if profile.blackhole
                || (first
                    && (burst
                        || (profile.drop_every != 0 && (frame + 1) % profile.drop_every == 0)))
            {
                if !profile.blackhole {
                    pending.push(frame);
                }
                continue;
            }
            if !first {
                retransmitted = retransmitted.checked_add(1).ok_or(Error::Arithmetic)?;
            }
            arrivals.push(frame);
        }
        if profile.reorder {
            arrivals.reverse();
        }
        for frame in arrivals {
            delivered.insert(frame);
        }
        clock.advance(1)?;
    }
    Ok(SimulationResult {
        sent,
        delivered: delivered.len() as u64,
        retransmitted,
        rounds,
    })
}

/// Deterministic frame-delivery simulation used for bounded loss/reorder tests.
/// `drop_every=0` means no loss; otherwise every Nth first transmission is lost.
pub fn simulate_delivery(
    total_frames: u64,
    drop_every: u64,
    reorder: bool,
) -> Result<SimulationResult, Error> {
    if total_frames == 0 || total_frames > 100_000 {
        return Err(Error::InvalidLimit);
    }
    let mut pending: Vec<u64> = (0..total_frames).collect();
    let mut delivered = BTreeSet::new();
    let mut sent = 0u64;
    let mut retransmitted = 0u64;
    let mut rounds = 0u64;
    while !pending.is_empty() {
        rounds = rounds.checked_add(1).ok_or(Error::Arithmetic)?;
        if rounds > total_frames.saturating_add(2) {
            return Err(Error::Capacity);
        }
        let current = std::mem::take(&mut pending);
        let mut arrivals = Vec::new();
        for frame in current {
            sent = sent.checked_add(1).ok_or(Error::Arithmetic)?;
            let first_attempt = !delivered.contains(&frame) && rounds == 1;
            if first_attempt && drop_every != 0 && (frame + 1) % drop_every == 0 {
                pending.push(frame);
                continue;
            }
            if rounds > 1 {
                retransmitted = retransmitted.checked_add(1).ok_or(Error::Arithmetic)?;
            }
            arrivals.push(frame);
        }
        if reorder {
            arrivals.reverse();
        }
        for frame in arrivals {
            delivered.insert(frame);
        }
    }
    Ok(SimulationResult {
        sent,
        delivered: delivered.len() as u64,
        retransmitted,
        rounds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_numbers_fail_closed() {
        let mut p = PacketNumbers::new(u64::MAX);
        assert_eq!(p.allocate(), Ok(u64::MAX));
        assert_eq!(p.allocate(), Err(Error::Exhausted));
    }
    #[test]
    fn ack_ranges_merge_and_bound_atomically() {
        let mut a = AckRanges::new(2).unwrap();
        for n in [5, 7, 6, 1] {
            a.insert(n).unwrap()
        }
        assert_eq!(
            a.ranges(),
            &[AckRange { start: 1, end: 1 }, AckRange { start: 5, end: 7 }]
        );
        assert_eq!(a.insert(3), Err(Error::TooManyRanges));
        assert!(!a.contains(3));
    }
    #[test]
    fn ack_range_boundaries_are_canonical() {
        let a = AckRanges::from_ranges(
            2,
            &[AckRange::new(0, 1).unwrap(), AckRange::new(2, 3).unwrap()],
        )
        .unwrap();
        assert_eq!(a.ranges(), &[AckRange { start: 0, end: 3 }]);
        assert_eq!(AckRange::new(2, 1), Err(Error::InvalidRange));
    }
    #[test]
    fn rtt_pto_are_deterministic() {
        let mut r = RttEstimator::default();
        r.update(100_000, 0);
        assert_eq!((r.smoothed_us, r.variance_us), (100_000, 50_000));
        r.update(120_000, 10_000);
        assert_eq!((r.smoothed_us, r.variance_us), (101_250, 40_000));
        assert_eq!(r.pto_us(1_000, 25_000, 0), 286_250);
        assert_eq!(r.pto_us(1_000, 25_000, 2), 1_145_000);
    }
    fn packet(n: u64, t: u64, f: u64) -> SentPacket {
        SentPacket {
            number: n,
            sent_at_us: t,
            bytes: 1200,
            ack_eliciting: true,
            frames: vec![FrameId(f)],
        }
    }
    #[test]
    fn packet_threshold_loss_retransmits_frames_not_packets() {
        let mut r = Recovery::default();
        for n in 0..=3 {
            r.on_sent(packet(n, 0, n)).unwrap()
        }
        let mut a = AckRanges::new(2).unwrap();
        a.insert(3).unwrap();
        let x = r.on_ack(&a, 10_000, 0).unwrap();
        assert_eq!(x.acked_packets, vec![3]);
        assert_eq!(x.lost_packets, vec![0]);
        assert_eq!(x.retransmit_frames, vec![FrameId(0)]);
        assert_eq!(r.in_flight(), 2);
    }
    #[test]
    fn reorder_inside_threshold_is_not_loss() {
        let mut r = Recovery::default();
        for n in 10..=12 {
            r.on_sent(packet(n, 1000, n)).unwrap()
        }
        let mut a = AckRanges::new(2).unwrap();
        a.insert(12).unwrap();
        let x = r.on_ack(&a, 2000, 0).unwrap();
        assert!(x.lost_packets.is_empty());
        assert_eq!(r.in_flight(), 2);
    }
    #[test]
    fn time_threshold_and_reno_pacing() {
        let mut r = Recovery::default();
        r.rtt.update(8_000, 0);
        r.on_sent(packet(1, 0, 7)).unwrap();
        let mut a = AckRanges::new(1).unwrap();
        a.insert(9).unwrap();
        let x = r.on_ack(&a, 9_000, 0).unwrap();
        assert_eq!(x.lost_packets, vec![1]);
        let mut c = Reno::new(1200).unwrap();
        c.sent(1200);
        c.acked(1200);
        assert!(c.cwnd > 12_000);
        assert!(c.pacing_interval_us(100_000, 1200) > 0);
        c.sent(2400);
        c.lost(2400);
        assert_eq!(c.cwnd, c.ssthresh);
    }
    #[test]
    fn huge_ack_range_is_constant_work_and_canonical() {
        let a = AckRanges::from_ranges(1, &[AckRange::new(0, u64::MAX).unwrap()]).unwrap();
        assert_eq!(
            a.ranges(),
            &[AckRange {
                start: 0,
                end: u64::MAX
            }]
        );
    }
    #[test]
    fn recovery_capacity_and_pto_are_bounded() {
        let mut r = Recovery::new(1, 1).unwrap();
        r.on_sent(packet(1, 0, 7)).unwrap();
        assert_eq!(r.on_sent(packet(2, 0, 8)), Err(Error::Capacity));
        assert_eq!(r.on_pto(1), Ok(vec![FrameId(7)]));
        assert_eq!(r.in_flight(), 1);
        assert_eq!(r.pto_count, 1);
    }
    #[test]
    #[test]
    fn fault_profiles_cover_burst_reorder_blackhole_and_clock() {
        let x = simulate_fault_profile(
            100,
            FaultProfile {
                burst_start: 20,
                burst_len: 10,
                reorder: true,
                ..FaultProfile::default()
            },
        )
        .unwrap();
        assert_eq!((x.delivered, x.retransmitted, x.rounds), (100, 10, 2));
        let x = simulate_fault_profile(
            8,
            FaultProfile {
                blackhole: true,
                ..FaultProfile::default()
            },
        )
        .unwrap();
        assert_eq!((x.delivered, x.sent), (0, 8));
        let mut clock = VirtualClock::new();
        assert_eq!(clock.advance(25), Ok(25));
        assert_eq!(clock.advance(u64::MAX), Err(Error::Arithmetic));
        assert_eq!(clock.now_us(), 25);
    }

    fn deterministic_loss_and_reorder_preserve_all_frames() {
        for drop_every in [0, 100, 20, 10] {
            for reorder in [false, true] {
                let x = simulate_delivery(1000, drop_every, reorder).unwrap();
                assert_eq!(x.delivered, 1000);
                assert!(x.rounds <= 2);
                assert_eq!(
                    x.retransmitted,
                    1000u64.checked_div(drop_every).unwrap_or(0)
                );
            }
        }
    }
}

/// Socket-free Packetization Layer PMTU Discovery candidate. Only an explicit,
/// authenticated probe acknowledgement may raise the confirmed size; ICMP and
/// ordinary packet loss are intentionally outside this state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlpmtudConfig {
    pub base_mtu: u16,
    pub max_mtu: u16,
    pub attempts_per_size: u8,
    pub max_probes: u16,
    pub blackhole_threshold: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    pub id: u64,
    pub path_generation: u64,
    pub size: u16,
    attempts_left: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlpmtudError {
    InvalidConfig,
    ProbeOutstanding,
    NoProbeNeeded,
    ProbeLimit,
    StaleAck,
    WrongSize,
    NoOutstandingProbe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeTimeout {
    Retry(Probe),
    ReducedUpperBound,
    Converged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plpmtud {
    config: PlpmtudConfig,
    generation: u64,
    confirmed: u16,
    upper: u16,
    next_probe_id: u64,
    probes_started: u16,
    outstanding: Option<Probe>,
    base_loss_run: u8,
}
impl Plpmtud {
    pub fn new(config: PlpmtudConfig, generation: u64) -> Result<Self, PlpmtudError> {
        if config.base_mtu < 576
            || config.max_mtu < config.base_mtu
            || config.attempts_per_size == 0
            || config.max_probes == 0
            || config.blackhole_threshold == 0
        {
            return Err(PlpmtudError::InvalidConfig);
        }
        Ok(Self {
            config,
            generation,
            confirmed: config.base_mtu,
            upper: config.max_mtu,
            next_probe_id: 0,
            probes_started: 0,
            outstanding: None,
            base_loss_run: 0,
        })
    }
    pub const fn confirmed_mtu(&self) -> u16 {
        self.confirmed
    }
    pub const fn upper_bound(&self) -> u16 {
        self.upper
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn outstanding(&self) -> Option<Probe> {
        self.outstanding
    }
    pub fn converged(&self) -> bool {
        self.confirmed >= self.upper
    }
    pub fn start_probe(&mut self) -> Result<Probe, PlpmtudError> {
        if self.outstanding.is_some() {
            return Err(PlpmtudError::ProbeOutstanding);
        }
        if self.converged() {
            return Err(PlpmtudError::NoProbeNeeded);
        }
        if self.probes_started >= self.config.max_probes {
            return Err(PlpmtudError::ProbeLimit);
        }
        let gap = self.upper - self.confirmed;
        let size = self.confirmed + gap.div_ceil(2);
        let probe = Probe {
            id: self.next_probe_id,
            path_generation: self.generation,
            size,
            attempts_left: self.config.attempts_per_size,
        };
        self.next_probe_id = self
            .next_probe_id
            .checked_add(1)
            .ok_or(PlpmtudError::ProbeLimit)?;
        self.probes_started += 1;
        self.outstanding = Some(probe);
        Ok(probe)
    }
    /// ACK must be bound to the authenticated path generation, probe id and size.
    pub fn acknowledge(
        &mut self,
        id: u64,
        generation: u64,
        size: u16,
    ) -> Result<bool, PlpmtudError> {
        let probe = self.outstanding.ok_or(PlpmtudError::NoOutstandingProbe)?;
        if generation != self.generation || generation != probe.path_generation || id != probe.id {
            return Err(PlpmtudError::StaleAck);
        }
        if size != probe.size {
            return Err(PlpmtudError::WrongSize);
        }
        self.confirmed = self.confirmed.max(size);
        self.outstanding = None;
        self.base_loss_run = 0;
        Ok(self.converged())
    }
    /// Timeout is probe-local loss evidence, never direct path-failure evidence.
    pub fn timeout(&mut self) -> Result<ProbeTimeout, PlpmtudError> {
        let mut probe = self.outstanding.ok_or(PlpmtudError::NoOutstandingProbe)?;
        if probe.attempts_left > 1 {
            probe.attempts_left -= 1;
            self.outstanding = Some(probe);
            return Ok(ProbeTimeout::Retry(probe));
        }
        self.outstanding = None;
        self.upper = probe.size.saturating_sub(1).max(self.confirmed);
        Ok(if self.converged() {
            ProbeTimeout::Converged
        } else {
            ProbeTimeout::ReducedUpperBound
        })
    }
    /// Repeated loss at or below the confirmed size invokes a conservative
    /// blackhole fallback. It does not declare the path failed.
    pub fn observe_confirmed_size_loss(&mut self, packet_size: u16) -> bool {
        if packet_size > self.confirmed {
            return false;
        }
        self.base_loss_run = self.base_loss_run.saturating_add(1);
        if self.base_loss_run < self.config.blackhole_threshold {
            return false;
        }
        self.confirmed = self.config.base_mtu;
        self.upper = self.upper.max(self.confirmed);
        self.outstanding = None;
        self.base_loss_run = 0;
        true
    }
    pub fn observe_progress(&mut self) {
        self.base_loss_run = 0;
    }
    /// A new path generation discards stale probe evidence and restarts from base.
    pub fn reset_generation(&mut self, generation: u64) {
        self.generation = generation;
        self.confirmed = self.config.base_mtu;
        self.upper = self.config.max_mtu;
        self.outstanding = None;
        self.probes_started = 0;
        self.base_loss_run = 0;
    }
}

#[cfg(test)]
mod plpmtud_tests {
    use super::*;
    fn config() -> PlpmtudConfig {
        PlpmtudConfig {
            base_mtu: 1200,
            max_mtu: 1500,
            attempts_per_size: 2,
            max_probes: 32,
            blackhole_threshold: 3,
        }
    }
    fn discover(actual: u16) -> Plpmtud {
        let mut p = Plpmtud::new(config(), 7).unwrap();
        while !p.converged() {
            let q = p.start_probe().unwrap();
            if q.size <= actual {
                p.acknowledge(q.id, q.path_generation, q.size).unwrap();
            } else {
                p.timeout().unwrap();
                p.timeout().unwrap();
            }
        }
        p
    }
    #[test]
    fn converges_across_path_mtus() {
        for mtu in [1200, 1201, 1280, 1499, 1500] {
            assert_eq!(discover(mtu).confirmed_mtu(), mtu);
        }
    }
    #[test]
    fn timeout_retries_then_reduces_without_path_failure() {
        let mut p = Plpmtud::new(config(), 1).unwrap();
        let q = p.start_probe().unwrap();
        assert_eq!(
            p.timeout(),
            Ok(ProbeTimeout::Retry(Probe {
                attempts_left: 1,
                ..q
            }))
        );
        assert_eq!(p.timeout(), Ok(ProbeTimeout::ReducedUpperBound));
        assert_eq!(p.upper_bound(), q.size - 1);
    }
    #[test]
    fn stale_reordered_duplicate_and_wrong_ack_are_rejected() {
        let mut p = Plpmtud::new(config(), 5).unwrap();
        let q = p.start_probe().unwrap();
        assert_eq!(p.acknowledge(q.id, 4, q.size), Err(PlpmtudError::StaleAck));
        assert_eq!(
            p.acknowledge(q.id, 5, q.size - 1),
            Err(PlpmtudError::WrongSize)
        );
        p.acknowledge(q.id, 5, q.size).unwrap();
        assert_eq!(
            p.acknowledge(q.id, 5, q.size),
            Err(PlpmtudError::NoOutstandingProbe)
        );
        let old = p.start_probe().unwrap();
        p.reset_generation(6);
        assert_eq!(
            p.acknowledge(old.id, 5, old.size),
            Err(PlpmtudError::NoOutstandingProbe)
        );
    }
    #[test]
    fn only_one_probe_and_resource_limit() {
        let mut c = config();
        c.max_probes = 1;
        let mut p = Plpmtud::new(c, 1).unwrap();
        p.start_probe().unwrap();
        assert_eq!(p.start_probe(), Err(PlpmtudError::ProbeOutstanding));
        p.timeout().unwrap();
        p.timeout().unwrap();
        assert_eq!(p.start_probe(), Err(PlpmtudError::ProbeLimit));
    }
    #[test]
    fn blackhole_fallback_is_bounded_and_progress_resets_counter() {
        let mut p = discover(1500);
        assert_eq!(p.confirmed_mtu(), 1500);
        assert!(!p.observe_confirmed_size_loss(1400));
        p.observe_progress();
        assert!(!p.observe_confirmed_size_loss(1400));
        assert!(!p.observe_confirmed_size_loss(1400));
        assert!(p.observe_confirmed_size_loss(1400));
        assert_eq!(p.confirmed_mtu(), 1200);
    }
    #[test]
    fn probe_arithmetic_and_rejection_are_bounded_at_u16_edges() {
        let c = PlpmtudConfig {
            base_mtu: 65_000,
            max_mtu: 65_535,
            attempts_per_size: 1,
            max_probes: 1,
            blackhole_threshold: 1,
        };
        let mut p = Plpmtud::new(c, u64::MAX).unwrap();
        let q = p.start_probe().unwrap();
        assert!(q.size > c.base_mtu && q.size <= c.max_mtu);
        assert_eq!(
            p.acknowledge(q.id, u64::MAX - 1, q.size),
            Err(PlpmtudError::StaleAck)
        );
        assert_eq!(p.outstanding(), Some(q));
        assert_eq!(
            p.acknowledge(q.id, u64::MAX, q.size - 1),
            Err(PlpmtudError::WrongSize)
        );
        assert_eq!(p.outstanding(), Some(q));
        assert_eq!(p.timeout(), Ok(ProbeTimeout::ReducedUpperBound));
        assert!(p.upper_bound() >= p.confirmed_mtu());
    }

    #[test]
    fn invalid_configurations_fail_closed() {
        for c in [
            PlpmtudConfig {
                base_mtu: 575,
                ..config()
            },
            PlpmtudConfig {
                max_mtu: 1199,
                ..config()
            },
            PlpmtudConfig {
                attempts_per_size: 0,
                ..config()
            },
            PlpmtudConfig {
                max_probes: 0,
                ..config()
            },
            PlpmtudConfig {
                blackhole_threshold: 0,
                ..config()
            },
        ] {
            assert_eq!(Plpmtud::new(c, 0), Err(PlpmtudError::InvalidConfig));
        }
    }
}

/// Bounded systematic XOR block-FEC candidate. FEC is an optional carrier
/// optimization: it does not acknowledge delivery, replace retransmission, or
/// alter congestion-control evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FecConfig {
    pub block_size: u8,
    pub symbol_size: u16,
    pub max_blocks: u16,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FecBlock {
    pub block_id: u64,
    pub data: Vec<Vec<u8>>,
    pub parity: Vec<u8>,
    pub received: Vec<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FecError {
    InvalidConfig,
    TooManyBlocks,
    WrongSymbolSize,
    IndexOutOfRange,
    Duplicate,
    Unrecoverable,
    Empty,
}
impl FecConfig {
    fn validate(self) -> Result<(), FecError> {
        if self.block_size < 2
            || self.block_size > 32
            || self.symbol_size == 0
            || self.max_blocks == 0
        {
            return Err(FecError::InvalidConfig);
        }
        Ok(())
    }
}
impl FecBlock {
    pub fn encode(config: FecConfig, block_id: u64, symbols: &[Vec<u8>]) -> Result<Self, FecError> {
        config.validate()?;
        if block_id > u64::from(config.max_blocks) {
            return Err(FecError::TooManyBlocks);
        }
        if symbols.len() != config.block_size as usize {
            return Err(FecError::InvalidConfig);
        }
        if symbols
            .iter()
            .any(|s| s.len() != config.symbol_size as usize)
        {
            return Err(FecError::WrongSymbolSize);
        }
        let mut parity = vec![0; config.symbol_size as usize];
        for s in symbols {
            for (i, b) in s.iter().enumerate() {
                parity[i] ^= *b
            }
        }
        Ok(Self {
            block_id,
            data: symbols.to_vec(),
            parity,
            received: vec![true; config.block_size as usize],
        })
    }
    pub fn mark_missing(&mut self, index: usize) -> Result<(), FecError> {
        if index >= self.data.len() {
            return Err(FecError::IndexOutOfRange);
        }
        if !self.received[index] {
            return Err(FecError::Duplicate);
        }
        self.received[index] = false;
        Ok(())
    }
    pub fn recover_one(&mut self) -> Result<usize, FecError> {
        let missing: Vec<_> = self
            .received
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (!r).then_some(i))
            .collect();
        if missing.is_empty() {
            return Err(FecError::Empty);
        }
        if missing.len() > 1 {
            return Err(FecError::Unrecoverable);
        }
        let index = missing[0];
        let mut recovered = self.parity.clone();
        for (i, s) in self.data.iter().enumerate() {
            if i != index {
                for (j, b) in s.iter().enumerate() {
                    recovered[j] ^= *b
                }
            }
        }
        self.data[index] = recovered;
        self.received[index] = true;
        Ok(index)
    }
    pub fn complete(&self) -> bool {
        self.received.iter().all(|x| *x)
    }
}

#[cfg(test)]
mod fec_tests {
    use super::*;
    #[test]
    fn xor_recovers_single_loss_and_is_reorder_independent() {
        let c = FecConfig {
            block_size: 4,
            symbol_size: 3,
            max_blocks: 2,
        };
        let mut b = FecBlock::encode(
            c,
            1,
            vec![
                b"abc".to_vec(),
                b"DEF".to_vec(),
                b"123".to_vec(),
                b"xyz".to_vec(),
            ]
            .as_slice(),
        )
        .unwrap();
        b.mark_missing(2).unwrap();
        assert_eq!(b.recover_one(), Ok(2));
        assert_eq!(b.data[2], b"123");
        assert!(b.complete());
    }
    #[test]
    fn multiple_loss_is_not_silently_recovered() {
        let c = FecConfig {
            block_size: 4,
            symbol_size: 2,
            max_blocks: 1,
        };
        let mut b = FecBlock::encode(
            c,
            1,
            vec![
                b"ab".to_vec(),
                b"CD".to_vec(),
                b"12".to_vec(),
                b"xy".to_vec(),
            ]
            .as_slice(),
        )
        .unwrap();
        b.mark_missing(0).unwrap();
        b.mark_missing(3).unwrap();
        assert_eq!(b.recover_one(), Err(FecError::Unrecoverable));
    }
    #[test]
    fn fec_block_identity_is_bounded_and_rejection_is_atomic() {
        let c = FecConfig {
            block_size: 2,
            symbol_size: 2,
            max_blocks: 3,
        };
        assert!(FecBlock::encode(c, 0, &[b"aa".to_vec(), b"bb".to_vec()]).is_ok());
        assert!(FecBlock::encode(c, 3, &[b"aa".to_vec(), b"bb".to_vec()]).is_ok());
        assert_eq!(
            FecBlock::encode(c, 4, &[b"aa".to_vec(), b"bb".to_vec()]),
            Err(FecError::TooManyBlocks)
        );
        assert_eq!(
            FecBlock::encode(c, u64::MAX, &[b"aa".to_vec(), b"bb".to_vec()]),
            Err(FecError::TooManyBlocks)
        );
        // Bounds are checked before symbol cloning/allocation and return no block state.
        assert_eq!(
            FecBlock::encode(c, 4, &[vec![0; 2], vec![0; 2]]),
            Err(FecError::TooManyBlocks)
        );
    }

    #[test]
    fn fec_limits_and_duplicate_boundaries_are_explicit() {
        let c = FecConfig {
            block_size: 1,
            symbol_size: 2,
            max_blocks: 1,
        };
        assert_eq!(
            FecBlock::encode(c, 1, &[b"aa".to_vec()]),
            Err(FecError::InvalidConfig)
        );
        let c = FecConfig {
            block_size: 2,
            symbol_size: 2,
            max_blocks: 1,
        };
        let mut b = FecBlock::encode(c, 1, &[b"aa".to_vec(), b"bb".to_vec()]).unwrap();
        assert_eq!(b.mark_missing(2), Err(FecError::IndexOutOfRange));
        b.mark_missing(0).unwrap();
        assert_eq!(b.mark_missing(0), Err(FecError::Duplicate));
    }
    #[test]
    fn parity_overhead_is_one_symbol_per_block() {
        let c = FecConfig {
            block_size: 8,
            symbol_size: 1200,
            max_blocks: 4,
        };
        let b = FecBlock::encode(c, 1, &vec![vec![0; 1200]; 8]).unwrap();
        assert_eq!(b.parity.len(), 1200);
        assert_eq!(b.data.len(), 8);
    }
}
