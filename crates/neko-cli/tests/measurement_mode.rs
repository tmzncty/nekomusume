//! Measurement-mode (`NEKO_MEASUREMENT=1`) limit pinning.
//!
//! Two contracts, both enforced by running the real binary (every rejection
//! below exits from inside the process, and environment inheritance makes
//! subprocess mode the only honest way to isolate the opt-in):
//!
//! 1. Default-off: the exact pre-measurement windows keep rejecting at their
//!    old edges — the relaxation is invisible without the variable.
//! 2. Opt-in: each relaxed ceiling accepts values above the default window,
//!    still rejects above the measurement ceiling, and every ceiling sits at
//!    the `neko_session` library hard limits (the research ceiling is the
//!    library floor; see `src/measurement.rs` for the derivations).
//!
//! Loopback-only; no VPS/WAN/sudo; no network behavior beyond the existing
//! loopback fixtures.
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};
use std::{fs, thread};

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

/// A syntactically valid 32-byte peer key, so later checks are reached.
const PEER: &str = "abababababababababababababababababababababababababababababababab";

/// The repository's port range (40080-40100) is shared across the whole test
/// binary; guard the lease + server lifetime with one mutex (established
/// preauth_rejection.rs pattern).
static PORT_LOCK: Mutex<()> = Mutex::new(());

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("neko-measurement-{name}-{}", std::process::id()))
}

fn argv(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

/// A client invocation built from exactly one occurrence of every flag;
/// `over` replaces rather than adds (the CLI reads the FIRST occurrence).
fn client_args(over: &[(&str, &str)]) -> Vec<String> {
    let mut pairs: Vec<(&str, &str)> = vec![
        ("--identity", "/tmp/neko-measurement-missing.id"),
        ("--transport", "tcp"),
        ("--port", "40080"),
        ("--addr", "127.0.0.1:9"),
        ("--server-key", PEER),
    ];
    for (k, v) in over {
        match pairs.iter_mut().find(|(pk, _)| pk == k) {
            Some(slot) => *slot = (k, v),
            None => pairs.push((k, v)),
        }
    }
    let mut out = vec!["client".to_string()];
    for (k, v) in pairs {
        out.push(k.to_string());
        out.push(v.to_string());
    }
    out
}

fn out_of(output: Output) -> (Option<i32>, String, String) {
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn run(args: &[String]) -> (Option<i32>, String, String) {
    out_of(Command::new(BIN).args(args).output().unwrap())
}

fn run_measurement(args: &[String]) -> (Option<i32>, String, String) {
    out_of(
        Command::new(BIN)
            .args(args)
            .env("NEKO_MEASUREMENT", "1")
            .output()
            .unwrap(),
    )
}

fn assert_rejected(code: Option<i32>, stderr: &str, message: &str, args: &[String]) {
    assert_eq!(code, Some(2), "args={args:?} stderr={stderr:?}");
    assert!(
        stderr.contains(message),
        "args={args:?} expected {message:?} in {stderr:?}"
    );
}

// ---------------------------------------------------------------------------
// 1. Default-off: the pre-measurement windows reject exactly as before.
// ---------------------------------------------------------------------------

#[test]
fn default_windows_reject_at_their_old_edges() {
    // client/server/probe duration window: 1-30.
    let (code, _, stderr) = run(&client_args(&[("--duration", "31")]));
    assert_rejected(code, &stderr, "duration outside 1-30", &client_args(&[]));
    // Exchange count window: 1-64.
    let (code, _, stderr) = run(&client_args(&[("--count", "65")]));
    assert_rejected(code, &stderr, "count outside 1-64", &client_args(&[]));
    // workload duration window: 1-600.
    let (code, _, stderr) = run(&argv(&["workload", "--duration", "601"]));
    assert_rejected(code, &stderr, "duration outside 1-600", &argv(&[]));
    // periodic count window: 1-600.
    let (code, _, stderr) = run(&argv(&[
        "periodic-client",
        "--count",
        "601",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert_rejected(code, &stderr, "count outside 1-600", &argv(&[]));
    // (The periodic application-byte ceiling is unreachable in the default
    // window: 600 * 1200 = 720_000 < 1_048_576, so the count bound always
    // bites first. The multistream records window completes the edge set.)
    let (code, _, stderr) = run(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "1",
        "--records",
        "65",
    ]));
    assert_rejected(code, &stderr, "records outside 1-64", &argv(&[]));
    // multistream stream window: 1-16.
    let (code, _, stderr) = run(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "17",
    ]));
    assert_rejected(code, &stderr, "streams outside 1-16", &argv(&[]));
}

#[test]
fn default_capabilities_and_stdout_carry_no_measurement_trace() {
    let (code, stdout, stderr) = run(&argv(&["capabilities", "--json"]));
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stdout.contains("\"measurement_mode\":false"), "{stdout}");
    assert!(stdout.contains("\"count_max\":64"), "{stdout}");
    assert!(stdout.contains("\"duration_seconds_max\":30"), "{stdout}");
    assert!(
        stdout.contains("\"workload_duration_seconds_max\":600"),
        "{stdout}"
    );
    assert!(
        stdout.contains("\"periodic_total_bytes_max\":1048576"),
        "{stdout}"
    );
    assert!(!stdout.contains("relaxed_limits"), "{stdout}");
    // A default-mode command prints no marker line.
    let (_, stdout, _) = run(&client_args(&[("--duration", "0")]));
    assert!(!stdout.contains("measurement_mode"), "{stdout}");
}

// ---------------------------------------------------------------------------
// 2. Opt-in: every ceiling accepts above the default window, rejects above
//    the measurement ceiling, and the marker line is present.
// ---------------------------------------------------------------------------

#[test]
fn measurement_mode_relaxes_duration_and_count_and_marks_output() {
    // Values above every default window pass validation (the command then
    // fails at the unreachable loopback target — expected); the marker line
    // proves relaxation applied, and no "outside" rejection appears.
    let (code, stdout, stderr) =
        run_measurement(&client_args(&[("--duration", "700"), ("--count", "100")]));
    assert!(
        stdout.contains("measurement_mode=true relaxed_limits"),
        "{stdout}"
    );
    assert!(!stderr.contains("outside"), "{stderr}");
    assert_eq!(code, Some(2), "{stderr}"); // connect refused at 127.0.0.1:9
    // The new ceiling still rejects: duration/count/workload windows.
    let (code, _, stderr) = run_measurement(&client_args(&[("--duration", "86401")]));
    assert_rejected(code, &stderr, "duration outside 1-86400", &argv(&[]));
    let (code, _, stderr) = run_measurement(&client_args(&[("--count", "13981")]));
    assert_rejected(code, &stderr, "count outside 1-13980", &argv(&[]));
    let (code, _, stderr) = run_measurement(&argv(&["workload", "--duration", "86401"]));
    assert_rejected(code, &stderr, "duration outside 1-86400", &argv(&[]));
}

#[test]
fn measurement_mode_relaxes_periodic_ceilings_up_to_the_library_limits() {
    // count above 600 passes config validation; the largest legal count is
    // 65_530 (count + 2 <= HARD_MAX_RUNTIME_QUEUE_RECORDS)...
    let (code, stdout, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--count",
        "65530",
        "--bytes",
        "32",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert!(
        stdout.contains("measurement_mode=true relaxed_limits"),
        "{stdout}"
    );
    assert!(!stderr.contains("outside"), "{stderr}");
    assert_eq!(code, Some(2), "{stderr}"); // connect refused
    // ...and one more is rejected by the config ceiling.
    let (code, _, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--count",
        "65531",
        "--bytes",
        "32",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert_rejected(code, &stderr, "count outside 1-65530", &argv(&[]));
    // Duration ceiling moves to 24 h.
    let (code, _, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--duration",
        "86401",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert_rejected(code, &stderr, "duration outside 1-86400", &argv(&[]));
    // Application bytes rise to 32 MiB (runtime lifetime total = 2x app bytes
    // must fit HARD_MAX_RUNTIME_TOTAL_BYTES = 64 MiB)...
    let (_, stdout, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--bytes",
        "1200",
        "--count",
        "13980",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert!(
        stdout.contains("measurement_mode=true relaxed_limits"),
        "{stdout}"
    );
    assert!(!stderr.contains("exceed"), "{stderr}");
    // ...and beyond it is rejected: the payload window itself (post-#3 the
    // periodic payload ceiling is the sealed-frame bound 1170, not 1200)...
    let (_, _, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--bytes",
        "1171",
        "--count",
        "2",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert!(
        stderr.contains("bytes outside 1-1170"),
        "payload above the sealed-frame ceiling must be rejected by name: {stderr}"
    );
    // ...and the 32 MiB application ceiling is still enforced with legal
    // payload sizes: 1170 * 28681 > 33_554_432 while 1170 * 28680 fits.
    let (_, _, stderr) = run_measurement(&argv(&[
        "periodic-client",
        "--bytes",
        "1170",
        "--count",
        "28681",
        "--addr",
        "127.0.0.1:40080",
    ]));
    assert!(
        stderr.contains("application bytes exceed 33554432")
            || stderr.contains("exceeds the runtime queue-byte limit"),
        "the 32 MiB application ceiling must still reject: {stderr}"
    );
}

#[test]
fn measurement_mode_relaxes_multistream_ceilings_up_to_the_library_limits() {
    // streams above 16 passes bounds validation (fails later at the missing
    // key, which is the accepted-value oracle). 256 x 255 x 1000 =
    // 65_280_000 <= HARD_MAX_RUNTIME_TOTAL_BYTES: the shape that sits at
    // the queue-record ceiling AND fits the 64 MiB lifetime total...
    let (_, stdout, stderr) = run_measurement(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "256",
        "--records",
        "255",
        "--bytes",
        "1000",
        "--server-key",
        PEER,
    ]));
    assert!(
        stdout.contains("measurement_mode=true relaxed_limits"),
        "{stdout}"
    );
    assert!(!stderr.contains("outside"), "{stderr}");
    assert!(!stderr.contains("exceeds"), "{stderr}");
    // ...while the new ceilings still reject: streams 257, bytes 4001, and
    // the records ceiling at the 65_536 queue-record declaration.
    let (code, _, stderr) = run_measurement(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "257",
    ]));
    assert_rejected(code, &stderr, "streams outside 1-256", &argv(&[]));
    let (code, _, stderr) = run_measurement(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--bytes",
        "4001",
    ]));
    assert_rejected(code, &stderr, "bytes outside 1-4000", &argv(&[]));
    let (code, _, stderr) = run_measurement(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "256",
        "--records",
        "256",
    ]));
    assert_rejected(
        code,
        &stderr,
        "records exceeds the measurement ceiling for this stream count",
        &argv(&[]),
    );
    // Total payload beyond the 64 MiB lifetime ceiling is rejected.
    let (code, _, stderr) = run_measurement(&argv(&[
        "multistream",
        "--mode",
        "client",
        "--streams",
        "256",
        "--records",
        "255",
        "--bytes",
        "4000",
    ]));
    assert_rejected(
        code,
        &stderr,
        "total payload exceeds the runtime total-byte limit",
        &argv(&[]),
    );
}

#[test]
fn measurement_capabilities_report_the_active_limits() {
    let (code, stdout, stderr) = run_measurement(&argv(&["capabilities", "--json"]));
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stdout.contains("\"measurement_mode\":true"), "{stdout}");
    assert!(stdout.contains("\"count_max\":13980"), "{stdout}");
    assert!(
        stdout.contains("\"duration_seconds_max\":86400"),
        "{stdout}"
    );
    assert!(
        stdout.contains("\"workload_duration_seconds_max\":86400"),
        "{stdout}"
    );
    assert!(
        stdout.contains("\"periodic_total_bytes_max\":33554432"),
        "{stdout}"
    );
}

// ---------------------------------------------------------------------------
// 3. End-to-end: relaxed windows carry a real loopback exchange.
// ---------------------------------------------------------------------------

struct LineChild {
    child: Child,
    lines: Receiver<String>,
}

fn spawn_line_server(args: &[String], measurement: bool) -> LineChild {
    let mut command = Command::new(BIN);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if measurement {
        command.env("NEKO_MEASUREMENT", "1");
    }
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().expect("stdout piped");
    let (tx, rx) = channel::<String>();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return,
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });
    LineChild { child, lines: rx }
}

impl LineChild {
    fn wait_marker(&mut self, marker: &str, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        let mut seen = String::new();
        loop {
            let remain = deadline.saturating_duration_since(Instant::now());
            if remain.is_zero() {
                let _ = self.child.kill();
                let _ = self.child.wait();
                panic!("marker {marker:?} not seen in time; seen:\n{seen}");
            }
            match self.lines.recv_timeout(remain) {
                Ok(line) => {
                    seen.push_str(&line);
                    if line.contains(marker) {
                        return;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    panic!("marker {marker:?} not seen in time; seen:\n{seen}");
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let status = self.child.wait();
                    panic!("child EOF before {marker:?}: {status:?}; seen:\n{seen}");
                }
            }
        }
    }

    fn exit_bounded(&mut self, timeout: Duration) -> std::process::ExitStatus {
        let deadline = Instant::now() + timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    panic!("child did not exit within {timeout:?}");
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(e) => panic!("try_wait failed: {e}"),
            }
        }
    }
}

impl Drop for LineChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn keygen(path: &std::path::Path) -> String {
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
        if let (Ok(tcp), Ok(_udp)) = (
            TcpListener::bind(("127.0.0.1", port)),
            std::net::UdpSocket::bind(("127.0.0.1", port)),
        ) {
            drop(tcp);
            drop(_udp);
            return port;
        }
    }
    panic!("no test port is locally available");
}

fn run_client_measurement(args: &[String]) -> std::process::ExitStatus {
    Command::new(BIN)
        .args(args)
        .env("NEKO_MEASUREMENT", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()
        .unwrap()
}

/// A full periodic exchange above the default duration window (700 s > 600):
/// both processes accept the window, the marker prints on both, and the
/// bounded exchange (3 records, 100 ms interval) completes well before it.
#[test]
fn measurement_periodic_exchange_runs_above_the_default_duration_window() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let port = lease_port();
    let server_identity = tmp("periodic-server-id");
    let client_identity = tmp("periodic-client-id");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let addr = format!("127.0.0.1:{port}");
    let mut server = spawn_line_server(
        &[
            "periodic-server".to_string(),
            "--port".into(),
            port.to_string(),
            "--bind".into(),
            addr.clone(),
            "--identity".into(),
            server_identity.to_string_lossy().into_owned(),
            "--client-key".into(),
            client_key,
            "--duration".into(),
            "700".into(), // above the default 600 ceiling
            "--count".into(),
            "3".into(),
            "--bytes".into(),
            "32".into(),
            "--interval-ms".into(),
            "100".into(),
        ],
        true,
    );
    server.wait_marker(
        "measurement_mode=true relaxed_limits",
        Duration::from_secs(5),
    );
    server.wait_marker("periodic_server_ready", Duration::from_secs(5));
    let status = run_client_measurement(&[
        "periodic-client".to_string(),
        "--port".into(),
        port.to_string(),
        "--addr".into(),
        addr,
        "--identity".into(),
        client_identity.to_string_lossy().into_owned(),
        "--server-key".into(),
        server_key,
        "--duration".into(),
        "700".into(),
        "--count".into(),
        "3".into(),
        "--bytes".into(),
        "32".into(),
        "--interval-ms".into(),
        "100".into(),
    ]);
    assert!(status.success(), "measurement periodic client failed");
    server.wait_marker(
        "periodic_server_summary authenticated=true",
        Duration::from_secs(10),
    );
    let status = server.exit_bounded(Duration::from_secs(10));
    assert!(status.success(), "measurement periodic server failed");
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}

/// A full multistream exchange above the default stream window (17 > 16):
/// both processes accept the shape and the bounded exchange completes.
#[test]
fn measurement_multistream_exchange_runs_above_the_default_stream_window() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let port = lease_port();
    let server_identity = tmp("multistream-server-id");
    let client_identity = tmp("multistream-client-id");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let addr = format!("127.0.0.1:{port}");
    let mut server = spawn_line_server(
        &[
            "multistream".to_string(),
            "--mode".into(),
            "server".into(),
            "--port".into(),
            port.to_string(),
            "--addr".into(),
            addr.clone(),
            "--identity".into(),
            server_identity.to_string_lossy().into_owned(),
            "--client-key".into(),
            client_key,
            "--streams".into(),
            "17".into(), // above the default 16 ceiling
            "--records".into(),
            "3".into(),
            "--bytes".into(),
            "32".into(),
        ],
        true,
    );
    server.wait_marker(
        "measurement_mode=true relaxed_limits",
        Duration::from_secs(5),
    );
    let status = run_client_measurement(&[
        "multistream".to_string(),
        "--mode".into(),
        "client".into(),
        "--port".into(),
        port.to_string(),
        "--addr".into(),
        addr,
        "--identity".into(),
        client_identity.to_string_lossy().into_owned(),
        "--server-key".into(),
        server_key,
        "--streams".into(),
        "17".into(),
        "--records".into(),
        "3".into(),
        "--bytes".into(),
        "32".into(),
    ]);
    assert!(status.success(), "measurement multistream client failed");
    server.wait_marker("\"ok\":true,\"role\":\"server\"", Duration::from_secs(10));
    let status = server.exit_bounded(Duration::from_secs(10));
    assert!(status.success(), "measurement multistream server failed");
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}
