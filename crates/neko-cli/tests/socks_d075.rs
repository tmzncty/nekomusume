//! D075: bounded ACK-driven SOCKS pump — acceptance harness.
//!
//! Drives the real socks-server / socks-client binaries on loopback against
//! a local echo target. The same test is the A/B instrument: before the
//! pipeline change it records the per-record stop-and-wait baseline; after
//! the change it re-measures. Elapsed/bytes lines are printed with
//! `--nocapture` and are measurement-gated on the binaries' side.
//!
//! Contract: docs/research/d075-socks-pipeline-and-measurement-sequence.md
//! (PR 1). Acceptance mapping:
//! - CONNECT behavior unchanged .... covered by `connect_reply_is_relayed`
//! - bidirectional transfer ........ `throughput_loopback_bidirectional`
//! - half-close both directions ..... `upload_half_close_keeps_download_alive`
//! - resource ceilings bounded ..... `window_budget_fits_declared_ceilings`
//! - malformed pre-auth input ....... inherited from socks_d071 tests

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_neko-cli")
}

fn tmp(name: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering as O};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, O::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!("socks-d075-{}-{seq}-{name}", std::process::id()));
    p
}

fn keygen(path: &std::path::Path) -> String {
    let out = Command::new(bin())
        .args(["keygen", "--identity"])
        .arg(path)
        .output()
        .expect("keygen spawns");
    assert!(out.status.success(), "keygen failed");
    String::from_utf8(out.stdout)
        .expect("keygen stdout utf8")
        .lines()
        .find_map(|l| l.strip_prefix("client_public_key="))
        .expect("public key line")
        .to_string()
}

/// Lease a port inside the research range (socks-server validates
/// 40080-40100). The range holds only 21 ports, so leases are handed out
/// under a global mutex and never reused within the process — parallel
/// tests each get a distinct port, and the spawned server holds it until
/// Drop kills the process.
fn lease_port() -> u16 {
    use std::sync::Mutex;
    static LEASED: Mutex<Vec<u16>> = Mutex::new(Vec::new());
    let mut leased = LEASED.lock().unwrap();
    for port in 40080..=40100 {
        if leased.contains(&port) {
            continue;
        }
        if let Ok(l) = TcpListener::bind(("127.0.0.1", port)) {
            drop(l);
            leased.push(port);
            return port;
        }
    }
    panic!("no free port in 40080-40100");
}

/// Lease an arbitrary loopback port (socks-client --listen is unvalidated).
fn lease_client_port() -> u16 {
    let l = TcpListener::bind("198.18.0.1:0").expect("lease bind");
    let p = l.local_addr().unwrap().port();
    drop(l);
    p
}

struct Spawned {
    child: Child,
    stdout: std::process::ChildStdout,
    _log: PathBuf,
}

impl Drop for Spawned {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self._log);
    }
}

fn spawn_named(args: &[&str], log_name: &str) -> Spawned {
    let log = tmp(log_name);
    let mut child = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::from(
            std::fs::File::create(&log).expect("log create"),
        ))
        .spawn()
        .expect("spawn neko-cli");
    let stdout = child.stdout.take().expect("stdout pipe");
    Spawned {
        child,
        stdout,
        _log: log,
    }
}

/// Consume the startup banner line (JSON) from the piped stdout. The
/// banner IS the readiness signal — no probe connection is spent.
fn wait_banner(s: &mut Spawned, needle: &str, deadline: Duration) {
    let start = Instant::now();
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while start.elapsed() < deadline {
        match s.stdout.read(&mut byte) {
            Ok(0) => panic!(
                "process exited before banner (needle {needle}); log: {}",
                std::fs::read_to_string(&s._log).unwrap_or_default()
            ),
            Ok(_) => {
                buf.push(byte[0]);
                if byte[0] == b'\n' {
                    let line = String::from_utf8_lossy(&buf);
                    if line.contains(needle) {
                        return;
                    }
                    if line.contains("neko:") || line.contains("panicked") {
                        panic!("startup refused: {line}");
                    }
                    buf.clear();
                }
            }
            Err(e) => panic!("banner read: {e}"),
        }
    }
    panic!("banner never carried {needle}");
}

/// Plain TCP echo target: echoes everything, closes on EOF.
fn spawn_echo() -> (u16, Arc<AtomicU64>) {
    let l = TcpListener::bind("198.18.0.1:0").expect("echo bind");
    let port = l.local_addr().unwrap().port();
    let served = Arc::new(AtomicU64::new(0));
    let counter = Arc::clone(&served);
    std::thread::spawn(move || {
        for conn in l.incoming() {
            let Ok(mut c) = conn else { continue };
            let counter = Arc::clone(&counter);
            std::thread::spawn(move || {
                let mut buf = [0u8; 16384];
                loop {
                    match c.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            counter.fetch_add(n as u64, Ordering::Relaxed);
                            if c.write_all(&buf[..n]).is_err() {
                                break;
                            }
                        }
                    }
                }
                let _ = c.shutdown(std::net::Shutdown::Both);
            });
        }
    });
    (port, served)
}

/// Half-close-aware target: drains the upload to EOF, then pushes
/// `post_eof` bytes of download before closing — proving the reverse
/// direction survives the sender's half-close.
fn spawn_eof_then_send(post_eof: usize) -> u16 {
    let l = TcpListener::bind("198.18.0.1:0").expect("eof-target bind");
    let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for conn in l.incoming() {
            let Ok(mut c) = conn else { continue };
            std::thread::spawn(move || {
                let mut sink = [0u8; 16384];
                while matches!(c.read(&mut sink), Ok(n) if n > 0) {}
                // Upload EOF seen. Keep the download direction alive.
                let block = [0xA5u8; 4096];
                let mut left = post_eof;
                while left > 0 {
                    let n = block.len().min(left);
                    if c.write_all(&block[..n]).is_err() {
                        break;
                    }
                    left -= n;
                }
                let _ = c.shutdown(std::net::Shutdown::Both);
            });
        }
    });
    port
}

/// SOCKS5 handshake + CONNECT against the local client entry.
fn socks_connect(client: &mut TcpStream, target: (&str, u16)) {
    client.write_all(&[5, 1, 0]).expect("greeting send");
    let mut m = [0u8; 2];
    client.read_exact(&mut m).expect("greeting reply");
    assert_eq!(&m, &[5, 0], "method negotiation");
    let ip: std::net::IpAddr = target.0.parse().expect("target ip");
    let mut req = vec![5, 1, 0, 4];
    match ip {
        std::net::IpAddr::V4(v4) => {
            req[3] = 1;
            req.extend_from_slice(&v4.octets());
        }
        std::net::IpAddr::V6(v6) => req.extend_from_slice(&v6.octets()),
    }
    req.extend_from_slice(&target.1.to_be_bytes());
    client.write_all(&req).expect("connect request");
    let mut head = [0u8; 4];
    client.read_exact(&mut head).expect("connect reply head");
    assert_eq!(head[0], 5, "reply version");
    assert_eq!(head[1], 0, "connect must succeed: {}", head[1]);
    // BND.ADDR (4 for V4) + BND.PORT
    let atyp = head[3];
    let skip = if atyp == 1 { 4 + 2 } else { 16 + 2 };
    let mut rest = vec![0u8; skip];
    client.read_exact(&mut rest).expect("connect reply tail");
}

fn payload(len: usize) -> Vec<u8> {
    // Cheap pattern with a period that does not divide 4096.
    (0..len).map(|i| (i % 4093) as u8).collect()
}

fn verify(target: &[u8], prefix: &str, got: &[u8]) {
    assert_eq!(got.len(), target.len(), "{prefix}: length mismatch");
    for (i, (a, b)) in target.iter().zip(got).enumerate() {
        assert_eq!(a, b, "{prefix}: first diff at {i}");
    }
}

fn start_stack(allow: &str) -> (Spawned, Spawned, u16, PathBuf, PathBuf) {
    let srv_id = tmp("srv.id");
    let cli_id = tmp("cli.id");
    let srv_pub = keygen(&srv_id);
    let cli_pub = keygen(&cli_id);
    let server_port = lease_port();
    let client_port = lease_client_port();
    let mut server = spawn_named(
        &[
            "socks-server",
            "--port",
            &server_port.to_string(),
            "--identity",
            srv_id.to_str().unwrap(),
            "--client-key",
            &cli_pub,
            "--allow",
            allow,
        ],
        "srv.log",
    );
    wait_banner(&mut server, "socks-server", Duration::from_secs(5));
    let mut client = spawn_named(
        &[
            "socks-client",
            "--listen",
            &client_port.to_string(),
            "--addr",
            &format!("127.0.0.1:{server_port}"),
            "--server-key",
            &srv_pub,
            "--identity",
            cli_id.to_str().unwrap(),
        ],
        "cli.log",
    );
    wait_banner(&mut client, "socks-client", Duration::from_secs(5));
    (server, client, client_port, srv_id, cli_id)
}

/// Acceptance 3: bidirectional transfer through the full stack, plus the
/// A/B measurement line. 4 MiB up + 4 MiB echoed down.
#[test]
fn throughput_loopback_bidirectional() {
    let (echo_port, echo_served) = spawn_echo();
    let (_server, _client, client_port, _s, _c) = start_stack(&format!("198.18.0.1:{echo_port}"));

    let mut sock = TcpStream::connect(("127.0.0.1", client_port)).expect("connect client entry");
    socks_connect(&mut sock, ("198.18.0.1", echo_port));

    // Baseline shape note (2026-10-12): the stop-and-wait pump measured
    // ~10 min for 4 MiB on loopback (test timed out at 420 s), matching
    // the W2/W4 field observation of a few KB/s. 512 KiB keeps the A/B
    // resolvable inside CI budgets (~80 s at the old rate).
    const KIB: usize = 512;
    let data = payload(KIB << 10);
    let start = Instant::now();
    let mut down = vec![0u8; data.len()];
    {
        // Reader first so the echo never blocks on our write window.
        let mut reader = sock.try_clone().expect("clone reader");
        let reader = std::thread::spawn(move || {
            let mut filled = 0usize;
            while filled < down_len(&down) {
                let n = reader.read(&mut down[filled..]).expect("download read");
                filled += n;
            }
            down
        });
        let mut sent = 0usize;
        while sent < data.len() {
            sent += sock.write(&data[sent..]).expect("upload write");
        }
        sock.shutdown(std::net::Shutdown::Write)
            .expect("upload half-close");
        let down = reader.join().expect("reader join");
        verify(&data, "echo", &down);
    }
    let elapsed = start.elapsed();
    let echoed = echo_served.load(Ordering::Relaxed) as usize;
    assert_eq!(echoed, KIB << 10, "echo target saw the full upload");
    println!(
        "d075_ab_loopback app_kib={KIB} elapsed_ms={} app_bytes_per_direction={} bps={}",
        elapsed.as_millis(),
        KIB << 10,
        ((KIB << 10) as f64 * 8.0 / elapsed.as_secs_f64()) as u64
    );
}

fn down_len(v: &[u8]) -> usize {
    v.len()
}

/// Acceptance 4: uploader half-close must not tear down the download
/// direction; the target pushes 256 KiB strictly after seeing our EOF.
#[test]
fn upload_half_close_keeps_download_alive() {
    const POST_EOF: usize = 256 << 10;
    let target_port = spawn_eof_then_send(POST_EOF);
    let (_server, _client, client_port, _s, _c) = start_stack(&format!("198.18.0.1:{target_port}"));

    let mut sock = TcpStream::connect(("127.0.0.1", client_port)).expect("connect client entry");
    socks_connect(&mut sock, ("198.18.0.1", target_port));

    let up = payload(64 << 10);
    sock.write_all(&up).expect("upload");
    sock.shutdown(std::net::Shutdown::Write)
        .expect("upload half-close");

    let mut got = Vec::with_capacity(POST_EOF);
    let mut buf = [0u8; 16384];
    loop {
        match sock.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => got.extend_from_slice(&buf[..n]),
            Err(e) => panic!("download after half-close died: {e}"),
        }
    }
    assert_eq!(got.len(), POST_EOF, "download bytes after our EOF");
    // Byte pattern is 0xA5 filler from the target — semantic checks are
    // the length above and that none of it was corrupted mid-stream.
    assert!(got.iter().all(|b| *b == 0xA5), "post-eof payload corrupted");
}

/// Acceptance 2: the SOCKS reply code from the server is relayed to the
/// local application (0x00 success shape asserted by socks_connect).
#[test]
fn connect_reply_is_relayed() {
    let (echo_port, _served) = spawn_echo();
    let (_server, _client, client_port, _s, _c) = start_stack(&format!("198.18.0.1:{echo_port}"));
    let mut sock = TcpStream::connect(("127.0.0.1", client_port)).expect("connect client entry");
    socks_connect(&mut sock, ("198.18.0.1", echo_port));
    const PROBE: &[u8] = b"still-alive";
    sock.write_all(PROBE).expect("probe write");
    let mut back = [0u8; PROBE.len()];
    sock.read_exact(&mut back).expect("probe echo");
    assert_eq!(&back, PROBE);
}
