//! Impairment decision engine + due-time queue — the deterministic core.
//!
//! `ArrivalEngine` is owned by the receive thread of each direction: given the
//! arrival sequence (proxy-relative times + lengths) and a seed, its decisions
//! are bit-identical across runs. Wall-clock scheduling only affects *when*
//! the pump releases packets, never *what* happens to them.
//!
//! `SendAccountant` is owned by the pump thread: purely observational
//! counters (sent bytes, delivered latency histogram, queue overflow).
//!
//! The two are merged into one `DirectionStats` at exit.

use crate::config::{DirectionConfig, Ge};
use crate::rng::Rng;
use crate::stats::DirectionStats;

/// One impairment outcome for a single arriving packet/chunk.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Drop before enqueue; stats reason tag.
    Drop(&'static str),
    /// Enqueue for send at `due_ms` (proxy-relative); `extra_dup` schedules a
    /// second identical copy at the same due time.
    Enqueue { due_ms: f64, extra_dup: bool },
    /// TCP kill window fired with RST semantics (teardown now).
    KillRst,
}

impl Decision {
    pub fn is_drop(&self) -> bool {
        matches!(self, Decision::Drop(_))
    }
}

/// splitmix64 finalizer — derives independent sub-streams from one --seed.
fn mix(seed: u64, salt: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15).wrapping_add(salt.wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Arrival-side engine: everything that must be reproducible under --seed.
pub struct ArrivalEngine {
    cfg: DirectionConfig,
    rng: Rng,
    ge_bad: bool,
    /// Token bucket in BYTES of credit (may go negative = debt) — arrival-driven.
    tokens: f64,
    /// Proxy-relative time of the last token refill (an arrival event).
    last_refill_ms: f64,
    next_rebind_ms: f64,
    rebinds_done: u64,
    pub stats: DirectionStats,
}

impl ArrivalEngine {
    pub fn new(cfg: DirectionConfig, seed: u64) -> Self {
        let burst = bucket_capacity(&cfg) as f64;
        let next_rebind = if cfg.nat_change_every_ms > 0 {
            cfg.nat_change_every_ms as f64
        } else {
            f64::INFINITY
        };
        ArrivalEngine {
            cfg,
            rng: Rng::new(seed),
            ge_bad: false,
            tokens: burst,
            last_refill_ms: 0.0,
            next_rebind_ms: next_rebind,
            rebinds_done: 0,
            stats: DirectionStats::default(),
        }
    }

    /// Decision pipeline for one arrival. Draw order is fixed:
    /// loss -> GE(transition, h) -> duplicate -> jitter -> reorder.
    /// (blackout/MTU/kill windows draw nothing.)
    pub fn decide(&mut self, now_ms: f64, len: usize) -> Decision {
        self.stats.received_pkts += 1;
        self.stats.received_bytes += len as u64;

        // TCP kill windows: RST tears down; Hold silently drops while open.
        for k in &self.cfg.tcp_kills {
            if k.window.contains(now_ms) {
                return match k.style {
                    crate::config::KillStyle::Rst => Decision::KillRst,
                    crate::config::KillStyle::Hold => {
                        self.stats.dropped_blackout += 1;
                        Decision::Drop("kill-hold")
                    }
                };
            }
        }

        // Scheduled blackout windows (UDP).
        for w in &self.cfg.blackouts {
            if w.contains(now_ms) {
                self.stats.dropped_blackout += 1;
                return Decision::Drop("blackout");
            }
        }

        // Independent random loss.
        if self.cfg.loss > 0.0 && self.rng.next_f64() < self.cfg.loss {
            self.stats.dropped_random += 1;
            return Decision::Drop("random");
        }

        // Gilbert-Elliott burst loss.
        if let Some(Ge { p, r, h }) = self.cfg.ge {
            let flip = if self.ge_bad {
                self.rng.next_f64() < r
            } else {
                self.rng.next_f64() < p
            };
            if flip {
                self.ge_bad = !self.ge_bad;
            }
            if self.ge_bad && self.rng.next_f64() < h {
                self.stats.dropped_ge += 1;
                return Decision::Drop("ge");
            }
        }

        // MTU blackhole: oversize datagrams are dropped whole (PMTU behavior).
        if let Some(mtu) = self.cfg.mtu {
            if len > mtu {
                self.stats.dropped_mtu += 1;
                return Decision::Drop("mtu");
            }
        }

        // Token bucket (arrival-driven): refill to now, then charge.
        if let Some(bps) = self.cfg.rate_bps {
            let elapsed = (now_ms - self.last_refill_ms).max(0.0);
            let cap = bucket_capacity(&self.cfg) as f64;
            self.tokens = (self.tokens + elapsed * bps as f64 / 8000.0).min(cap);
            self.last_refill_ms = now_ms;
            self.tokens -= len as f64;
            if self.tokens < 0.0 {
                self.stats.rate_deferred += 1;
            }
        }

        // Duplication draw.
        let extra_dup = self.cfg.duplicate_p > 0.0 && self.rng.next_f64() < self.cfg.duplicate_p;
        if extra_dup {
            self.stats.duplicates += 1;
        }

        // Base delay + uniform jitter.
        let mut due = now_ms + self.cfg.delay_ms;
        if self.cfg.jitter_ms > 0.0 {
            due += self.rng.next_f64() * self.cfg.jitter_ms;
        }

        // Reorder: extra delay on a probabilistic subset.
        if self.cfg.reorder_p > 0.0 && self.rng.next_f64() < self.cfg.reorder_p {
            self.stats.reorder_hits += 1;
            due += self.cfg.reorder_delay_ms as f64;
        }

        // Shaper debt defers release until the bucket would cover this packet.
        if self.tokens < 0.0 {
            if let Some(bps) = self.cfg.rate_bps {
                let wait_ms = (-self.tokens) * 8000.0 / bps as f64;
                due = due.max(now_ms + wait_ms);
            }
        }

        Decision::Enqueue { due_ms: due, extra_dup }
    }

    /// NAT rebind check (arrival-driven, deterministic given arrival times).
    /// Returns true when the outbound socket must be swapped now.
    pub fn should_rebind_now(&mut self, now_ms: f64) -> bool {
        if now_ms >= self.next_rebind_ms {
            self.rebinds_done += 1;
            self.stats.nat_rebinds = self.rebinds_done;
            self.next_rebind_ms += self.cfg.nat_change_every_ms as f64;
            true
        } else {
            false
        }
    }

    /// TCP reader pre-check: an RST kill window is open right now.
    pub fn rst_now(&self, now_ms: f64) -> bool {
        self.cfg.tcp_kills.iter().any(|k| {
            k.style == crate::config::KillStyle::Rst && k.window.contains(now_ms)
        })
    }

    /// TCP reader pre-check: a stall (hold-kill or UDP blackout window) is
    /// active — TCP must PARK and preserve bytes, not drop them.
    pub fn hold_now(&self, now_ms: f64) -> bool {
        self.cfg.tcp_kills.iter().any(|k| {
            k.style == crate::config::KillStyle::Hold && k.window.contains(now_ms)
        }) || self.cfg.blackouts.iter().any(|w| w.contains(now_ms))
    }

    /// Queue-overflow drop bookkeeping (called by the reader thread when the
    /// shared queue rejects a push).
    pub fn stats_inc_overflow(&mut self) {
        self.stats.dropped_overflow += 1;
    }
}

fn bucket_capacity(cfg: &DirectionConfig) -> u64 {
    if cfg.rate_burst_bytes > 0 {
        cfg.rate_burst_bytes
    } else if let Some(bps) = cfg.rate_bps {
        (bps / 8).max(1500)
    } else {
        0
    }
}

/// Pump-side observational counters (wall-clock dependent, not seeded).
#[derive(Debug, Default)]
pub struct SendAccountant {
    pub sent_pkts: u64,
    pub sent_bytes: u64,
    pub dropped_overflow: u64,
    pub tcp_rst_kills: u64,
    pub nat_rebinds: u64,
    pub latency: LatencyStats,
}

#[derive(Debug, Default, Clone)]
pub struct LatencyStats {
    pub count: u64,
    pub sum_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub hist: [u64; crate::stats::N_BUCKETS],
}

impl SendAccountant {
    pub fn on_send(&mut self, len: usize, latency_ms: f64) {
        self.sent_pkts += 1;
        self.sent_bytes += len as u64;
        let l = &mut self.latency;
        if l.count == 0 || latency_ms < l.min_ms {
            l.min_ms = latency_ms;
        }
        if latency_ms > l.max_ms {
            l.max_ms = latency_ms;
        }
        l.count += 1;
        l.sum_ms += latency_ms;
        l.hist[crate::stats::bucket_index(latency_ms)] += 1;
    }

    /// Fold into a DirectionStats that already holds arrival-side counters.
    pub fn merge_into(&self, st: &mut DirectionStats) {
        st.sent_pkts += self.sent_pkts;
        st.sent_bytes += self.sent_bytes;
        st.dropped_overflow += self.dropped_overflow;
        st.tcp_rst_kills += self.tcp_rst_kills;
        st.nat_rebinds += self.nat_rebinds;
        st.latency_count += self.latency.count;
        st.latency_sum_ms += self.latency.sum_ms;
        if self.latency.count > 0 {
            if st.latency_count == self.latency.count || st.latency_min_ms == 0.0 {
                st.latency_min_ms = if st.latency_min_ms == 0.0 {
                    self.latency.min_ms
                } else {
                    st.latency_min_ms.min(self.latency.min_ms)
                };
            }
            st.latency_max_ms = st.latency_max_ms.max(self.latency.max_ms);
        }
        for i in 0..crate::stats::N_BUCKETS {
            st.hist[i] += self.latency.hist[i];
        }
    }
}

/// Min-heap element: ordered by (due_ms, seq); seq breaks ties FIFO so output
/// order is fully determined by the decision sequence.
#[derive(Debug)]
pub struct Queued {
    pub due_ms: f64,
    pub seq: u64,
    pub arrival_ms: f64,
    /// Routing tag: UDP client id (both directions); unused by TCP.
    pub route: u64,
    pub copy_id: u32,
    pub data: Vec<u8>,
}

impl Queued {
    fn key(&self) -> (f64, u64) {
        (self.due_ms, self.seq)
    }
}
impl PartialEq for Queued {
    fn eq(&self, o: &Self) -> bool {
        self.key() == o.key()
    }
}
impl Eq for Queued {}
impl PartialOrd for Queued {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Queued {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.key().partial_cmp(&o.key()).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Due-time release queue with a packet capacity cap.
pub struct ImpairQueue {
    heap: std::collections::BinaryHeap<std::cmp::Reverse<Queued>>,
    next_seq: u64,
    cap: usize,
}

impl ImpairQueue {
    pub fn new(cap: usize) -> Self {
        ImpairQueue { heap: std::collections::BinaryHeap::new(), next_seq: 0, cap }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// None = capacity exceeded (caller counts an overflow drop).
    pub fn push(
        &mut self,
        due_ms: f64,
        arrival_ms: f64,
        route: u64,
        copy_id: u32,
        data: Vec<u8>,
    ) -> Option<u64> {
        if self.heap.len() >= self.cap {
            return None;
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        self.heap.push(std::cmp::Reverse(Queued { due_ms, seq, arrival_ms, route, copy_id, data }));
        Some(seq)
    }

    /// Pop every item due at or before `now_ms`, in (due, seq) order.
    pub fn pop_due(&mut self, now_ms: f64) -> Vec<Queued> {
        let mut out = Vec::new();
        while let Some(std::cmp::Reverse(item)) = self.heap.peek() {
            if item.due_ms <= now_ms {
                let std::cmp::Reverse(item) = self.heap.pop().unwrap();
                out.push(item);
            } else {
                break;
            }
        }
        out
    }

    pub fn next_due(&self) -> Option<f64> {
        self.heap.peek().map(|std::cmp::Reverse(q)| q.due_ms)
    }

    /// Drain everything ignoring due times (TCP teardown).
    pub fn drain_all(&mut self) -> Vec<Queued> {
        let mut out = Vec::new();
        while let Some(std::cmp::Reverse(item)) = self.heap.pop() {
            out.push(item);
        }
        out
    }
}

/// Deterministic per-direction seed derivation.
pub fn direction_seed(seed: u64, salt: u64) -> u64 {
    mix(seed, salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DirectionConfig, Ge, TcpKill, Window, KillStyle};

    fn cfg(loss: f64) -> DirectionConfig {
        DirectionConfig { loss, ..Default::default() }
    }

    #[test]
    fn ten_percent_loss_lands_in_nine_to_eleven_over_10k() {
        let mut eng = ArrivalEngine::new(cfg(0.10), 42);
        let dropped = (0..10_000)
            .filter(|i| eng.decide(1.0 * *i as f64, 100).is_drop())
            .count();
        assert!(
            (900..=1100).contains(&dropped),
            "dropped {dropped} of 10000, expected 900-1100"
        );
    }

    #[test]
    fn zero_loss_never_drops() {
        let mut eng = ArrivalEngine::new(cfg(0.0), 1);
        for i in 0..1000 {
            assert!(!eng.decide(i as f64, 10).is_drop());
        }
    }

    #[test]
    fn deterministic_under_same_seed_same_arrivals() {
        let run = |seed: u64| {
            let mut c = DirectionConfig::default();
            c.loss = 0.2;
            c.delay_ms = 10.0;
            c.jitter_ms = 5.0;
            c.duplicate_p = 0.2;
            c.reorder_p = 0.3;
            c.reorder_delay_ms = 40.0;
            c.rate_bps = Some(8_000_000);
            let mut e = ArrivalEngine::new(c, seed);
            (0..2000)
                .map(|i| match e.decide(i as f64 * 0.1, 64) {
                    Decision::Drop(r) => format!("D:{r}"),
                    Decision::Enqueue { due_ms, extra_dup } => format!("E:{due_ms:.6}:{extra_dup}"),
                    Decision::KillRst => "K".into(),
                })
                .collect::<Vec<_>>()
        };
        // Same seed + same arrival pattern => identical decisions.
        assert_eq!(run(99), run(99));
        assert_ne!(run(99), run(100));
    }

    #[test]
    fn mtu_boundary_is_exact() {
        let mut c = DirectionConfig::default();
        c.mtu = Some(1200);
        let mut eng = ArrivalEngine::new(c, 7);
        assert!(!eng.decide(0.0, 1200).is_drop(), "== mtu passes");
        assert!(eng.decide(0.0, 1201).is_drop(), "> mtu drops");
        assert!(!eng.decide(0.0, 1199).is_drop(), "< mtu passes");
        assert_eq!(eng.stats.dropped_mtu, 1);
    }

    #[test]
    fn delay_and_jitter_bounds() {
        let mut c = DirectionConfig::default();
        c.delay_ms = 20.0;
        c.jitter_ms = 10.0;
        let mut e = ArrivalEngine::new(c, 3);
        for i in 0..5000u32 {
            let t = i as f64;
            match e.decide(t, 10) {
                Decision::Enqueue { due_ms, .. } => {
                    let d = due_ms - t;
                    assert!((20.0..30.0).contains(&d), "delay {d} out of [20,30]");
                }
                other => panic!("unexpected {other:?}"),
            }
        }
    }

    #[test]
    fn jitter_is_roughly_uniform() {
        // Coarse chi-square-ish: quartile counts must be within 15% of 25%.
        let mut c = DirectionConfig::default();
        c.jitter_ms = 100.0;
        let mut e = ArrivalEngine::new(c, 11);
        let mut q = [0usize; 4];
        for i in 0..40_000u32 {
            if let Decision::Enqueue { due_ms, .. } = e.decide(i as f64, 1) {
                let j = due_ms - i as f64;
                q[(j / 25.0) as usize] += 1;
            }
        }
        for (k, &n) in q.iter().enumerate() {
            let expect = 10_000.0;
            assert!(
                (n as f64 - expect).abs() / expect < 0.15,
                "jitter quartile {k}: {n} vs {expect}"
            );
        }
    }

    #[test]
    fn reorder_adds_extra_delay_probabilistically() {
        let mut c = DirectionConfig::default();
        c.reorder_p = 0.5;
        c.reorder_delay_ms = 75.0;
        let mut e = ArrivalEngine::new(c, 17);
        let mut hit = 0;
        let n = 20_000;
        for i in 0..n {
            if let Decision::Enqueue { due_ms, .. } = e.decide(i as f64, 1) {
                let d = due_ms - i as f64;
                if d > 70.0 {
                    hit += 1;
                    assert!((75.0..80.0).contains(&d), "reorder delta {d}");
                } else {
                    assert!(d < 1.0, "unexpected delay {d}");
                }
            }
        }
        assert!((hit as f64 / n as f64 - 0.5).abs() < 0.05, "hit rate {hit}/{n}");
        assert_eq!(e.stats.reorder_hits, hit as u64);
    }

    #[test]
    fn duplicate_injects_extra_copy() {
        let mut c = DirectionConfig::default();
        c.duplicate_p = 1.0;
        let mut e = ArrivalEngine::new(c, 5);
        for i in 0..1000 {
            match e.decide(i as f64, 10) {
                Decision::Enqueue { extra_dup, .. } => assert!(extra_dup),
                other => panic!("unexpected {other:?}"),
            }
        }
        assert_eq!(e.stats.duplicates, 1000);
    }

    #[test]
    fn token_bucket_defers_exactly() {
        // 8000 bps = 1000 B/s, burst 1000 B. 500 B packets:
        // t=0: burst covers pkt1 (500 left) and pkt2 (0 left).
        // pkt3 must wait 500 B / 1000 B/s = 500 ms.
        let mut c = DirectionConfig::default();
        c.rate_bps = Some(8000);
        c.rate_burst_bytes = 1000;
        let mut e = ArrivalEngine::new(c, 1);
        let mk = |e: &mut ArrivalEngine, t: f64| match e.decide(t, 500) {
            Decision::Enqueue { due_ms, .. } => due_ms,
            o => panic!("{o:?}"),
        };
        assert!((mk(&mut e, 0.0) - 0.0).abs() < 1e-9);
        assert!((mk(&mut e, 0.0) - 0.0).abs() < 1e-9);
        let d3 = mk(&mut e, 0.0);
        assert!((d3 - 500.0).abs() < 1e-6, "pkt3 due {d3}, want 500");
        // pkt4 at t=0 waits 1000ms (debt 1000 B after pkt3's charge).
        let d4 = mk(&mut e, 0.0);
        assert!((d4 - 1000.0).abs() < 1e-6, "pkt4 due {d4}, want 1000");
        // Arrival at t=250: 250 B refilled, but debt was 1000 B (pkts 3+4
        // both charged) => debt 750 B, then pkt5 adds 500 => 1250 B debt.
        // FIFO shaper: pkt5 departs after pkt4 (due 1000) at 1000+500=1500.
        let d5 = mk(&mut e, 250.0);
        assert!((d5 - 1500.0).abs() < 1e-6, "pkt5 due {d5}, want 1500");
        assert_eq!(e.stats.rate_deferred, 3);
    }

    #[test]
    fn token_bucket_sustains_rate_without_defer() {
        // 10 kB/s, 100 B every 10 ms: exactly on rate, never defers.
        let mut c = DirectionConfig::default();
        c.rate_bps = Some(80_000);
        c.rate_burst_bytes = 1000;
        let mut e = ArrivalEngine::new(c, 2);
        for i in 0..10_000u32 {
            match e.decide(i as f64 * 10.0, 100) {
                Decision::Enqueue { due_ms, .. } => {
                    assert!(due_ms <= i as f64 * 10.0 + 1e-6, "unexpected defer at {i}");
                }
                o => panic!("{o:?}"),
            }
        }
        assert_eq!(e.stats.rate_deferred, 0);
    }

    #[test]
    fn blackout_window_drops_inside_only() {
        let mut c = DirectionConfig::default();
        c.blackouts.push(Window { start_ms: 500, end_ms: 1500 });
        let mut e = ArrivalEngine::new(c, 9);
        assert!(!e.decide(499.9, 10).is_drop());
        assert!(e.decide(500.0, 10).is_drop(), "start inclusive");
        assert!(e.decide(1499.9, 10).is_drop());
        assert!(!e.decide(1500.0, 10).is_drop(), "end exclusive");
        assert_eq!(e.stats.dropped_blackout, 2);
    }

    #[test]
    fn nat_rebind_cadence() {
        let mut c = DirectionConfig::default();
        c.nat_change_every_ms = 300;
        let mut e = ArrivalEngine::new(c, 4);
        let mut rebinds = 0;
        for i in 0..1250 {
            if e.should_rebind_now(i as f64) {
                rebinds += 1;
            }
        }
        assert_eq!(rebinds, 4, "300/600/900/1200");
        assert_eq!(e.stats.nat_rebinds, 4);
    }

    #[test]
    fn tcp_kill_windows() {
        let mut c = DirectionConfig::default();
        c.tcp_kills.push(TcpKill {
            window: Window { start_ms: 100, end_ms: 200 },
            style: KillStyle::Rst,
        });
        c.tcp_kills.push(TcpKill {
            window: Window { start_ms: 300, end_ms: 400 },
            style: KillStyle::Hold,
        });
        let mut e = ArrivalEngine::new(c, 8);
        assert!(!matches!(e.decide(99.0, 10), Decision::KillRst));
        assert!(matches!(e.decide(100.0, 10), Decision::KillRst));
        assert!(e.decide(350.0, 10).is_drop(), "hold drops silently");
        assert!(!e.decide(450.0, 10).is_drop());
        assert_eq!(e.stats.dropped_blackout, 1);
    }

    #[test]
    fn ge_steady_state_matches_theory() {
        // Steady-state bad fraction = p/(p+r); with h=1.0 loss rate equals it.
        let (p, r) = (0.02f64, 0.5f64);
        let mut c = DirectionConfig::default();
        c.ge = Some(Ge { p, r, h: 1.0 });
        let mut e = ArrivalEngine::new(c, 77);
        let n = 200_000;
        let dropped = (0..n).filter(|i| e.decide(*i as f64, 10).is_drop()).count() as f64;
        let theory = p / (p + r);
        assert!((dropped / n as f64 - theory).abs() < 0.005);
    }

    #[test]
    fn ge_is_bursty_compared_to_independent() {
        let mut c = DirectionConfig::default();
        c.ge = Some(Ge { p: 0.05, r: 0.1, h: 1.0 });
        let mut e = ArrivalEngine::new(c, 5);
        let mut runs: Vec<usize> = Vec::new();
        let mut cur = 0;
        for i in 0..20_000 {
            let d = e.decide(i as f64, 10).is_drop();
            if d {
                cur += 1;
            } else if cur > 0 {
                runs.push(cur);
                cur = 0;
            }
        }
        if cur > 0 {
            runs.push(cur);
        }
        let mean_run = runs.iter().sum::<usize>() as f64 / runs.len() as f64;
        // Expected run length for r=0.1 is 1/r = 10; independent loss with
        // the same mean rate (33%) would give ~1.5.
        assert!(mean_run > 5.0, "mean run {mean_run} not bursty");
        assert_eq!(runs.iter().sum::<usize>() as u64, e.stats.dropped_ge);
    }

    #[test]
    fn queue_releases_in_due_seq_order() {
        let mut q = ImpairQueue::new(10);
        // Push out of due order; same-due ties break by seq.
        q.push(100.0, 0.0, 0, 0, b"late".to_vec());
        q.push(10.0, 0.0, 0, 0, b"early".to_vec());
        q.push(10.0, 0.0, 0, 0, b"early2".to_vec());
        q.push(50.0, 0.0, 0, 0, b"mid".to_vec());
        assert!(q.pop_due(9.9).is_empty());
        let got: Vec<Vec<u8>> = q.pop_due(10.0).into_iter().map(|p| p.data).collect();
        assert_eq!(got, vec![b"early".to_vec(), b"early2".to_vec()]);
        let got: Vec<Vec<u8>> = q.pop_due(1000.0).into_iter().map(|p| p.data).collect();
        assert_eq!(got, vec![b"mid".to_vec(), b"late".to_vec()]);
        assert!(q.is_empty());
    }

    #[test]
    fn queue_capacity_enforced() {
        let mut q = ImpairQueue::new(2);
        assert!(q.push(1.0, 0.0, 0, 0, vec![0; 4]).is_some());
        assert!(q.push(1.0, 0.0, 0, 0, vec![0; 4]).is_some());
        assert!(q.push(1.0, 0.0, 0, 0, vec![0; 4]).is_none());
    }

    #[test]
    fn direction_seeds_differ() {
        assert_ne!(direction_seed(42, 1), direction_seed(42, 2));
        assert_ne!(direction_seed(42, 1), direction_seed(43, 1));
    }
}
