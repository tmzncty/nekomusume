//! P0 admission-rejection regression: an unauthenticated remote must not be
//! able to terminate a TCP listener by triggering a pre-auth admission
//! rejection (or any single-connection pre-auth failure). Lane C T0-1/T0-2
//! (docs/reviews/review-2026-10-07-lane-c-tests.md) against Lane A F1
//! (docs/reviews/review-2026-10-07-lane-a-core.md).
//!
//! Every test runs on loopback only, spawns the real `neko-cli` binary, and
//! asserts the positive contract: after N abusive connections the server
//! process is still alive AND a legitimate client still completes its
//! exchange. Before the fix each TCP server exited with code 2 on the very
//! first abusive connection, so the three TCP tests fail (red) against the
//! unfixed tree; the failover UDP test locks the already-correct
//! reject-and-continue shape.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};
use std::{fs, thread};

/// The repository's port range (40080-40100) is shared across the whole test
/// binary; guard the lease + server lifetime with one mutex, mirroring
/// probe.rs's TEST_PORT_LOCK.
static PORT_LOCK: Mutex<()> = Mutex::new(());

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_neko-cli")
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("neko-p0-{name}-{}", std::process::id()))
}

fn keygen(path: &std::path::Path) -> String {
    let out = Command::new(bin())
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

/// Lease an in-range port by binding (TCP+UDP) then releasing, so the server
/// under test can bind it without colliding with parallel in-range tests.
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

/// Spawned-child guard: kills + reaps on drop so a failed or panicking test
/// cannot leak a loopback listener (established repo pattern, ServerGuard).
struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A spawned server whose stdout lines are forwarded by a reader thread; the
/// channel keeps every line so tests can wait for readiness markers, poll
/// diagnostics while the child is live, and keep the pipe drained (no
/// stdout-full deadlock).
struct LineChild {
    child: Child,
    lines: Receiver<String>,
}

fn spawn_line_server(args: &[String]) -> LineChild {
    let mut child = Command::new(bin())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
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
    /// Bounded readiness wait: consume lines until one contains the marker.
    /// Fails closed (kill + reap) on timeout; on reader EOF (child exiting)
    /// surfaces the child's exit status.
    fn wait_marker(&mut self, marker: &str, timeout: Duration) -> String {
        let deadline = Instant::now() + timeout;
        let mut seen = String::new();
        loop {
            let remain = deadline.saturating_duration_since(Instant::now());
            if remain.is_zero() {
                fail_closed(&mut self.child);
                panic!("server not ready within {timeout:?} (marker {marker:?}), seen:\n{seen}");
            }
            match self.lines.recv_timeout(remain) {
                Ok(line) => {
                    seen.push_str(&line);
                    if line.contains(marker) {
                        return seen;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    fail_closed(&mut self.child);
                    panic!(
                        "server not ready within {timeout:?} (marker {marker:?}), seen:\n{seen}"
                    );
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let status = self.child.wait();
                    panic!(
                        "server EOF before readiness marker {marker:?}: status={status:?}, seen:\n{seen}"
                    );
                }
            }
        }
    }

    /// Consume all currently buffered lines without blocking (used while the
    /// child is live to inspect diagnostics emitted so far).
    fn drain_available(&self) -> Vec<String> {
        let mut out = Vec::new();
        while let Ok(line) = self.lines.try_recv() {
            out.push(line);
        }
        out
    }

    fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn exit_bounded(&mut self, timeout: Duration) -> std::process::ExitStatus {
        let deadline = Instant::now() + timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return status,
                Ok(None) if Instant::now() >= deadline => {
                    fail_closed(&mut self.child);
                    panic!("child did not exit within {timeout:?}");
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(e) => panic!("try_wait failed: {e}"),
            }
        }
    }
}

fn fail_closed(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn bounded_connect(addr: &str) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match TcpStream::connect(addr) {
            Ok(stream) => return stream,
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                assert!(
                    Instant::now() < deadline,
                    "server did not accept connections before deadline: {e}"
                );
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("unexpected connect error: {e}"),
        }
    }
}

/// One abusive connection: connect (bounded retry while the server binds),
/// send a well-framed but unauthenticated handshake body (passes the length
/// bound, fails authentication or admission), and wait bounded for the server
/// to close its side. The exact rejection reason is irrelevant to the
/// kill-survival contract; what matters is that the listener survives it.
fn abusive_connection(addr: &str) {
    let mut stream = bounded_connect(addr);
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let body = [0x41u8; 64];
    stream
        .write_all(&(body.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&body).unwrap();
    // Drain bounded until EOF (the fixed server closes the connection after
    // rejection) or read timeout (an unfixed server never closes it: it
    // exits the whole process instead).
    let mut sink = [0u8; 64];
    loop {
        match stream.read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

/// Common assertions after the abusive phase: the server is still alive and a
/// legitimate client completes its exchange. Returns after the client exit is
/// observed; the caller still owns the server for its own exit assertions.
fn legit_client_succeeds(args: &[String]) {
    let mut client = KillOnDrop(
        Command::new(bin())
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let status = client.0.wait().expect("client wait");
    assert!(
        status.success(),
        "legitimate client failed after abusive connections"
    );
}

// ---------------------------------------------------------------------------
// 1. `server --transport tcp` (Lane A F1 / Lane C T0-1)
// ---------------------------------------------------------------------------

#[test]
fn server_tcp_survives_preauth_rejections_then_serves_client() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let port = lease_port();
    let server_identity = tmp("server-tcp-server");
    let client_identity = tmp("server-tcp-client");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let mut server = spawn_line_server(&[
        "server".to_string(),
        "--transport".into(),
        "tcp".into(),
        "--port".into(),
        port.to_string(),
        "--bind".into(),
        format!("127.0.0.1:{port}"),
        "--identity".into(),
        server_identity.to_string_lossy().into_owned(),
        "--client-key".into(),
        client_key,
        "--duration".into(),
        "20".into(),
    ]);
    server.wait_marker("lifecycle_state=READY", Duration::from_secs(5));
    let addr = format!("127.0.0.1:{port}");

    // Nine abusive connections: one more than the per-source pre-auth state
    // cap (8). Before the fix the first one kills the process.
    for i in 0..9 {
        abusive_connection(&addr);
        assert!(
            server.alive(),
            "server died at abusive connection {i} (pre-auth rejection must close the connection, not the listener)"
        );
    }

    legit_client_succeeds(&[
        "client".to_string(),
        "--transport".into(),
        "tcp".into(),
        "--port".into(),
        port.to_string(),
        "--addr".into(),
        addr,
        "--identity".into(),
        client_identity.to_string_lossy().into_owned(),
        "--server-key".into(),
        server_key,
        "--count".into(),
        "1".into(),
        "--bytes".into(),
        "16".into(),
        "--duration".into(),
        "5".into(),
    ]);

    // The server completes its single-session lifecycle and exits 0.
    let status = server.exit_bounded(Duration::from_secs(25));
    assert!(status.success(), "server did not exit cleanly");
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}

// ---------------------------------------------------------------------------
// 2. `periodic-server`
// ---------------------------------------------------------------------------

#[test]
fn periodic_server_survives_preauth_rejections_then_serves_client() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let port = lease_port();
    let server_identity = tmp("periodic-server-id");
    let client_identity = tmp("periodic-client-id");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let mut server = spawn_line_server(&[
        "periodic-server".to_string(),
        "--port".into(),
        port.to_string(),
        "--bind".into(),
        format!("127.0.0.1:{port}"),
        "--identity".into(),
        server_identity.to_string_lossy().into_owned(),
        "--client-key".into(),
        client_key,
        "--duration".into(),
        "20".into(),
        "--count".into(),
        "3".into(),
        "--bytes".into(),
        "16".into(),
        "--interval-ms".into(),
        "100".into(),
        "--ack-timeout-ms".into(),
        "500".into(),
    ]);
    server.wait_marker("periodic_server_ready", Duration::from_secs(5));
    let addr = format!("127.0.0.1:{port}");

    for i in 0..9 {
        abusive_connection(&addr);
        assert!(
            server.alive(),
            "periodic server died at abusive connection {i}"
        );
    }

    legit_client_succeeds(&[
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
        "10".into(),
        "--count".into(),
        "3".into(),
        "--bytes".into(),
        "16".into(),
        "--interval-ms".into(),
        "100".into(),
        "--ack-timeout-ms".into(),
        "500".into(),
    ]);

    let status = server.exit_bounded(Duration::from_secs(25));
    assert!(status.success(), "periodic server did not exit cleanly");
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}

// ---------------------------------------------------------------------------
// 3. `multistream --mode server`
// ---------------------------------------------------------------------------

#[test]
fn multistream_server_survives_preauth_rejections_then_serves_client() {
    // multistream has no in-range port requirement and no ready marker; the
    // first abusive connect (with bounded retry) doubles as the readiness
    // proof.
    let hold = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = hold.local_addr().unwrap().port();
    drop(hold);
    let server_identity = tmp("multistream-server-id");
    let client_identity = tmp("multistream-client-id");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let mut server = spawn_line_server(&[
        "multistream".to_string(),
        "--mode".into(),
        "server".into(),
        "--addr".into(),
        format!("127.0.0.1:{port}"),
        "--streams".into(),
        "1".into(),
        "--records".into(),
        "1".into(),
        "--bytes".into(),
        "16".into(),
        "--session-window".into(),
        "17".into(),
        "--stream-window".into(),
        "17".into(),
        "--identity".into(),
        server_identity.to_string_lossy().into_owned(),
        "--client-key".into(),
        client_key,
    ]);
    let addr = format!("127.0.0.1:{port}");

    for i in 0..9 {
        abusive_connection(&addr);
        assert!(
            server.alive(),
            "multistream server died at abusive connection {i}"
        );
    }

    legit_client_succeeds(&[
        "multistream".to_string(),
        "--mode".into(),
        "client".into(),
        "--addr".into(),
        addr,
        "--streams".into(),
        "1".into(),
        "--records".into(),
        "1".into(),
        "--bytes".into(),
        "16".into(),
        "--session-window".into(),
        "17".into(),
        "--stream-window".into(),
        "17".into(),
        "--identity".into(),
        client_identity.to_string_lossy().into_owned(),
        "--server-key".into(),
        server_key,
    ]);

    let status = server.exit_bounded(Duration::from_secs(10));
    assert!(status.success(), "multistream server did not exit cleanly");
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}

// ---------------------------------------------------------------------------
// 4. UDP regression: failover-server rejects + continues (locks T0-2)
// ---------------------------------------------------------------------------

#[test]
fn failover_server_udp_admission_rejections_are_counted_and_survived() {
    let _ports = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let port = lease_port();
    let server_identity = tmp("failover-server-id");
    let client_identity = tmp("failover-client-id");
    let client_key = keygen(&client_identity);
    let server_key = keygen(&server_identity);
    let mut server = spawn_line_server(&[
        "failover-server".to_string(),
        "--udp-port".into(),
        port.to_string(),
        "--tcp-port".into(),
        port.to_string(),
        "--udp-bind".into(),
        format!("127.0.0.1:{port}"),
        "--tcp-bind".into(),
        format!("127.0.0.1:{port}"),
        "--identity".into(),
        server_identity.to_string_lossy().into_owned(),
        "--client-key".into(),
        client_key,
        "--server-key".into(),
        server_key,
        "--count".into(),
        "1".into(),
        "--bytes".into(),
        "16".into(),
        "--duration".into(),
        "5".into(),
        "--diagnostic".into(),
        "--experiment-id".into(),
        "p0-udp-admission-reject".into(),
    ]);
    server.wait_marker("\"event\":\"start\"", Duration::from_secs(5));

    // Abusive datagrams from fresh sources: each must be classified
    // (malformed_or_unadmitted) and released without terminating the
    // listener. This locks the reject-and-continue shape the TCP servers are
    // being migrated onto (the failover_server non-pending UDP path).
    let malformed = [b'N', b'1', 1, 1, 0];
    for _ in 0..2 {
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender.send_to(&malformed, ("127.0.0.1", port)).unwrap();
    }
    // Bounded wait for BOTH classification diagnostics while staying live.
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut classified = 0usize;
    while Instant::now() < deadline {
        for line in server.drain_available() {
            if line.contains("malformed_or_unadmitted") {
                classified += 1;
            }
        }
        if classified >= 2 {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        server.alive(),
        "failover server died on malformed datagrams"
    );
    assert_eq!(
        classified, 2,
        "expected one malformed_or_unadmitted diagnostic per abusive datagram"
    );
    // An idle failover server exits 2 at duration expiry by design; the test
    // has proven survival past the datagrams, so kill it here (LineChild's
    // child is killed via fail_closed on scope end through the guard below).
    fail_closed(&mut server.child);
    let _ = fs::remove_file(&server_identity);
    let _ = fs::remove_file(&client_identity);
}
