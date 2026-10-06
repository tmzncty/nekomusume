//! Per-direction statistics and the JSON writer.
//!
//! Counters are plain `u64`s owned by the engine (single writer per
//! direction); the process-exit writer reads them after the pump threads
//! have joined, so no atomics are needed on the hot path.

use std::io::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

pub const BUCKET_BOUNDS_MS: [f64; 11] = [
    1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 5000.0,
];
pub const N_BUCKETS: usize = BUCKET_BOUNDS_MS.len() + 1; // last bucket = >=5000

pub fn bucket_label(i: usize) -> String {
    if i == 0 {
        format!("<{}", BUCKET_BOUNDS_MS[0] as u64)
    } else if i < BUCKET_BOUNDS_MS.len() {
        format!(
            "{}-{}",
            BUCKET_BOUNDS_MS[i - 1] as u64,
            BUCKET_BOUNDS_MS[i] as u64
        )
    } else {
        format!(">={}", BUCKET_BOUNDS_MS[BUCKET_BOUNDS_MS.len() - 1] as u64)
    }
}

fn bucket_of(latency_ms: f64) -> usize {
    for (i, b) in BUCKET_BOUNDS_MS.iter().enumerate() {
        if latency_ms < *b {
            return i;
        }
    }
    BUCKET_BOUNDS_MS.len()
}

/// Public bucket index for the pump-side accountant.
pub fn bucket_index(latency_ms: f64) -> usize {
    bucket_of(latency_ms)
}

#[derive(Debug, Default, Clone)]
pub struct DirectionStats {
    pub received_pkts: u64,
    pub received_bytes: u64,
    pub sent_pkts: u64,
    pub sent_bytes: u64,
    pub dropped_random: u64,
    pub dropped_ge: u64,
    pub dropped_mtu: u64,
    pub dropped_blackout: u64,
    pub dropped_overflow: u64,
    pub duplicates: u64,
    pub reorder_hits: u64,
    pub rate_deferred: u64,
    pub nat_rebinds: u64,
    /// TCP only: connections torn down by RST kill windows.
    pub tcp_rst_kills: u64,
    /// Enqueue-time decisions that scheduled extra delay beyond base+jitter
    /// (i.e. reordering injections) are counted in `reorder_hits`;
    /// `latency_*` below measures actually-delivered end-to-end queueing.
    pub latency_count: u64,
    pub latency_sum_ms: f64,
    pub latency_min_ms: f64,
    pub latency_max_ms: f64,
    pub hist: [u64; N_BUCKETS],
}

impl DirectionStats {
    pub fn record_delivered(&mut self, latency_ms: f64) {
        if self.latency_count == 0 || latency_ms < self.latency_min_ms {
            self.latency_min_ms = latency_ms;
        }
        if latency_ms > self.latency_max_ms {
            self.latency_max_ms = latency_ms;
        }
        self.latency_count += 1;
        self.latency_sum_ms += latency_ms;
        self.hist[bucket_of(latency_ms)] += 1;
    }

    pub fn total_dropped(&self) -> u64 {
        self.dropped_random
            + self.dropped_ge
            + self.dropped_mtu
            + self.dropped_blackout
            + self.dropped_overflow
    }

    fn json(&self, indent: &str) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("\"received_pkts\": {}", self.received_pkts));
        lines.push(format!("\"received_bytes\": {}", self.received_bytes));
        lines.push(format!("\"sent_pkts\": {}", self.sent_pkts));
        lines.push(format!("\"sent_bytes\": {}", self.sent_bytes));
        lines.push(format!(
            "\"dropped\": {{\"random\": {}, \"ge\": {}, \"mtu\": {}, \"blackout\": {}, \"overflow\": {}, \"total\": {}}}",
            self.dropped_random,
            self.dropped_ge,
            self.dropped_mtu,
            self.dropped_blackout,
            self.dropped_overflow,
            self.total_dropped()
        ));
        lines.push(format!("\"duplicates\": {}", self.duplicates));
        lines.push(format!("\"reorder_hits\": {}", self.reorder_hits));
        lines.push(format!("\"rate_deferred\": {}", self.rate_deferred));
        lines.push(format!("\"nat_rebinds\": {}", self.nat_rebinds));
        lines.push(format!("\"tcp_rst_kills\": {}", self.tcp_rst_kills));
        let (mean, p) = if self.latency_count > 0 {
            (
                self.latency_sum_ms / self.latency_count as f64,
                self.latency_count,
            )
        } else {
            (0.0, 0)
        };
        lines.push(format!(
            "\"latency_ms\": {{\"count\": {p}, \"min\": {min:.3}, \"max\": {max:.3}, \"mean\": {mean:.3}}}",
            min = self.latency_min_ms,
            max = self.latency_max_ms,
        ));
        let hist: Vec<String> = (0..N_BUCKETS)
            .filter(|&i| self.hist[i] > 0)
            .map(|i| format!("\"{}\": {}", bucket_label(i), self.hist[i]))
            .collect();
        lines.push(format!("\"latency_hist_ms\": {{{}}}", hist.join(", ")));
        let mut s = String::with_capacity(1024);
        s.push_str("{\n");
        s.push_str(
            &lines
                .iter()
                .map(|l| format!("{indent}  {l}"))
                .collect::<Vec<_>>()
                .join(",\n"),
        );
        s.push_str(&format!("\n{indent}}}"));
        s
    }
}

fn unix_ms(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn unix_ms_now() -> u64 {
    unix_ms(SystemTime::now())
}

#[derive(Debug, Clone)]
pub struct StatsDoc {
    pub mode: String,
    pub seed: u64,
    pub started_unix_ms: u64,
    pub ended_unix_ms: u64,
    pub duration_ms: u64,
    pub c2s: DirectionStats,
    pub s2c: DirectionStats,
}

impl StatsDoc {
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(2048);
        s.push_str("{\n");
        s.push_str(&format!("  \"mode\": \"{}\",\n", self.mode));
        s.push_str(&format!("  \"seed\": {},\n", self.seed));
        s.push_str(&format!("  \"started_unix_ms\": {},\n", self.started_unix_ms));
        s.push_str(&format!("  \"ended_unix_ms\": {},\n", self.ended_unix_ms));
        s.push_str(&format!("  \"duration_ms\": {},\n", self.duration_ms));
        s.push_str("  \"c2s\": ");
        s.push_str(&self.c2s.json("  ").trim_start_matches('\n').to_string());
        s.push_str(",\n  \"s2c\": ");
        s.push_str(&self.s2c.json("  ").trim_start_matches('\n').to_string());
        s.push_str("\n}\n");
        s
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        f.write_all(self.to_json().as_bytes())?;
        f.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_is_parseable_by_a_standard_parser() {
        // std has no JSON parser; structural assertions instead: balanced
        // braces, quoted keys, trailing commas absent. Full parse is covered
        // by the end-to-end python check in the smoke script.
        let mut st = DirectionStats::default();
        st.received_pkts = 10;
        st.sent_pkts = 9;
        st.dropped_random = 1;
        for i in 0..9 {
            st.record_delivered(50.0 + i as f64);
        }
        let doc = StatsDoc {
            mode: "udp".into(),
            seed: 42,
            started_unix_ms: 1,
            ended_unix_ms: 2,
            duration_ms: 1,
            c2s: st.clone(),
            s2c: DirectionStats::default(),
        };
        let j = doc.to_json();
        assert!(j.starts_with('{') && j.trim_end().ends_with('}'));
        let open = j.matches('{').count();
        let close = j.matches('}').count();
        assert_eq!(open, close, "unbalanced braces:\n{j}");
        let ob = j.matches('[').count();
        let cb = j.matches(']').count();
        assert_eq!(ob, cb);
        assert!(!j.contains(",\n}"), "trailing comma before close brace:\n{j}");
        assert!(j.contains("\"latency_ms\""));
        assert!(j.contains("\"50-100\""), "histogram bucket label missing:\n{j}");
    }

    #[test]
    fn buckets_partition() {
        assert_eq!(bucket_of(0.5), 0);
        assert_eq!(bucket_of(1.5), 1);
        assert_eq!(bucket_of(49.9), 5);
        assert_eq!(bucket_of(4999.0), 10);
        assert_eq!(bucket_of(6000.0), 11);
    }
}
