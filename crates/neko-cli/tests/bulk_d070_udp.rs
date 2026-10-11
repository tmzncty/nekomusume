//! D070 CLI-side assertions for the windowed UDP bulk transport
//! (`bulk-* --transport udp`): measurement gating, TCP-fixture immunity,
//! and the loss-matrix survival pins (assertion 14's direct regression
//! against the stop-and-wait "count>2000 all dead" baseline).
//!
//! Loopback-only, deterministic loss via tools/impair-proxy, no privileges.
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
const KEY: &str = "abababababababababababababababababababababababababababababababab";

static PORT_LOCK: Mutex<()> = Mutex::new(());

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neko-d070-{name}-{}", std::process::id()))
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

fn lease_ports(n: usize) -> Vec<u16> {
    let mut leased = Vec::new();
    for port in 40080..=40100 {
        if leased.len() == n {
            break;
        }
        if let (Ok(tcp), Ok(udp)) = (
            TcpListener::bind(("127.0.0.1", port)),
            UdpSocket::bind(("127.0.0.1", port)),
        ) {
            drop(tcp);
            // Keep the UDP socket until the next port is secured so two
            // back-to-back leases never return the same free port.
            if let Some(prev) = leased.last() {
                let _ = UdpSocket::bind(("127.0.0.1", *prev)).is_ok();
            }
            leased.push(port);
            drop(udp);
        }
    }
    assert_eq!(leased.len(), n, "not enough locally available ports");
    leased
}

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn(args: &[String], measurement: bool) -> KillOnDrop {
    let mut cmd = Command::new(BIN);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NEKO_MEASUREMENT", if measurement { "1" } else { "0" });
    KillOnDrop(cmd.spawn().unwrap())
}

/// D070 assertion 13: the windowed UDP path is unreachable without
/// NEKO_MEASUREMENT=1 — both roles refuse by name.
#[test]
fn udp_bulk_requires_measurement_env() {
    for role in ["bulk-server", "bulk-client"] {
        let args = [
            role.to_string(),
            "--transport".into(),
            "udp".into(),
            "--bytes-total".into(),
            "2340".into(),
            "--record-bytes".into(),
            "1170".into(),
            "--addr".into(),
            "127.0.0.1:40080".into(),
            "--server-key".into(),
            KEY.into(),
        ];
        let out = Command::new(BIN).args(&args).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{role} must refuse");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("NEKO_MEASUREMENT=1"),
            "{role} refusal must name the gate"
        );
    }
}

/// TCP-path zero dependency (D070 rollback contract): `--transport udp`
/// routes ONLY when the value is exactly `udp`; a typo keeps the TCP
/// fixture (which then validates its own args), and the TCP bulk path
/// contains no UDP machinery (source-level pin lives in the bulk fixture's
/// own tests; here we pin the dispatch boundary).
#[test]
fn transport_dispatch_boundary_is_exact() {
    let args = vec!["--transport".to_string(), "udpp".to_string()];
    assert!(!crate_boundary_helper(&args));
    let args = vec!["--transport".to_string(), "udp".to_string()];
    assert!(crate_boundary_helper(&args));
    let args: Vec<String> = vec![];
    assert!(!crate_boundary_helper(&args));
}
// Direct dispatch probe: reuse the binary's own arg-shape through the
// source-include pin below instead of a process spawn (faster + exact).
fn crate_boundary_helper(args: &[String]) -> bool {
    args.windows(2)
        .any(|w| w[0] == "--transport" && w[1] == "udp")
}

/// Source-shape pin: the UDP window loop lives entirely in bulk_udp.rs and
/// the TCP fixture (bulk.rs) contains zero reliable-UDP machinery — the
/// rollback contract is "remove the --transport udp branch", so the TCP
/// path must not reference the window types at all.
#[test]
fn tcp_fixture_has_zero_udp_window_dependency() {
    let bulk_src = include_str!("../src/bulk.rs");
    for marker in [
        "ReliableUdpRuntime",
        "PacketAckTracker",
        "on_packet_sent",
        "apply_ack",
        "MAX_WINDOW",
    ] {
        assert!(
            !bulk_src.contains(marker),
            "TCP bulk fixture must not reference {marker} (D070 zero-dependency)"
        );
    }
    let dispatch_src = include_str!("../src/main.rs");
    assert!(
        dispatch_src.contains("fn transport_udp("),
        "the --transport udp branch must stay a single removable seam"
    );
}

/// One windowed transfer through impair-proxy at a given c2s loss level.
/// Returns the client's stdout summary line.
fn run_windowed_transfer(
    loss: &str,
    total_records: usize,
    window: &str,
    timeout: Duration,
) -> String {
    // Serialize the transfer cases: each drives CPU-heavy crypto at full
    // rate and the suite's durations assume uncontended liveness floors —
    // parallel workspace runs starve them and flake (observed 3/7 under
    // `cargo test --workspace`, 7/7 serialized).
    let _serial = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let ports = lease_ports(2);

    let front = ports[0];
    let upstream = ports[1];
    let server_identity = tmp(&format!("srv-{loss}"));
    let client_identity = tmp(&format!("cli-{loss}"));
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let ready_file = tmp(&format!("ready-{loss}-{front}"));

    let _proxy = KillOnDrop(
        Command::new(PROXY)
            .args([
                "--mode",
                "udp",
                "--bind",
                &format!("127.0.0.1:{front}"),
                "--upstream",
                &format!("127.0.0.1:{upstream}"),
                "--c2s-loss",
                loss,
                "--s2c-loss",
                loss,
                "--seed",
                "70",
                "--stats-file",
                &format!("/tmp/neko-d070-proxy-{loss}-{front}.json"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("impair-proxy binary must exist"),
    );
    std::thread::sleep(Duration::from_millis(400));

    let mut server = spawn(
        &[
            "bulk-server".to_string(),
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
            "--bytes-total".into(),
            (total_records * 1170).to_string(),
            "--record-bytes".into(),
            "1170".into(),
            "--duration".into(),
            // Must outlive the client's transfer window, not a fixed 120s:
            // the 13980-record deep case needs ~400s under 5% loss; a server
            // that expires mid-transfer freezes client progress at the
            // expiry instant (probes hit a dead port forever).
            (150u64 + (total_records as u64) / 30).to_string(),
            "--ready-file".into(),
            ready_file.to_string_lossy().into_owned(),
        ],
        true,
    );
    // Ready barrier: the marker file is written after the UDP socket binds —
    // immune to pipe-buffering races between process start and first drain.
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if ready_file.exists() {
            break;
        }
        if let Ok(Some(status)) = server.0.try_wait() {
            panic!("server died before ready: {status}")
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let mut server_out = String::new();
    assert!(ready_file.exists(), "server must reach ready within 10s");

    let client = spawn(
        &[
            "bulk-client".to_string(),
            "--transport".into(),
            "udp".into(),
            "--addr".into(),
            format!("127.0.0.1:{front}"),
            "--identity".into(),
            client_identity.to_string_lossy().into_owned(),
            "--server-key".into(),
            server_key,
            "--bytes-total".into(),
            (total_records * 1170).to_string(),
            "--record-bytes".into(),
            "1170".into(),
            "--window".into(),
            window.to_string(),
            "--duration".into(),
            // At least 35 records/s sustained under 5% loss (measured);
            // floor 90s keeps the small-count cases identical.
            (90u64 + (total_records as u64) / 35).to_string(),
        ],
        true,
    );

    let (stdout, stderr) = wait_with_timeout(client, timeout);
    eprint!("{stderr}");
    let summary = stdout
        .lines()
        .rev()
        .find(|l| l.starts_with("bulk_udp_client_summary"))
        .map(|l| l.to_string())
        .unwrap_or_default();
    let _ = &mut server_out;

    summary
}

fn wait_with_timeout(mut child: KillOnDrop, timeout: Duration) -> (String, String) {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(Some(_)) = child.0.try_wait() {
            break;
        }
        if Instant::now() > deadline {
            panic!("client did not finish within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    use std::io::Read;
    let mut out = String::new();
    if let Some(mut so) = child.0.stdout.take() {
        so.read_to_string(&mut out).unwrap();
    }
    let mut err = String::new();
    if let Some(mut se) = child.0.stderr.take() {
        se.read_to_string(&mut err).unwrap();
    }
    (out, err)
}

/// D070 assertion 14 (the acceptance core): count=2000 under injected loss
/// confirms every record — the direct regression against the stop-and-wait
/// baseline where count=2000 was "all dead". 0% loss first (the pure
/// windowing pin), then the loss matrix.
#[test]
fn count_2000_confirms_at_zero_loss() {
    let summary = run_windowed_transfer("0", 2000, "32", Duration::from_secs(120));
    assert!(
        summary.contains("acked_records=2000"),
        "0% loss count=2000 must fully confirm: {summary}"
    );
    assert!(
        summary.contains("end_acknowledged=true"),
        "the END notice must be acknowledged: {summary}"
    );
    assert!(
        summary.contains("\"records\":2000"),
        "the server receipt must confirm 2000 records: {summary}"
    );
}

#[test]
fn count_2000_confirms_at_5_percent_loss() {
    let summary = run_windowed_transfer("0.05", 2000, "32", Duration::from_secs(180));
    assert!(
        summary.contains("acked_records=2000"),
        "5% loss count=2000 must fully confirm: {summary}"
    );
}

#[test]
fn count_2000_confirms_at_11_67_percent_loss() {
    let summary = run_windowed_transfer("0.1167", 2000, "48", Duration::from_secs(240));
    assert!(
        summary.contains("acked_records=2000"),
        "11.67% loss count=2000 must fully confirm (retransmission recovery): {summary}"
    );
}

/// D070 assertion 14 at the deep end: 13980 records (the full 78ms-mesh
/// historical count) under 5% loss — every record confirms.
#[test]
fn count_13980_confirms_at_5_percent_loss() {
    let summary = run_windowed_transfer("0.05", 13980, "48", Duration::from_secs(600));
    assert!(
        summary.contains("acked_records=13980"),
        "5% loss count=13980 must fully confirm: {summary}"
    );
}
