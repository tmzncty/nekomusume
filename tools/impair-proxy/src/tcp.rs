//! TCP runtime: accept per client, one upstream connection each, per-conn
//! direction pair (reader+pump) with the same engine semantics as UDP.
//!
//! TCP applicability matrix (documented in README):
//! - delay/jitter/reorder/duplicate: applied per-read chunk as extra due-ms.
//! - rate: token bucket defers release (backpressure, never drops).
//! - MTU: chunked writes of at most mtu bytes (PMTU ceiling), never drops.
//! - blackout window: stall — parking reads and writes for the window (no
//!   data loss, mimics a link gone silent).
//! - kill rst: immediate socket teardown with RST semantics (SO_LINGER 0).
//! - NAT rebinding is UDP-only (TCP rebinding IS teardown; use --c2s-kill).

use crate::config::{Config, KillStyle};
use crate::impair::{direction_seed, ArrivalEngine, Decision, ImpairQueue, SendAccountant};
use crate::stats::StatsDoc;
use crate::udp::STOP_REQUESTED;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const CHUNK: usize = 65536;

struct SharedQueue {
    q: Mutex<ImpairQueue>,
    cv: Condvar,
}

fn now_ms(start: &Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

// Force an RST on next drop: SO_LINGER with 0 timeout. `set_linger` is
// still unstable, so call setsockopt directly (same technique as the
// signal(2) declaration in main.rs — no libc crate).
unsafe extern "C" {
    fn setsockopt(
        fd: i32,
        level: i32,
        optname: i32,
        optval: *const u8,
        optlen: u32,
    ) -> i32;
}

fn set_rst_on_close(s: &TcpStream) {
    use std::os::fd::AsRawFd;
    // struct linger { int l_onoff; int l_linger; } — Linux x86_64
    const SOL_SOCKET: i32 = 1;
    const SO_LINGER: i32 = 13;
    let linger: [i32; 2] = [1, 0]; // on, 0s => RST
    let rc = unsafe {
        setsockopt(
            s.as_raw_fd(),
            SOL_SOCKET,
            SO_LINGER,
            linger.as_ptr().cast::<u8>(),
            std::mem::size_of::<[i32; 2]>() as u32,
        )
    };
    let _ = rc;
}

pub fn run_tcp(cfg: &Config) -> Result<StatsDoc, String> {
    let upstream: SocketAddr = cfg
        .upstream
        .ok_or_else(|| "TCP mode requires --upstream HOST:PORT".to_string())?;

    let listener = TcpListener::bind(cfg.bind.unwrap_or_else(|| "127.0.0.1:0".parse().unwrap()))
        .map_err(|e| format!("bind listener: {e}"))?;
    let local = listener.local_addr().map_err(|e| e.to_string())?;

    let start = Arc::new(Instant::now());
    let c2s_stats_shared: Arc<Mutex<Vec<crate::stats::DirectionStats>>> =
        Arc::new(Mutex::new(Vec::new()));
    let s2c_stats_shared = Arc::new(Mutex::new(Vec::new()));

    if let Some(pf) = &cfg.port_file {
        std::fs::write(pf, format!("{local}\n")).map_err(|e| format!("port file: {e}"))?;
    }
    if cfg.verbose {
        eprintln!("[impair-proxy] tcp listening on {local} -> {upstream}");
    }

    let started_unix = crate::stats::unix_ms_now();

    // deadline for accepting loop
    let deadline = if cfg.duration_ms > 0 {
        Some(Instant::now() + Duration::from_millis(cfg.duration_ms))
    } else {
        None
    };
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("listener nonblocking: {e}"))?;

    while !STOP_REQUESTED.load(Ordering::Relaxed) {
        if let Some(d) = deadline {
            if Instant::now() >= d {
                break;
            }
        }
        match listener.accept() {
            Ok((client, _addr)) => {
                client.set_nonblocking(false).ok();
                client.set_nodelay(true).ok();
                let cfg_c = cfg.clone();
                let start_c = start.clone();
                let c2s_sh = c2s_stats_shared.clone();
                let s2c_sh = s2c_stats_shared.clone();
                std::thread::Builder::new()
                    .name("tcp-conn".into())
                    .spawn(move || handle_conn(client, upstream, &cfg_c, start_c, c2s_sh, s2c_sh))
                    .map_err(|e| format!("spawn conn thread: {e}"))?;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }

    // Connection threads watch STOP_REQUESTED too and exit within ~100ms.
    // Give them a moment to finish and record stats.
    std::thread::sleep(Duration::from_millis(400));

    let mut c2s_all = c2s_stats_shared.lock().unwrap().clone();
    let mut s2c_all = s2c_stats_shared.lock().unwrap().clone();

    let c2s = if c2s_all.is_empty() {
        crate::stats::DirectionStats::default()
    } else {
        let mut base = c2s_all.remove(0);
        for s in c2s_all {
            merge_stats(&mut base, s);
        }
        base
    };
    let s2c = if s2c_all.is_empty() {
        crate::stats::DirectionStats::default()
    } else {
        let mut base = s2c_all.remove(0);
        for s in s2c_all {
            merge_stats(&mut base, s);
        }
        base
    };

    let ended = crate::stats::unix_ms_now();
    Ok(StatsDoc {
        mode: cfg.mode.as_str().to_string(),
        seed: cfg.seed,
        started_unix_ms: started_unix,
        ended_unix_ms: ended,
        duration_ms: ended - started_unix,
        c2s,
        s2c,
    })
}

fn merge_stats(base: &mut crate::stats::DirectionStats, s: crate::stats::DirectionStats) {
    base.received_pkts += s.received_pkts;
    base.received_bytes += s.received_bytes;
    base.sent_pkts += s.sent_pkts;
    base.sent_bytes += s.sent_bytes;
    base.dropped_random += s.dropped_random;
    base.dropped_ge += s.dropped_ge;
    base.dropped_mtu += s.dropped_mtu;
    base.dropped_blackout += s.dropped_blackout;
    base.dropped_overflow += s.dropped_overflow;
    base.duplicates += s.duplicates;
    base.reorder_hits += s.reorder_hits;
    base.rate_deferred += s.rate_deferred;
    base.nat_rebinds += s.nat_rebinds;
    base.tcp_rst_kills += s.tcp_rst_kills;
    base.latency_count += s.latency_count;
    base.latency_sum_ms += s.latency_sum_ms;
    if s.latency_count > 0 {
        if base.latency_count == s.latency_count || base.latency_min_ms == 0.0 {
            base.latency_min_ms = if base.latency_min_ms == 0.0 {
                s.latency_min_ms
            } else {
                base.latency_min_ms.min(s.latency_min_ms)
            };
        }
        base.latency_max_ms = base.latency_max_ms.max(s.latency_max_ms);
    }
    for i in 0..crate::stats::N_BUCKETS {
        base.hist[i] += s.hist[i];
    }
}

/// One accepted client: two direction pipelines c2s / s2c.
fn handle_conn(
    client: TcpStream,
    upstream: SocketAddr,
    cfg: &Config,
    start: Arc<Instant>,
    c2s_out: Arc<Mutex<Vec<crate::stats::DirectionStats>>>,
    s2c_out: Arc<Mutex<Vec<crate::stats::DirectionStats>>>,
) {
    let up_conn = match TcpStream::connect_timeout(
        &upstream,
        Duration::from_millis(2000),
    ) {
        Ok(s) => s,
        Err(_) => {
            set_rst_on_close(&client);
            return;
        }
    };
    up_conn.set_nodelay(true).ok();
    let up_read = match up_conn.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };

    let c2s_q = Arc::new(SharedQueue { q: Mutex::new(ImpairQueue::new(cfg.queue_cap)), cv: Condvar::new() });
    let s2c_q = Arc::new(SharedQueue { q: Mutex::new(ImpairQueue::new(cfg.queue_cap)), cv: Condvar::new() });

    // Tear-down signaling between the two direction pipelines of this conn.
    let kill = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // c2s: read client -> engine -> queue -> write upstream
    let c2s_reader = {
        let client_r = match client.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let q = c2s_q.clone();
        let kill = kill.clone();
        let start = start.clone();
        let cfg_c = cfg.c2s.clone();
        let seed = direction_seed(cfg.seed, 0xC25);
        std::thread::Builder::new()
            .name("c2s-read".into())
            .spawn(move || {
                let mut eng = ArrivalEngine::new(cfg_c.clone(), seed);
                let mut buf = vec![0u8; CHUNK];
                let mut client_r = client_r;
                // Wake up regularly so the kill flag is observed even when
                // the client is silent (else join deadlocks, no RST, ever).
                client_r.set_read_timeout(Some(Duration::from_millis(50))).ok();
                loop {
                    if kill.load(Ordering::Relaxed)
                        || STOP_REQUESTED.load(Ordering::Relaxed)
                    {
                        break;
                    }
                    let now = now_ms(&start);
                    if eng.hold_now(now) {
                        // stall window: park briefly, do not read, do not drop
                        std::thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    match client_r.read(&mut buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let now = now_ms(&start);
                            // MTU ceiling: cap the write chunk size.
                            let eff = eng_mtu_cap(&cfg_c, n);
                            match eng.decide(now, eff) {
                                Decision::Drop(_) => {
                                    // MTU drop was already the UDP semantic;
                                    // for TCP, MTU is a chunk cap not a drop,
                                    // so Drop here can only be loss/ge/etc —
                                    // dropping data on a reliable stream is
                                    // wrong, so we forward anyway but count
                                    // the impairment decision in stats.
                                    push_all(&q, now, now, &buf[..n], &mut eng);
                                }
                                Decision::KillRst => {
                                    eng.stats.tcp_rst_kills += 1;
                                    set_rst_on_close(&client_r);
                                    kill.store(true, Ordering::Relaxed);
                                    break;
                                }
                                Decision::Enqueue { due_ms, extra_dup } => {
                                    push_due(&q, due_ms, now, &buf[..n], extra_dup, &mut eng);
                                }
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                eng
            })
            .ok()
    };

    // c2s pump: queue -> upstream write
    let c2s_pump = {
        let q = c2s_q.clone();
        let up_w = match up_conn.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let kill = kill.clone();
        let start = start.clone();
        let mtu = cfg.c2s.mtu;
        std::thread::Builder::new()
            .name("c2s-pump".into())
            .spawn(move || {
                let mut up_w = up_w;
                let mut acct = SendAccountant::default();
                loop {
                    // Exit on teardown OR global stop so stats are not lost
                    // (run_tcp joins this handle before collecting).
                    if kill.load(Ordering::Relaxed) || STOP_REQUESTED.load(Ordering::Relaxed) {
                        break;
                    }
                    let now = now_ms(&start);
                    let batch = {
                        let mut g = q.q.lock().unwrap();
                        if g.is_empty() {
                            let _ = q.cv.wait_timeout(g, Duration::from_millis(20)).unwrap();
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
                    for p in batch {
                        if STOP_REQUESTED.load(Ordering::Relaxed) {
                            break;
                        }
                        if write_all_mtu(&mut up_w, &p.data, mtu, now, &start).is_err() {
                            kill.store(true, Ordering::Relaxed);
                            break;
                        }
                        acct.on_send(p.data.len(), now - p.arrival_ms);
                    }
                }
                // on teardown, flush what's due
                let due = { q.q.lock().unwrap().pop_due(f64::INFINITY) };
                for p in due {
                    if write_all_mtu(&mut up_w, &p.data, mtu, now_ms(&start), &start).is_ok() {
                        acct.on_send(p.data.len(), 0.0);
                    }
                }
                acct
            })
            .ok()
    };

    // s2c: read upstream -> engine -> queue -> write client
    let s2c_reader = {
        let q = s2c_q.clone();
        let kill = kill.clone();
        let start = start.clone();
        let cfg_c = cfg.s2c.clone();
        let seed = direction_seed(cfg.seed, 0x52C);
        std::thread::Builder::new()
            .name("s2c-read".into())
            .spawn(move || {
                let mut eng = ArrivalEngine::new(cfg_c.clone(), seed);
                let mut buf = vec![0u8; CHUNK];
                let mut up_read = up_read;
                up_read.set_read_timeout(Some(Duration::from_millis(50))).ok();
                loop {
                    if kill.load(Ordering::Relaxed)
                        || STOP_REQUESTED.load(Ordering::Relaxed)
                    {
                        break;
                    }
                    let now = now_ms(&start);
                    if eng.hold_now(now) {
                        std::thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    match up_read.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let now = now_ms(&start);
                            let eff = eng_mtu_cap(&cfg_c, n);
                            match eng.decide(now, eff) {
                                Decision::Drop(_) => {
                                    push_all(&q, now, now, &buf[..n], &mut eng);
                                }
                                Decision::KillRst => {
                                    eng.stats.tcp_rst_kills += 1;
                                    // RST toward client: writer side will fail next write
                                    kill.store(true, Ordering::Relaxed);
                                    break;
                                }
                                Decision::Enqueue { due_ms, extra_dup } => {
                                    push_due(&q, due_ms, now, &buf[..n], extra_dup, &mut eng);
                                }
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                eng
            })
            .ok()
    };

    // s2c pump: queue -> client write
    let s2c_pump = {
        let q = s2c_q.clone();
        let client_w = match client.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };
        let kill = kill.clone();
        let start = start.clone();
        let mtu = cfg.s2c.mtu;
        std::thread::Builder::new()
            .name("s2c-pump".into())
            .spawn(move || {
            let mut client_w = client_w;
            let mut acct = SendAccountant::default();
            loop {
                if kill.load(Ordering::Relaxed) || STOP_REQUESTED.load(Ordering::Relaxed) {
                    break;
                }
                let now = now_ms(&start);
                let batch = {
                    let mut g = q.q.lock().unwrap();
                    if g.is_empty() {
                        let _ = q.cv.wait_timeout(g, Duration::from_millis(20)).unwrap();
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
                for p in batch {
                    if STOP_REQUESTED.load(Ordering::Relaxed) {
                        break;
                    }
                    if write_all_mtu(&mut client_w, &p.data, mtu, now, &start).is_err() {
                        kill.store(true, Ordering::Relaxed);
                        break;
                    }
                    acct.on_send(p.data.len(), now - p.arrival_ms);
                }
            }
            let due = { q.q.lock().unwrap().pop_due(f64::INFINITY) };
            for p in due {
                if write_all_mtu(&mut client_w, &p.data, mtu, now_ms(&start), &start).is_ok() {
                    acct.on_send(p.data.len(), 0.0);
                }
            }
            acct
        })
        .ok()
    };

    let c2s_eng = c2s_reader.and_then(|h| h.join().ok());
    let c2s_acct = c2s_pump.and_then(|h| h.join().ok());
    let s2c_eng = s2c_reader.and_then(|h| h.join().ok());
    let s2c_acct = s2c_pump.and_then(|h| h.join().ok());

    let mut c2s_stats = c2s_eng.map(|e| e.stats).unwrap_or_default();
    if let Some(a) = c2s_acct {
        a.merge_into(&mut c2s_stats);
    }
    let mut s2c_stats = s2c_eng.map(|e| e.stats).unwrap_or_default();
    if let Some(a) = s2c_acct {
        a.merge_into(&mut s2c_stats);
    }
    c2s_out.lock().unwrap().push(c2s_stats);
    s2c_out.lock().unwrap().push(s2c_stats);
}

fn eng_mtu_cap(cfg: &crate::config::DirectionConfig, n: usize) -> usize {
    match cfg.mtu {
        Some(m) if n > m => m, // report capped length to the engine; chunks below
        _ => n,
    }
}

fn push_all(
    q: &Arc<SharedQueue>,
    now: f64,
    due: f64,
    data: &[u8],
    eng: &mut ArrivalEngine,
) {
    let mut g = q.q.lock().unwrap();
    if g.push(due, now, 0, 0, data.to_vec()).is_none() {
        eng.stats_inc_overflow();
    }
}

fn push_due(
    q: &Arc<SharedQueue>,
    due_ms: f64,
    now: f64,
    data: &[u8],
    extra_dup: bool,
    eng: &mut ArrivalEngine,
) {
    let mut g = q.q.lock().unwrap();
    if g.push(due_ms, now,  0, 0, data.to_vec()).is_none() {
        eng.stats_inc_overflow();
    }
    if extra_dup && g.push(due_ms, now, 0, 1, data.to_vec()).is_none() {
        eng.stats_inc_overflow();
    }
    drop(g);
    q.cv.notify_one();
}

/// Write in mtu-capped chunks; the token-bucket due time is already enforced
/// upstream of this call. Blocks are normal TCP backpressure.
fn write_all_mtu(
    w: &mut TcpStream,
    data: &[u8],
    mtu: Option<usize>,
    _now: f64,
    _start: &Instant,
) -> std::io::Result<()> {
    if let Some(m) = mtu {
        for chunk in data.chunks(m) {
            w.write_all(chunk)?;
        }
        Ok(())
    } else {
        w.write_all(data)
    }
}

#[allow(dead_code)]
fn unused(_k: KillStyle) {}
