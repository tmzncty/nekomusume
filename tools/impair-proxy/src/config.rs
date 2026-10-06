//! CLI configuration: hand-rolled argv parser (zero dependencies).
//!
//! Every impairment knob exists twice — `--c2s-*` (client -> upstream) and
//! `--s2c-*` (upstream -> client). Defaults are zero impairment, so the proxy
//! is a plain forwarder out of the box.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Udp,
    Tcp,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Udp => "udp",
            Mode::Tcp => "tcp",
        }
    }
}

/// Gilbert-Elliott burst-loss model: two-state Markov chain.
/// `p` = P(good -> bad), `r` = P(bad -> good), `h` = loss prob while bad.
#[derive(Debug, Clone, Copy)]
pub struct Ge {
    pub p: f64,
    pub r: f64,
    pub h: f64,
}

/// Scheduled blackout window in proxy-relative milliseconds: [start, end).
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub start_ms: u64,
    pub end_ms: u64,
}

impl Window {
    fn parse(s: &str) -> Result<Self, String> {
        let (a, b) = s.split_once(':').ok_or("window must be START_MS:END_MS")?;
        let start: u64 = a.trim().parse().map_err(|_| format!("bad start {a:?}"))?;
        let end: u64 = b.trim().parse().map_err(|_| format!("bad end {b:?}"))?;
        if end <= start {
            return Err(format!("window end ({end}) must be > start ({start})"));
        }
        Ok(Window { start_ms: start, end_ms: end })
    }
    pub fn contains(&self, t_ms: f64) -> bool {
        (self.start_ms as f64) <= t_ms && t_ms < (self.end_ms as f64)
    }
}

/// How a TCP kill window tears the connection down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillStyle {
    /// RST both sides immediately (visible failure).
    Rst,
    /// Silently stop forwarding until the window ends (blackhole/stall).
    Hold,
}

impl KillStyle {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "rst" => Ok(KillStyle::Rst),
            "hold" => Ok(KillStyle::Hold),
            other => Err(format!("kill style must be rst|hold, got {other:?}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TcpKill {
    pub window: Window,
    pub style: KillStyle,
}

/// Per-direction impairment configuration. All optional knobs default to
/// "off"; `rate_bps` uses shaper semantics (defer, not drop).
#[derive(Debug, Clone, Default)]
pub struct DirectionConfig {
    pub loss: f64,                 // independent drop probability [0,1)
    pub ge: Option<Ge>,            // Gilbert-Elliott burst loss
    pub delay_ms: f64,             // fixed one-way delay
    pub jitter_ms: f64,            // uniform extra delay in [0, jitter]
    pub reorder_p: f64,            // P(extra reorder delay on a packet)
    pub reorder_delay_ms: f64,     // the extra delay applied on reorder hit
    pub duplicate_p: f64,          // P(packet is additionally sent twice)
    pub rate_bps: Option<u64>,     // token-bucket shaper, bits/s
    pub rate_burst_bytes: u64,     // bucket capacity (default: 1s of rate)
    pub mtu: Option<usize>,        // drop datagrams strictly longer than this
    pub blackouts: Vec<Window>,    // proxy-relative blackout windows
    pub nat_change_every_ms: u64,  // rebind outbound UDP socket periodically
    pub tcp_kills: Vec<TcpKill>,   // TCP only: teardown windows
}

impl DirectionConfig {
    fn impaired(&self) -> bool {
        self.loss > 0.0
            || self.ge.is_some()
            || self.delay_ms > 0.0
            || self.jitter_ms > 0.0
            || self.reorder_p > 0.0
            || self.duplicate_p > 0.0
            || self.rate_bps.is_some()
            || self.mtu.is_some()
            || !self.blackouts.is_empty()
            || self.nat_change_every_ms > 0
            || !self.tcp_kills.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Mode,
    pub bind: Option<SocketAddr>,
    pub upstream: Option<SocketAddr>,
    pub seed: u64,
    pub stats_file: Option<String>,
    pub port_file: Option<String>,
    /// 0 = run until SIGINT/SIGTERM.
    pub duration_ms: u64,
    pub verbose: bool,
    /// Queue cap per direction (packets); excess is dropped as `overflow`.
    pub queue_cap: usize,
    pub c2s: DirectionConfig,
    pub s2c: DirectionConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode: Mode::Udp,
            bind: None,
            upstream: None,
            seed: 1,
            stats_file: None,
            port_file: None,
            duration_ms: 0,
            verbose: false,
            queue_cap: 100_000,
            c2s: DirectionConfig::default(),
            s2c: DirectionConfig::default(),
        }
    }
}

fn parse_prob(name: &str, v: &str) -> Result<f64, String> {
    let p: f64 = v.parse().map_err(|_| format!("{name}: not a number: {v:?}"))?;
    if !(0.0..=1.0).contains(&p) {
        return Err(format!("{name}: probability must be within [0,1], got {p}"));
    }
    Ok(p)
}

fn parse_f64_ms(name: &str, v: &str) -> Result<f64, String> {
    let x: f64 = v.parse().map_err(|_| format!("{name}: not a number: {v:?}"))?;
    if x < 0.0 {
        return Err(format!("{name}: must be >= 0, got {x}"));
    }
    Ok(x)
}

fn parse_u64(name: &str, v: &str) -> Result<u64, String> {
    v.parse().map_err(|_| format!("{name}: not an integer: {v:?}"))
}

fn parse_addr(name: &str, v: &str) -> Result<SocketAddr, String> {
    v.parse().map_err(|_| format!("{name}: expected HOST:PORT, got {v:?}"))
}

/// Raw argument bag: scalar keys keep the last value, repeatable keys
/// (`*-blackout`, `*-kill`) accumulate, boolean flags need no value.
struct Args {
    scalars: HashMap<String, String>,
    repeats: HashMap<String, Vec<String>>,
    flags: std::collections::HashSet<String>,
}

/// Keys that take no value.
const FLAG_KEYS: [&str; 1] = ["verbose"];

fn split_eq(arg: &str) -> Option<(String, String)> {
    let (k, v) = arg.split_once('=')?;
    Some((k.to_string(), v.to_string()))
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut scalars = HashMap::new();
        let mut repeats: HashMap<String, Vec<String>> = HashMap::new();
        let mut flags = std::collections::HashSet::new();
        let mut i = 0;
        while i < argv.len() {
            let arg = argv[i].clone();
            if !arg.starts_with("--") {
                return Err(format!("unexpected positional argument {arg:?}"));
            }
            let (key, inline_val) = if let Some((k, v)) = split_eq(&arg) {
                (k, Some(v))
            } else {
                (arg.clone(), None)
            };
            let key = key.trim_start_matches("--").to_string();
            if FLAG_KEYS.contains(&key.as_str()) {
                if let Some(v) = inline_val {
                    scalars.insert(key.clone(), v);
                } else {
                    flags.insert(key);
                }
                i += 1;
                continue;
            }
            let val = match inline_val {
                Some(v) => v,
                None => {
                    i += 1;
                    argv.get(i).cloned().ok_or(format!("--{key} needs a value"))?
                }
            };
            if key.ends_with("-blackout") || key.ends_with("-kill") {
                repeats.entry(key).or_default().push(val);
            } else {
                scalars.insert(key, val);
            }
            i += 1;
        }
        Ok(Args { scalars, repeats, flags })
    }
}

pub fn parse_config(argv: &[String]) -> Result<Config, String> {
    let args = Args::parse(argv)?;
    let mut cfg = Config::default();

    if let Some(v) = args.scalars.get("mode") {
        cfg.mode = match v.as_str() {
            "udp" => Mode::Udp,
            "tcp" => Mode::Tcp,
            other => return Err(format!("--mode must be udp|tcp, got {other:?}")),
        };
    }
    if let Some(v) = args.scalars.get("bind") {
        cfg.bind = Some(parse_addr("--bind", v)?);
    }
    if let Some(v) = args.scalars.get("upstream") {
        cfg.upstream = Some(parse_addr("--upstream", v)?);
    }
    if let Some(v) = args.scalars.get("seed") {
        cfg.seed = parse_u64("--seed", v)?;
    }
    if let Some(v) = args.scalars.get("stats-file") {
        cfg.stats_file = Some(v.clone());
    }
    if let Some(v) = args.scalars.get("port-file") {
        cfg.port_file = Some(v.clone());
    }
    if let Some(v) = args.scalars.get("duration-ms") {
        cfg.duration_ms = parse_u64("--duration-ms", v)?;
    }
    if let Some(v) = args.scalars.get("queue-cap") {
        cfg.queue_cap = parse_u64("--queue-cap", v)? as usize;
        if cfg.queue_cap == 0 {
            return Err("--queue-cap must be > 0".into());
        }
    }
    if args.flags.contains("verbose") {
        cfg.verbose = true;
    } else if let Some(v) = args.scalars.get("verbose") {
        cfg.verbose = matches!(v.as_str(), "1" | "true" | "yes" | "on");
    }

    for (prefix, dir) in [("c2s", &mut cfg.c2s), ("s2c", &mut cfg.s2c)] {
        let g = |k: &str| args.scalars.get(&format!("{prefix}-{k}")).cloned();
        if let Some(v) = g("loss") {
            dir.loss = parse_prob(&format!("{prefix}-loss"), &v)?;
        }
        if let Some(v) = g("ge") {
            let parts: Vec<&str> = v.split(':').collect();
            if parts.len() != 3 {
                return Err(format!("{prefix}-ge must be P:R:H (e.g. 0.02:0.5:0.9), got {v:?}"));
            }
            let p = parse_prob(&format!("{prefix}-ge p"), parts[0])?;
            let r = parse_prob(&format!("{prefix}-ge r"), parts[1])?;
            let h = parse_prob(&format!("{prefix}-ge h"), parts[2])?;
            if p <= 0.0 || r <= 0.0 {
                return Err(format!("{prefix}-ge: p and r must be > 0 (else no transitions)"));
            }
            dir.ge = Some(Ge { p, r, h });
        }
        if let Some(v) = g("delay-ms") {
            dir.delay_ms = parse_f64_ms(&format!("{prefix}-delay-ms"), &v)?;
        }
        if let Some(v) = g("jitter-ms") {
            dir.jitter_ms = parse_f64_ms(&format!("{prefix}-jitter-ms"), &v)?;
        }
        if let Some(v) = g("reorder") {
            dir.reorder_p = parse_prob(&format!("{prefix}-reorder"), &v)?;
        }
        if let Some(v) = g("reorder-delay-ms") {
            dir.reorder_delay_ms = parse_f64_ms(&format!("{prefix}-reorder-delay-ms"), &v)?;
        }
        if dir.reorder_p > 0.0 && dir.reorder_delay_ms <= 0.0 {
            return Err(format!(
                "{prefix}-reorder > 0 requires {prefix}-reorder-delay-ms > 0"
            ));
        }
        if let Some(v) = g("duplicate") {
            dir.duplicate_p = parse_prob(&format!("{prefix}-duplicate"), &v)?;
        }
        if let Some(v) = g("rate-bps") {
            let bps = parse_u64(&format!("{prefix}-rate-bps"), &v)?;
            if bps == 0 {
                return Err(format!("{prefix}-rate-bps must be > 0"));
            }
            dir.rate_bps = Some(bps);
        }
        if let Some(v) = g("rate-burst-bytes") {
            dir.rate_burst_bytes = parse_u64(&format!("{prefix}-rate-burst-bytes"), &v)?;
        }
        if dir.rate_burst_bytes == 0 && dir.rate_bps.is_some() {
            // default burst: one second worth of shaped bytes, at least 1500
            dir.rate_burst_bytes = (dir.rate_bps.unwrap() / 8).max(1500);
        }
        if let Some(v) = g("mtu") {
            let mtu = parse_u64(&format!("{prefix}-mtu"), &v)? as usize;
            if mtu == 0 {
                return Err(format!("{prefix}-mtu must be > 0"));
            }
            dir.mtu = Some(mtu);
        }
        if let Some(list) = args.repeats.get(&format!("{prefix}-blackout")) {
            for v in list {
                dir.blackouts.push(Window::parse(v)?);
            }
        }
        if let Some(v) = g("nat-change-every-ms") {
            dir.nat_change_every_ms = parse_u64(&format!("{prefix}-nat-change-every-ms"), &v)?;
            if dir.nat_change_every_ms == 0 {
                return Err(format!("{prefix}-nat-change-every-ms must be > 0"));
            }
        }
        if let Some(list) = args.repeats.get(&format!("{prefix}-kill")) {
            for v in list {
                let parts: Vec<&str> = v.split(':').collect();
                if parts.len() != 3 {
                    return Err(format!(
                        "{prefix}-kill must be START_MS:END_MS:rst|hold, got {v:?}"
                    ));
                }
                let window = Window::parse(&format!("{}:{}", parts[0], parts[1]))?;
                let style = KillStyle::parse(parts[2])?;
                dir.tcp_kills.push(TcpKill { window, style });
            }
        }
    }

    // Cross-checks: impairment applicability per mode.
    if cfg.mode == Mode::Udp {
        for (name, d) in [("c2s", &cfg.c2s), ("s2c", &cfg.s2c)] {
            if !d.tcp_kills.is_empty() {
                return Err(format!("{name}-kill is TCP-only (UDP uses {name}-blackout)"));
            }
        }
    } else {
        for (name, d) in [("c2s", &cfg.c2s), ("s2c", &cfg.s2c)] {
            if d.nat_change_every_ms > 0 {
                return Err(format!(
                    "{name}-nat-change-every-ms is UDP-only (TCP rebind = teardown; use {name}-kill)"
                ));
            }
        }
    }

    Ok(cfg)
}

/// Human-readable summary line for verbose startup logging.
pub fn describe(cfg: &Config) -> String {
    let mut s = format!(
        "mode={} seed={} queue_cap={}",
        cfg.mode.as_str(),
        cfg.seed,
        cfg.queue_cap
    );
    if let Some(b) = cfg.bind {
        s.push_str(&format!(" bind={b}"));
    }
    if let Some(u) = cfg.upstream {
        s.push_str(&format!(" upstream={u}"));
    }
    for (name, d) in [("c2s", &cfg.c2s), ("s2c", &cfg.s2c)] {
        if !d.impaired() {
            continue;
        }
        let mut parts: Vec<String> = Vec::new();
        if d.loss > 0.0 {
            parts.push(format!("loss={}", d.loss));
        }
        if let Some(ge) = &d.ge {
            parts.push(format!("ge={}:{}:{}", ge.p, ge.r, ge.h));
        }
        if d.delay_ms > 0.0 || d.jitter_ms > 0.0 {
            parts.push(format!("delay={}+jitter{}", d.delay_ms, d.jitter_ms));
        }
        if d.reorder_p > 0.0 {
            parts.push(format!("reorder={}@{}ms", d.reorder_p, d.reorder_delay_ms));
        }
        if d.duplicate_p > 0.0 {
            parts.push(format!("dup={}", d.duplicate_p));
        }
        if let Some(bps) = d.rate_bps {
            parts.push(format!("rate={bps}bps/burst={}", d.rate_burst_bytes));
        }
        if let Some(mtu) = d.mtu {
            parts.push(format!("mtu={mtu}"));
        }
        for w in &d.blackouts {
            parts.push(format!("blackout={}:{}", w.start_ms, w.end_ms));
        }
        if d.nat_change_every_ms > 0 {
            parts.push(format!("nat-every={}ms", d.nat_change_every_ms));
        }
        for k in &d.tcp_kills {
            parts.push(format!(
                "kill={}:{}:{}",
                k.window.start_ms,
                k.window.end_ms,
                match k.style {
                    KillStyle::Rst => "rst",
                    KillStyle::Hold => "hold",
                }
            ));
        }
        s.push_str(&format!(" [{name}: {}]", parts.join(" ")));
    }
    s
}

/// Default poll interval for the send pumps: 1 ms resolution is plenty for
/// impairment semantics and keeps idle CPU near zero.
pub const PUMP_INTERVAL: Duration = Duration::from_millis(1);
