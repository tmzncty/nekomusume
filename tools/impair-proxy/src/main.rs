//! impair-proxy: userland loopback UDP/TCP impairment proxy.
//!
//! Zero-dependency Rust. See README.md for the full knob list.

mod config;
mod impair;
mod rng;
mod stats;
mod tcp;
mod udp;

use config::Mode;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    if argv.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(0);
    }
    if argv.iter().any(|a| a == "--version") {
        println!("impair-proxy {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let cfg = match config::parse_config(&argv) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("impair-proxy: {e}\n(use --help for usage)");
            std::process::exit(2);
        }
    };

    install_signal_handlers();

    if cfg.verbose {
        eprintln!("[impair-proxy] {}", config::describe(&cfg));
    }

    let result = match cfg.mode {
        Mode::Udp => udp::run_udp(&cfg),
        Mode::Tcp => tcp::run_tcp(&cfg),
    };

    let doc = match result {
        Ok(d) => d,
        Err(e) => {
            eprintln!("impair-proxy: {e}");
            std::process::exit(1);
        }
    };

    let out = cfg.stats_file.clone().unwrap_or_else(|| "impair-stats.json".into());
    if let Err(e) = doc.write_to_file(&out) {
        eprintln!("impair-proxy: stats write failed: {e}");
        std::process::exit(1);
    }
    if cfg.verbose {
        eprintln!(
            "[impair-proxy] stats -> {out} (c2s: recv {} send {} drop {} | s2c: recv {} send {} drop {})",
            doc.c2s.received_pkts,
            doc.c2s.sent_pkts,
            doc.c2s.total_dropped(),
            doc.s2c.received_pkts,
            doc.s2c.sent_pkts,
            doc.s2c.total_dropped(),
        );
    }
}

static SIGNAL_NOTE: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_sig: i32) {
    // Only async-signal-safe work: flip the flag the pumps poll.
    SIGNAL_NOTE.store(true, Ordering::Relaxed);
    udp::STOP_REQUESTED.store(true, Ordering::SeqCst);
}

fn install_signal_handlers() {
    // No libc dependency: declare the two symbols we need.
    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    unsafe {
        signal(SIGINT, on_signal as usize);
        signal(SIGTERM, on_signal as usize);
    }
}

fn print_help() {
    println!(
        "impair-proxy — userland loopback UDP/TCP impairment proxy (zero deps)

USAGE
  impair-proxy --mode udp --bind 127.0.0.1:9000 --upstream 127.0.0.1:9001 [knobs...]
  impair-proxy --mode tcp --bind 127.0.0.1:9000 --upstream 127.0.0.1:9001 [knobs...]

GLOBAL
  --mode udp|tcp             transport (default udp)
  --bind HOST:PORT           frontend listen address (default 127.0.0.1:0)
  --upstream HOST:PORT       where to forward
  --seed N                   deterministic seed (default 1)
  --stats-file PATH          JSON stats output (default impair-stats.json)
  --port-file PATH           write actual bound port here at startup
  --duration-ms N            stop after N ms (default: until SIGINT/SIGTERM)
  --queue-cap N              per-direction queue cap in packets (default 100000)
  --verbose                  log startup line and exit summary

PER-DIRECTION KNOBS  (replace PREFIX with c2s or s2c)
  --PREFIX-loss P                    independent drop probability [0,1]
  --PREFIX-ge P:R:H                  Gilbert-Elliott burst loss (p, r, bad-state loss)
  --PREFIX-delay-ms F                fixed delay
  --PREFIX-jitter-ms F               uniform extra delay [0,F]
  --PREFIX-reorder P                 probability of extra reorder delay
  --PREFIX-reorder-delay-ms F        the reorder delay
  --PREFIX-duplicate P               probability of sending an extra copy
  --PREFIX-rate-bps N                token-bucket shaper, bits/s
  --PREFIX-rate-burst-bytes N        bucket capacity (default: 1s of rate)
  --PREFIX-mtu N                     UDP: drop datagrams > N (PMTU blackhole)
                                     TCP: cap write chunks at N (ceiling)
  --PREFIX-blackout T1:T2            drop everything in [T1,T2) ms (repeatable)
  --PREFIX-nat-change-every-ms N     UDP: rebind outbound socket every N ms
  --PREFIX-kill T1:T2:rst|hold       TCP: RST-tear or stall connections (repeatable)

SIGNALS
  SIGINT/SIGTERM: flush and write stats, then exit 0.

DETERMINISM
  Given the same --seed and the same arrival sequence (times + lengths),
  every impairment decision is identical across runs. Scheduling jitter only
  affects wall-clock release, not what happens to a packet.
"
    );
}
