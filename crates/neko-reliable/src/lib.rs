//! Deterministic, socket-free UDP packet recovery candidate.
//! Packet ACK evidence in this crate is carrier-local and never Session delivery evidence.
use std::collections::{BTreeMap, BTreeSet};

pub const PACKET_THRESHOLD: u64 = 3;
pub const LOSS_TIME_NUMERATOR: u64 = 9;
pub const LOSS_TIME_DENOMINATOR: u64 = 8;
pub const DEFAULT_MSS: u64 = 1200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Exhausted,
    InvalidLimit,
    InvalidRange,
    TooManyRanges,
    UnknownPacket,
    Arithmetic,
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
        for r in ranges {
            if r.start > r.end {
                return Err(Error::InvalidRange);
            };
            for n in r.start..=r.end {
                out.insert(n)?;
                if n == u64::MAX {
                    break;
                }
            }
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
#[derive(Debug, Default)]
pub struct Recovery {
    sent: BTreeMap<u64, SentPacket>,
    outstanding_frames: BTreeSet<FrameId>,
    pub rtt: RttEstimator,
}
impl Recovery {
    pub fn on_sent(&mut self, p: SentPacket) -> Result<(), Error> {
        if self.sent.contains_key(&p.number) {
            return Err(Error::InvalidRange);
        };
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
        Ok(out)
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
            cwnd: 10 * mss,
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
}
