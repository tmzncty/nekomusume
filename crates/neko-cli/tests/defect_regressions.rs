//! Defect regressions from the 2026-10-07/08 limit campaigns:
//! #1 UDP probe echo wait and #3 periodic payload ceiling.
//!
//! #1: the UDP probe client's per-exchange echo wait historically used a
//! socket read timeout equal to the WHOLE session duration, so one lost
//! echo (single-sided 100% loss) parked the client for the entire
//! --duration before the "echo timeout" failure path ever ran — the
//! semantic existed but was unreachable in practice. The red test pins the
//! fixed shape: under 100% server-to-client loss via tools/impair-proxy,
//! the client exits within the negotiated deadline window (a small bound
//! past the per-exchange timeout), never parking for the whole duration.
//!
//! #3: the periodic CLI validated --bytes against the raw datagram ceiling
//! (1200) instead of the sealed frame ceiling (1170 = 1200 - the 30-byte
//! process-data header), so bytes=1171..1200 passed validation and then
//! core-dumped the client (rc=134) at the first seal_unreliable unwrap.
//! The red tests pin: the CLI rejects bytes above the true ceiling in both
//! default and measurement modes with a message carrying the real number,
//! the boundary value itself still runs (config-level accept), and the
//! capabilities report stops claiming 1200.
//!
//! Loopback-only; spawns the real binary; 100%-loss impairment comes from
//! tools/impair-proxy (deterministic, loopback, no privileges).
use std::net::{TcpListener, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");
const PROXY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/impair-proxy/target/release/impair-proxy"
);

static PORT_LOCK: Mutex<()> = Mutex::new(());

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neko-defect-{name}-{}", std::process::id()))
}

fn keygen(path: &Path) -> String {
    let out = Command::new(BIN)
        .args(["keygen", "--identity", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .strip_prefix("client_public_key=")
        .unwrap()
        .to_string()
}

fn lease_port() -> u16 {
    for port in 40080..=40100 {
        if let (Ok(tcp), Ok(udp)) = (
            TcpListener::bind(("127.0.0.1", port)),
            UdpSocket::bind(("127.0.0.1", port)),
        ) {
            drop(tcp);
            drop(udp);
            return port;
        }
    }
    panic!("no test port is locally available");
}

/// Kill+reap guard (established pattern).
struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn(args: &[String], measurement: bool) -> KillOnDrop {
    let mut cmd = Command::new(BIN);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if measurement {
        cmd.env("NEKO_MEASUREMENT", "1");
    }
    KillOnDrop(cmd.spawn().unwrap())
}

/// #1 red→green: 100% s2c loss via impair-proxy; the client must exit within
/// the deadline window (echo timeout path), never park for the full duration.
#[test]
fn udp_echo_loss_exits_within_deadline_window() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let front = lease_port();
    let upstream = lease_port();
    let server_identity = tmp("udp-loss-server");
    let client_identity = tmp("udp-loss-client");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);

    // impairment: client -> (front) proxy -> (upstream) server; drop 100%
    // of the server->client direction so the first echo never arrives.
    let proxy = KillOnDrop(
        Command::new(PROXY)
            .args([
                "--mode",
                "udp",
                "--bind",
                &format!("127.0.0.1:{front}"),
                "--upstream",
                &format!("127.0.0.1:{upstream}"),
                "--s2c-loss",
                "1.0",
                "--seed",
                "11",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("impair-proxy binary must exist (build tools/impair-proxy first)"),
    );
    std::thread::sleep(Duration::from_millis(500));

    let server = spawn(
        &[
            "server".to_string(),
            "--transport".into(),
            "udp".into(),
            "--port".into(),
            upstream.to_string(),
            "--bind".into(),
            format!("127.0.0.1:{upstream}"),
            "--identity".into(),
            server_identity.to_string_lossy().into_owned(),
            "--client-key".into(),
            client_key,
            "--duration".into(),
            "30".into(),
        ][..],
        true,
    );
    // wait for readiness by polling the port
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(sock) = UdpSocket::bind("127.0.0.1:0") {
            let _ = sock.send_to(b"x", format!("127.0.0.1:{upstream}"));
        }
        // presence probe: the neko server never answers junk, so readiness is
        // asserted indirectly below by the client completing the handshake
        // through the lossy proxy (c2s is intact).
        if Instant::now() > deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // client: 3 exchanges, duration 20s. The first echo is lost (s2c 100%).
    // Before the fix the client parks for the full 20s inside ONE recv whose
    // read timeout equals the whole duration; after the fix the per-exchange
    // wait is bounded and the client exits near the exchange deadline.
    let client_args = [
        "client".to_string(),
        "--transport".into(),
        "udp".into(),
        "--addr".into(),
        format!("127.0.0.1:{front}"),
        "--port".into(),
        upstream.to_string(),
        "--identity".into(),
        client_identity.to_string_lossy().into_owned(),
        "--server-key".into(),
        server_key,
        "--bytes".into(),
        "32".into(),
        "--count".into(),
        "3".into(),
        "--duration".into(),
        "20".into(),
    ];
    let t0 = Instant::now();
    let mut client = spawn(&client_args[..], true);
    // bounded wait for exit; 12s < the 20s duration is the red line: the old
    // behavior cannot exit before 20s (one recv timeout = whole duration).
    let status = loop {
        match client.0.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if t0.elapsed() > Duration::from_secs(12) => {
                panic!(
                    "client still running after 12s under 100% echo loss — \
                     parked until the whole --duration again (#1 regression)"
                );
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => panic!("try_wait failed: {e}"),
        }
    };
    assert!(
        !status.success(),
        "loss-path client must fail, got {status:?}"
    );
    let elapsed = t0.elapsed();
    assert!(
        elapsed < Duration::from_secs(12),
        "client exited but only after {elapsed:?} — parked near the full duration"
    );

    let _ = std::fs::remove_file(&server_identity);
    let _ = std::fs::remove_file(&client_identity);
    drop(server);
    drop(proxy);
}

/// #3 red→green: the CLI must reject --bytes above the TRUE sealed-frame
/// ceiling (1170), in both default and measurement modes, with the real
/// number in the message; the boundary value itself still passes validation
/// (observable as a later, different failure with no server listening).
#[test]
fn periodic_bytes_validation_uses_the_sealed_frame_ceiling() {
    // default mode
    for (bytes, expect_reject) in [(1169, false), (1170, false), (1171, true), (1200, true)] {
        let out = Command::new(BIN)
            .args([
                "periodic-client",
                "--bytes",
                &bytes.to_string(),
                "--count",
                "2",
                "--addr",
                "127.0.0.1:40080",
                "--identity",
                "/tmp/neko-defect-none.id",
                "--server-key",
                "ababababababababababababababababababababababababababababababababab",
            ])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        if expect_reject {
            assert_eq!(
                out.status.code(),
                Some(2),
                "bytes={bytes} must be rejected by CLI validation, stderr={stderr:?}"
            );
            assert!(
                stderr.contains("1170"),
                "rejection must carry the true ceiling, got {stderr:?}"
            );
        } else {
            // accepted by validation: the rejection naming the payload
            // window must NOT appear; a later-stage failure (key/identity/
            // connect) is the accept oracle
            assert!(
                !stderr.contains("bytes outside"),
                "bytes={bytes} must pass validation, got {stderr:?}"
            );
        }
    }
    // measurement mode keeps the same sealed-frame ceiling
    let out = Command::new(BIN)
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "periodic-client",
            "--bytes",
            "1171",
            "--count",
            "2",
            "--addr",
            "127.0.0.1:40080",
            "--identity",
            "/tmp/neko-defect-none.id",
            "--server-key",
            "ababababababababababababababababababababababababababababababababab",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("1170"),
        "measurement mode must use the same ceiling"
    );
}

/// #3 red→green: the capabilities report must stop claiming the raw 1200
/// datagram ceiling for periodic payloads and state the true 1170.
#[test]
fn capabilities_report_the_true_periodic_bytes_ceiling() {
    let out = Command::new(BIN)
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"periodic_bytes_max\":1170"),
        "capabilities must state the true sealed-frame ceiling, got {stdout}"
    );
    // the raw datagram ceiling stays (it is the UDP probe payload bound)
    assert!(stdout.contains("\"bytes_max\":1200"), "{stdout}");
}
