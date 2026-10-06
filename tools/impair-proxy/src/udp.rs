//! UDP runtime: one frontend socket, one upstream socket (rebindable for
//! NAT simulation), two directions, four threads.
//!
//! ```text
//! client ⇄ [frontend: bound] ──c2s──▶ [upstream sock] ⇄ upstream
//!                                   ◀──s2c──
//! ```
//!
//! Per direction: a reader thread owns the ArrivalEngine (all seeded
//! decisions) and a pump thread owns the queue + send accounting. Threads
//! share a Mutex<ImpairQueue> + Condvar and an AtomicBool stop flag.

use crate::config::Config;
use crate::impair::{direction_seed, ArrivalEngine, ImpairQueue, SendAccountant};
use crate::stats::{unix_ms_now, StatsDoc};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const RECV_BUF: usize = 65536;

struct SharedQueue {
    q: Mutex<ImpairQueue>,
    cv: Condvar,
}

impl SharedQueue {
    fn new(cap: usize) -> Arc<Self> {
        Arc::new(SharedQueue { q: Mutex::new(ImpairQueue::new(cap)), cv: Condvar::new() })
    }
}

/// Rebindable upstream socket: generation counter for change detection.
struct UpstreamSock {
    genctr: AtomicU64,
    sock: Mutex<UdpSocket>,
}

impl UpstreamSock {
    fn new(addr_family_bind: &str) -> std::io::Result<Arc<Self>> {
        let s = UdpSocket::bind(addr_family_bind)?;
        s.set_read_timeout(Some(Duration::from_millis(50)))?;
        Ok(Arc::new(UpstreamSock { genctr: AtomicU64::new(0), sock: Mutex::new(s) }))
    }
}

fn now_ms(start: &Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

pub fn run_udp(cfg: &Config) -> Result<StatsDoc, String> {
    let upstream: SocketAddr = cfg
        .upstream
        .ok_or_else(|| "UDP mode requires --upstream HOST:PORT".to_string())?;

    let frontend = UdpSocket::bind(cfg.bind.unwrap_or_else(|| "127.0.0.1:0".parse().unwrap()))
        .map_err(|e| format!("bind frontend: {e}"))?;
    frontend
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    let local = frontend.local_addr().map_err(|e| e.to_string())?;

    let up = UpstreamSock::new("127.0.0.1:0").map_err(|e| format!("bind upstream sock: {e}"))?;

    let start = Arc::new(Instant::now());
    let stop = Arc::new(AtomicBool::new(false));
    let active_client: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));

    let c2s_q = SharedQueue::new(cfg.queue_cap);
    let s2c_q = SharedQueue::new(cfg.queue_cap);

    // ---- c2s reader: frontend -> decide -> c2s queue ----
    let c2s_reader = {
        let fe = frontend.try_clone().map_err(|e| e.to_string())?;
        let q = c2s_q.clone();
        let stop = stop.clone();
        let start = start.clone();
        let clients = active_client.clone();
        let eng_cfg = cfg.c2s.clone();
        let seed = direction_seed(cfg.seed, 0xC25);
        std::thread::Builder::new()
            .name("c2s-read".into())
            .spawn(move || {
                let mut eng = ArrivalEngine::new(eng_cfg, seed);
                let mut buf = [0u8; RECV_BUF];
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    match fe.recv_from(&mut buf) {
                        Ok((n, src)) => {
                            *clients.lock().unwrap() = Some(src);
                            let now = now_ms(&start);
                            match eng.decide(now, n) {
                                crate::impair::Decision::Drop(_) => {}
                                crate::impair::Decision::KillRst => {}
                                crate::impair::Decision::Enqueue { due_ms, extra_dup } => {
                                    let mut g = q.q.lock().unwrap();
                                    if g
                                        .push(due_ms, now, 0, 0, buf[..n].to_vec())
                                        .is_none()
                                    {
                                        eng.stats_inc_overflow();
                                    }
                                    if extra_dup
                                        && g.push(due_ms, now, 0, 1, buf[..n].to_vec()).is_none()
                                    {
                                        eng.stats_inc_overflow();
                                    }
                                    drop(g);
                                    q.cv.notify_one();
                                }
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                eng
            })
            .map_err(|e| e.to_string())?
    };

    // ---- c2s pump: queue -> upstream (with NAT rebind) ----
    let c2s_pump = {
        let q = c2s_q.clone();
        let up = up.clone();
        let stop = stop.clone();
        let start = start.clone();
        let upstream = upstream;
        let nat_every = cfg.c2s.nat_change_every_ms;
        std::thread::Builder::new()
            .name("c2s-pump".into())
            .spawn(move || {
                let mut next_rebind_ms = if nat_every > 0 { nat_every as f64 } else { f64::INFINITY };
                let mut acct = SendAccountant::default();
                loop {
                    if stop.load(Ordering::Relaxed) {
                        // flush everything still queued, then exit
                        let due = { q.q.lock().unwrap().pop_due(f64::INFINITY) };
                        send_batch(
                            &up, upstream, &due, &mut acct, f64::INFINITY, &mut next_rebind_ms, nat_every,
                        );
                        break;
                    }
                    let now = now_ms(&start);
                    let batch = {
                        let mut g = q.q.lock().unwrap();
                        if g.is_empty() {
                            drop(g);
                            // park until notified or 20 ms
                            let g2 = q.q.lock().unwrap();
                            let _ = q.cv.wait_timeout(g2, Duration::from_millis(20)).unwrap();
                            Vec::new()
                        } else {
                            let batch = g.pop_due(now);
                            if batch.is_empty() {
                                drop(g);
                                std::thread::sleep(Duration::from_micros(300));
                            }
                            batch
                        }
                    };
                    if !batch.is_empty() {
                        send_batch(&up, upstream, &batch, &mut acct, now, &mut next_rebind_ms, nat_every);
                    }
                }
                acct
            })
            .map_err(|e| e.to_string())?
    };

    // ---- s2c reader: upstream sock -> decide -> s2c queue ----
    let s2c_reader = {
        let up = up.clone();
        let q = s2c_q.clone();
        let stop = stop.clone();
        let start = start.clone();
        let upstream = upstream;
        let eng_cfg = cfg.s2c.clone();
        let seed = direction_seed(cfg.seed, 0x52C);
        std::thread::Builder::new()
            .name("s2c-read".into())
            .spawn(move || {
                let mut eng = ArrivalEngine::new(eng_cfg, seed);
                let mut buf = [0u8; RECV_BUF];
                let mut my_gen = u64::MAX;
                let mut sock: Option<UdpSocket> = None;
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let gen_now = up.genctr.load(Ordering::Acquire);
                    if sock.is_none() || gen_now != my_gen {
                        let s = up.sock.lock().unwrap().try_clone().ok();
                        if let Some(s) = s {
                            my_gen = gen_now;
                            sock = Some(s);
                        }
                    }
                    let Some(s) = sock.as_ref() else {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    };
                    match s.recv_from(&mut buf) {
                        Ok((n, src)) if src == upstream => {
                            let now = now_ms(&start);
                            match eng.decide(now, n) {
                                crate::impair::Decision::Drop(_) => {}
                                crate::impair::Decision::KillRst => {}
                                crate::impair::Decision::Enqueue { due_ms, extra_dup } => {
                                    let mut g = q.q.lock().unwrap();
                                    if g.push(due_ms, now, 0, 0, buf[..n].to_vec()).is_none() {
                                        eng.stats_inc_overflow();
                                    }
                                    if extra_dup
                                        && g.push(due_ms, now, 0, 1, buf[..n].to_vec()).is_none()
                                    {
                                        eng.stats_inc_overflow();
                                    }
                                    drop(g);
                                    q.cv.notify_one();
                                }
                            }
                        }
                        Ok(_) => {} // packet from a non-upstream source: ignore
                        Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => {}
                    }
                }
                eng
            })
            .map_err(|e| e.to_string())?
    };

    // ---- s2c pump: queue -> active client ----
    let s2c_pump = {
        let q = s2c_q.clone();
        let fe = frontend.try_clone().map_err(|e| e.to_string())?;
        let stop = stop.clone();
        let start = start.clone();
        let clients = active_client.clone();
        std::thread::Builder::new()
            .name("s2c-pump".into())
            .spawn(move || {
                let mut acct = SendAccountant::default();
                loop {
                    if stop.load(Ordering::Relaxed) {
                        let due = { q.q.lock().unwrap().pop_due(f64::INFINITY) };
                        flush_to_client(&fe, &clients, &due, &mut acct, f64::INFINITY);
                        break;
                    }
                    let now = now_ms(&start);
                    let batch = {
                        let mut g = q.q.lock().unwrap();
                        if g.is_empty() {
                            drop(g);
                            let g2 = q.q.lock().unwrap();
                            let _ = q.cv.wait_timeout(g2, Duration::from_millis(20)).unwrap();
                            Vec::new()
                        } else {
                            let batch = g.pop_due(now);
                            if batch.is_empty() {
                                drop(g);
                                std::thread::sleep(Duration::from_micros(300));
                            }
                            batch
                        }
                    };
                    if !batch.is_empty() {
                        flush_to_client(&fe, &clients, &batch, &mut acct, now);
                    }
                }
                acct
            })
            .map_err(|e| e.to_string())?
    };

    if let Some(pf) = &cfg.port_file {
        std::fs::write(pf, format!("{local}\n")).map_err(|e| format!("port file: {e}"))?;
    }
    if cfg.verbose {
        eprintln!("[impair-proxy] listening on {local} -> {upstream}");
    }

    // ---- supervise: duration / stop flag ----
    let started_unix = unix_ms_now();
    if cfg.duration_ms > 0 {
        let deadline = Instant::now() + Duration::from_millis(cfg.duration_ms);
        while Instant::now() < deadline && !STOP_REQUESTED.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
    } else {
        while !STOP_REQUESTED.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    stop.store(true, Ordering::Relaxed);
    // give threads a moment to observe and flush
    std::thread::sleep(Duration::from_millis(250));

    let c2s_eng = c2s_reader.join().map_err(|_| "c2s reader panicked")?;
    let c2s_acct = c2s_pump.join().map_err(|_| "c2s pump panicked")?;
    let s2c_eng = s2c_reader.join().map_err(|_| "s2c reader panicked")?;
    let s2c_acct = s2c_pump.join().map_err(|_| "s2c pump panicked")?;

    let mut c2s_stats = c2s_eng.stats;
    c2s_acct.merge_into(&mut c2s_stats);
    let mut s2c_stats = s2c_eng.stats;
    s2c_acct.merge_into(&mut s2c_stats);

    let ended = unix_ms_now();
    Ok(StatsDoc {
        mode: cfg.mode.as_str().to_string(),
        seed: cfg.seed,
        started_unix_ms: started_unix,
        ended_unix_ms: ended,
        duration_ms: ended - started_unix,
        c2s: c2s_stats,
        s2c: s2c_stats,
    })
}

fn send_batch(
    up: &UpstreamSock,
    upstream: SocketAddr,
    batch: &[crate::impair::Queued],
    acct: &mut SendAccountant,
    now: f64,
    next_rebind_ms: &mut f64,
    nat_every_ms: u64,
) {
    if batch.is_empty() {
        return;
    }
    let do_rebind = nat_every_ms > 0 && now >= *next_rebind_ms;
    if do_rebind {
        match UdpSocket::bind("127.0.0.1:0").and_then(|s| {
            s.set_read_timeout(Some(Duration::from_millis(50)))?;
            Ok(s)
        }) {
            Ok(s) => {
                *up.sock.lock().unwrap() = s;
                up.genctr.fetch_add(1, Ordering::Release);
                acct.nat_rebinds += 1;
                *next_rebind_ms += nat_every_ms as f64;
            }
            Err(_) => {}
        }
    }
    let sock = up.sock.lock().unwrap();
    for p in batch {
        let _ = sock.send_to(&p.data, upstream);
        acct.on_send(p.data.len(), now - p.arrival_ms);
    }
}

fn flush_to_client(
    fe: &UdpSocket,
    clients: &Mutex<Option<SocketAddr>>,
    batch: &[crate::impair::Queued],
    acct: &mut SendAccountant,
    now: f64,
) {
    let Some(dst) = *clients.lock().unwrap() else { return };
    for p in batch {
        let _ = fe.send_to(&p.data, dst);
        acct.on_send(p.data.len(), now - p.arrival_ms);
    }
}

/// Global stop flag set by SIGINT/SIGTERM (installed from main.rs).
pub static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

// keep HashMap import used even if single-client path changes later
#[allow(dead_code)]
type ClientMap = HashMap<SocketAddr, u64>;
