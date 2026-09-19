use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, TrustPolicy, TrustRecord,
    TrustStatus,
};
use neko_wire::{NEGOTIATION_VERSION, NegotiationRole, VersionNegotiator};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neko-cli-{name}-{}", std::process::id()))
}
fn process_resource_snapshot(pid: u32) -> Option<(usize, usize)> {
    let fd_count = fs::read_dir(format!("/proc/{pid}/fd")).ok()?.count();
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let rss_kib = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some((fd_count, rss_kib))
}

fn key(bin: &str, path: &std::path::Path) -> String {
    let out = Command::new(bin)
        .args(["keygen", "--identity", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .strip_prefix("client_public_key=")
        .unwrap()
        .to_string()
}

struct ReadyServer {
    child: Child,
    stdout: BufReader<std::process::ChildStdout>,
    startup_log: String,
}

fn start_server(
    bin: &str,
    transport: &str,
    port: u16,
    identity: &std::path::Path,
    client_key: &str,
) -> ReadyServer {
    start_server_for(bin, transport, port, identity, client_key, "5")
}

fn start_server_for(
    bin: &str,
    transport: &str,
    port: u16,
    identity: &std::path::Path,
    client_key: &str,
    duration: &str,
) -> ReadyServer {
    let mut child = Command::new(bin)
        .args([
            "server",
            "--transport",
            transport,
            "--port",
            &port.to_string(),
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--identity",
            identity.to_str().unwrap(),
            "--client-key",
            client_key,
            "--duration",
            duration,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut startup_log = String::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        assert!(Instant::now() < deadline, "READY timeout: {startup_log}");
        let mut line = String::new();
        let read = stdout.read_line(&mut line).unwrap();
        assert_ne!(read, 0, "server exited before READY: {startup_log}");
        startup_log.push_str(&line);
        if line.contains("lifecycle_state=READY readiness=true") {
            break;
        }
    }
    ReadyServer {
        child,
        stdout,
        startup_log,
    }
}

fn finish_server(mut server: ReadyServer) -> (std::process::ExitStatus, String) {
    let mut remainder = String::new();
    server.stdout.read_to_string(&mut remainder).unwrap();
    let status = server.child.wait().unwrap();
    server.startup_log.push_str(&remainder);
    (status, server.startup_log)
}

fn ready_failover_server(mut child: Child) -> ReadyServer {
    // `read_line` is blocking, so perform it off-thread and bound the wait on
    // the receiver. On timeout terminate the child; EOF then releases the
    // reader and preserves every line read before the diagnostic start event.
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let stdout = child.stdout.take().unwrap();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut startup_log = String::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    let _ = tx.send(Err(startup_log));
                    return;
                }
                Ok(_) => {
                    startup_log.push_str(&line);
                    if line.contains("\"role\":\"server\",\"event\":\"start\"") {
                        let _ = tx.send(Ok((reader, startup_log)));
                        return;
                    }
                }
                Err(error) => {
                    startup_log.push_str(&format!("stdout read error: {error}"));
                    let _ = tx.send(Err(startup_log));
                    return;
                }
            }
        }
    });
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok((stdout, startup_log))) => ReadyServer {
            child,
            stdout,
            startup_log,
        },
        Ok(Err(startup_log)) => {
            let _ = child.wait();
            panic!("failover exited before start: {startup_log}");
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("failover start timeout: {error}");
        }
    }
}

fn ready_endpoint_rebind_server(mut child: Child) -> ReadyServer {
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut startup_log = String::new();
    loop {
        let mut line = String::new();
        assert_ne!(
            reader.read_line(&mut line).unwrap(),
            0,
            "endpoint rebind exited before ready: {startup_log}"
        );
        startup_log.push_str(&line);
        if line.contains("endpoint_rebind_server_ready") {
            return ReadyServer {
                child,
                stdout: reader,
                startup_log,
            };
        }
    }
}

fn signal_term(child: &Child) {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
}

// H-R9-032: extract the numeric `packet_number` field from a diagnostic line.
// Reuses the same split/trim/parse pattern as the positive P2 fixture rather
// than inventing a new runtime diagnostic or protocol field.
fn packet_number(line: &str) -> Option<u64> {
    line.split("\"packet_number\":").nth(1).and_then(|v| {
        v.trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse::<u64>()
            .ok()
    })
}
#[test]
fn rejects_unbounded_arguments() {
    let out = Command::new(env!("CARGO_BIN_EXE_neko-cli"))
        .args([
            "client",
            "--transport",
            "tcp",
            "--port",
            "40079",
            "--addr",
            "127.0.0.1:40079",
            "--server-key",
            "00",
            "--bytes",
            "1201",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
#[test]
fn matrix_probe_distinguishes_invalid_failed_and_reachable_outcomes() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let run = |args: &[&str]| Command::new(bin).args(args).output().unwrap();

    for args in [
        vec![
            "probe",
            "--matrix",
            "--target",
            "192.0.2.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv6",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:0",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--timeout-ms",
            "0",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--timeout-ms",
            "5001",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bytes",
            "0",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bytes",
            "1201",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--timeout-ms",
            "not-a-number",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bytes",
            "not-a-number",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bogus",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "stray",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--target",
            "127.0.0.1:10",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--transport",
            "udp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--ip-version",
            "ipv6",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--timeout-ms",
            "100",
            "--timeout-ms",
            "200",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bytes",
            "17",
            "--bytes",
            "1201",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "--ip-version",
            "ipv4",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--timeout-ms",
            "--json",
        ],
        vec![
            "probe",
            "--matrix",
            "--target",
            "127.0.0.1:9",
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
            "--bytes",
            "--json",
        ],
    ] {
        let out = run(&args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "args={args:?} stdout={} stderr={}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stdout.is_empty(),
            "invalid input must not emit a reachability result"
        );
    }

    // Keep the UDP port owned but intentionally never answer. This produces a
    // deterministic completed failure without a bind-release race.
    let silent_udp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let silent_addr = silent_udp.local_addr().unwrap();
    let failed_out = run(&[
        "probe",
        "--matrix",
        "--target",
        &silent_addr.to_string(),
        "--transport",
        "udp",
        "--ip-version",
        "ipv4",
        "--timeout-ms",
        "100",
        "--json",
    ]);
    assert_eq!(failed_out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed_out.stdout).contains("\"reachable\":false"));

    let tcp = TcpListener::bind("127.0.0.1:0").unwrap();
    let tcp_addr = tcp.local_addr().unwrap();
    let tcp_peer = thread::spawn(move || tcp.accept().unwrap());
    let before_tcp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let tcp_out = run(&[
        "probe",
        "--matrix",
        "--target",
        &tcp_addr.to_string(),
        "--transport",
        "tcp",
        "--ip-version",
        "ipv4",
        "--json",
    ]);
    let _ = tcp_peer.join().unwrap();
    let after_tcp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    assert_eq!(tcp_out.status.code(), Some(0));
    let tcp_json = String::from_utf8_lossy(&tcp_out.stdout);
    assert!(tcp_json.contains("\"reachable\":true"));
    let observed = tcp_json
        .split("\"observed_at_unix_ms\":")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!((before_tcp..=after_tcp).contains(&observed));

    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let udp_addr = udp.local_addr().unwrap();
    let udp_peer = thread::spawn(move || {
        let mut buf = [0u8; 1200];
        let (n, peer) = udp.recv_from(&mut buf).unwrap();
        udp.send_to(&buf[..n], peer).unwrap();
    });
    let udp_out = run(&[
        "probe",
        "--matrix",
        "--target",
        &udp_addr.to_string(),
        "--transport",
        "udp",
        "--ip-version",
        "ipv4",
        "--bytes",
        "17",
        "--json",
    ]);
    udp_peer.join().unwrap();
    assert_eq!(udp_out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&udp_out.stdout).contains("\"reachable\":true"));

    let failed_human = run(&[
        "probe",
        "--matrix",
        "--target",
        &silent_addr.to_string(),
        "--transport",
        "udp",
        "--ip-version",
        "ipv4",
        "--timeout-ms",
        "100",
    ]);
    assert_eq!(failed_human.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&failed_human.stdout).trim(),
        "fail: 喵呜呜呜呜…"
    );
}

#[test]
fn help_and_capabilities_cover_dispatch_surface() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let help = Command::new(bin).arg("--help").output().unwrap();
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("probe --matrix"));
    assert!(help.contains("local-loopback only"));
    let human_capabilities = Command::new(bin).arg("capabilities").output().unwrap();
    assert!(human_capabilities.status.success());
    let human_capabilities = String::from_utf8_lossy(&human_capabilities.stdout);
    let capabilities = Command::new(bin)
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    assert!(capabilities.status.success());
    let capabilities = String::from_utf8_lossy(&capabilities.stdout);
    for command in [
        "server",
        "client",
        "probe",
        "health-observe",
        "failover",
        "multistream",
        "scheduler-fairness",
        "key-update",
        "periodic-server",
        "periodic-client",
        "lab",
        "workload",
        "endpoint-rebind-server",
        "endpoint-rebind-client",
        "keygen",
        "capabilities",
    ] {
        assert!(help.contains(command), "help missing {command}: {help}");
        assert!(
            human_capabilities.contains(command),
            "human capabilities missing {command}: {human_capabilities}"
        );
        assert!(
            capabilities.contains(&format!("\"name\":\"{command}\"")),
            "JSON capabilities missing {command}: {capabilities}"
        );
    }
    assert!(help.contains("failover --role server|client"));
    assert!(help.contains("failover-server|failover-client"));
    for alias in ["failover-server", "failover-client"] {
        assert!(
            human_capabilities.contains(alias),
            "human capabilities missing alias {alias}: {human_capabilities}"
        );
        assert!(
            !capabilities.contains(&format!("\"name\":\"{alias}\"")),
            "JSON capabilities must keep {alias} as an alias, not a canonical command"
        );
    }
    let unknown = Command::new(bin).arg("not-a-command").output().unwrap();
    assert!(!unknown.status.success());
    for args in [vec!["failover"], vec!["failover", "--role", "unknown"]] {
        let identity = tmp("canonical-failover-invalid-role");
        let _ = fs::remove_file(&identity);
        let output = Command::new(bin)
            .args(args)
            .args(["--identity", identity.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!identity.exists());
    }
    for (role, expected) in [("server", "client-key"), ("client", "server-key")] {
        let output = Command::new(bin)
            .args(["failover", "--role", role])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn advertised_socket_free_fixtures_execute_and_reject_bounds() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let positives = [
        (
            vec![
                "health-observe",
                "--path",
                "7",
                "--rtt-us",
                "1200",
                "--loss-per-mille",
                "5",
                "--pto",
                "1",
                "--count",
                "2",
                "--json",
            ],
            "\"samples\"",
        ),
        (
            vec![
                "scheduler-fairness",
                "--rounds",
                "2",
                "--bytes",
                "8",
                "--json",
            ],
            "\"fixture\":\"scheduler-fairness\"",
        ),
        (
            vec![
                "workload",
                "--duration",
                "1",
                "--concurrency",
                "2",
                "--records",
                "3",
                "--bytes",
                "8",
                "--json",
            ],
            "\"fixture\":\"session-workload\"",
        ),
        (
            vec!["key-update", "--json"],
            "\"fixture\":\"secure-session-key-update\"",
        ),
        (vec!["lab", "--json"], "\"demo\":\"failover\""),
    ];
    for (args, expected) in positives {
        let output = Command::new(bin).args(&args).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "args={args:?} stdout={stdout}");
        if args[0] != "health-observe" {
            assert!(
                stdout.contains("\"ok\":true"),
                "args={args:?} stdout={stdout}"
            );
        }
        assert!(stdout.contains(expected), "args={args:?} stdout={stdout}");
    }

    for args in [
        vec![
            "health-observe",
            "--path",
            "7",
            "--rtt-us",
            "1200",
            "--loss-per-mille",
            "1001",
            "--pto",
            "1",
        ],
        vec!["scheduler-fairness", "--rounds", "65"],
        vec!["workload", "--duration", "0"],
    ] {
        let output = Command::new(bin).args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
    }
}

#[test]
fn authenticated_tcp_and_udp_loopback_probe_starts_after_ready() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40093u16), ("udp", 40094u16)] {
        let sp = tmp(&format!("{transport}-ready-server"));
        let cp = tmp(&format!("{transport}-ready-client"));
        let sk = key(bin, &sp);
        let ck = key(bin, &cp);
        let server = start_server(bin, transport, port, &sp, &ck);
        assert!(
            server
                .startup_log
                .contains("lifecycle_state=READY readiness=true")
        );

        let out = Command::new(bin)
            .args([
                "client",
                "--transport",
                transport,
                "--port",
                &port.to_string(),
                "--addr",
                &format!("127.0.0.1:{port}"),
                "--server-key",
                &sk,
                "--identity",
                cp.to_str().unwrap(),
                "--bytes",
                "32",
                "--duration",
                "2",
            ])
            .output()
            .unwrap();
        let (server_status, server_log) = finish_server(server);
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("probe_ok"));
        assert!(server_status.success(), "{server_log}");
        assert!(server_log.contains("lifecycle_state=STOPPED readiness=false"));
    }
}

#[test]
fn authenticated_tcp_benchmark_echoes_exact_payload_and_hash() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("benchmark-server");
    let cp = tmp("benchmark-client");
    let payload_path = tmp("benchmark-payload");
    let payload = b"equal-application-payload";
    fs::write(&payload_path, payload).unwrap();
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let server = start_server(bin, "tcp", 40081, &sp, &ck);

    let out = Command::new(bin)
        .args([
            "client",
            "--transport",
            "tcp",
            "--port",
            "40081",
            "--addr",
            "127.0.0.1:40081",
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--bytes",
            &payload.len().to_string(),
            "--count",
            "1",
            "--duration",
            "2",
            "--payload-file",
            payload_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let _ = fs::remove_file(payload_path);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let log = String::from_utf8(out.stdout).unwrap();
    assert!(log.contains("\"application_bytes\":25"), "{log}");
    assert!(
        log.contains("\"payload_sha256\":\"cf241de87cf4e86eca5350ac13106043592ab1a8bceb97851833d90440b52cef\""),
        "{log}"
    );
    assert!(log.contains("\"fd_count\":"), "{log}");
    assert!(log.contains("\"wire_bytes\":null"), "{log}");
    assert!(server_status.success(), "{server_log}");
}

#[test]
fn benchmark_payload_mode_fails_closed_outside_exact_contract() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let payload_path = tmp("benchmark-invalid-payload");
    fs::write(&payload_path, b"abcd").unwrap();
    let identity_path = tmp("benchmark-invalid-identity");
    for extra in [
        vec!["--transport", "udp", "--count", "1", "--json"],
        vec!["--transport", "tcp", "--count", "2", "--json"],
        vec!["--transport", "tcp", "--count", "1"],
    ] {
        let mut args = vec![
            "client",
            "--port",
            "40080",
            "--addr",
            "127.0.0.1:40080",
            "--server-key",
            "00",
            "--bytes",
            "4",
            "--payload-file",
            payload_path.to_str().unwrap(),
            "--identity",
            identity_path.to_str().unwrap(),
        ];
        args.extend(extra);
        let out = Command::new(bin).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
    }
    let _ = fs::remove_file(payload_path);
    let _ = fs::remove_file(identity_path);
}

#[cfg(unix)]
#[test]
fn identity_files_are_owner_only_regular_and_fail_closed() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let secure = tmp("identity-secure");
    let permissive = tmp("identity-permissive");
    let target = tmp("identity-symlink-target");
    let link = tmp("identity-symlink");
    for path in [&secure, &permissive, &target, &link] {
        let _ = fs::remove_file(path);
    }

    let first = Command::new(bin)
        .args(["keygen", "--identity", secure.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(first.status.success());
    assert_eq!(
        fs::symlink_metadata(&secure).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let second = Command::new(bin)
        .args(["keygen", "--identity", secure.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);

    let restrictive = tmp("identity-restrictive-umask");
    let _ = fs::remove_file(&restrictive);
    let created = Command::new("sh")
        .args([
            "-c",
            "umask 0777; exec \"$1\" keygen --identity \"$2\"",
            "sh",
            bin,
            restrictive.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    assert_eq!(
        fs::symlink_metadata(&restrictive)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let reloaded = Command::new(bin)
        .args(["keygen", "--identity", restrictive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(reloaded.status.success());
    assert_eq!(created.stdout, reloaded.stdout);

    fs::copy(&secure, &permissive).unwrap();
    fs::set_permissions(&permissive, fs::Permissions::from_mode(0o644)).unwrap();
    let before = fs::read(&permissive).unwrap();
    let rejected = Command::new(bin)
        .args(["keygen", "--identity", permissive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert_eq!(fs::read(&permissive).unwrap(), before);
    assert_eq!(
        fs::symlink_metadata(&permissive)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );

    fs::copy(&secure, &target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    symlink(&target, &link).unwrap();
    let target_before = fs::read(&target).unwrap();
    let rejected = Command::new(bin)
        .args(["keygen", "--identity", link.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert_eq!(fs::read(&target).unwrap(), target_before);

    let client = tmp("identity-invalid-server-client");
    let client_key = key(bin, &client);
    let server = Command::new(bin)
        .args([
            "server",
            "--transport",
            "tcp",
            "--bind",
            "127.0.0.1:0",
            "--identity",
            permissive.to_str().unwrap(),
            "--client-key",
            &client_key,
        ])
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&server.stdout),
        String::from_utf8_lossy(&server.stderr)
    );
    assert!(!server.status.success());
    assert!(!log.contains("lifecycle_state=READY"), "{log}");

    for path in [&secure, &restrictive, &permissive, &target, &link, &client] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn deterministic_invalid_configuration_does_not_create_identity() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let cases = [
        (
            tmp("invalid-config-server-key"),
            vec![
                "server".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--bind".into(),
                "127.0.0.1:40080".into(),
                "--client-key".into(),
                "zz".into(),
            ],
        ),
        (
            tmp("invalid-config-server-short-key"),
            vec![
                "server".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--bind".into(),
                "127.0.0.1:40080".into(),
                "--client-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-server-long-key"),
            vec![
                "server".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--bind".into(),
                "127.0.0.1:40080".into(),
                "--client-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-client-short-key"),
            vec![
                "client".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--addr".into(),
                "127.0.0.1:40080".into(),
                "--server-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-client-long-key"),
            vec![
                "client".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--addr".into(),
                "127.0.0.1:40080".into(),
                "--server-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-server-bind"),
            vec![
                "server".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--bind".into(),
                "not-an-address".into(),
                "--client-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-client-address"),
            vec![
                "client".to_string(),
                "--transport".into(),
                "tcp".into(),
                "--port".into(),
                "40080".into(),
                "--addr".into(),
                "not-an-address".into(),
                "--server-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-failover-server-hex"),
            vec![
                "failover-server".to_string(),
                "--client-key".into(),
                "zz".into(),
            ],
        ),
        (
            tmp("invalid-config-failover-server-short-key"),
            vec![
                "failover-server".to_string(),
                "--client-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-failover-server-long-key"),
            vec![
                "failover-server".to_string(),
                "--client-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-failover-server-udp-bind"),
            vec![
                "failover-server".to_string(),
                "--udp-bind".into(),
                "not-an-address".into(),
                "--client-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-failover-server-tcp-bind"),
            vec![
                "failover-server".to_string(),
                "--tcp-bind".into(),
                "not-an-address".into(),
                "--client-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-failover-client-short-key"),
            vec![
                "failover-client".to_string(),
                "--server-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-failover-client-long-key"),
            vec![
                "failover-client".to_string(),
                "--server-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-failover-client-address"),
            vec![
                "failover-client".to_string(),
                "--addr".into(),
                "not-an-address".into(),
                "--server-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-endpoint-server-short-key"),
            vec![
                "endpoint-rebind-server".to_string(),
                "--client-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-endpoint-server-long-key"),
            vec![
                "endpoint-rebind-server".to_string(),
                "--client-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-endpoint-server-bind"),
            vec![
                "endpoint-rebind-server".to_string(),
                "--udp-bind".into(),
                "not-an-address".into(),
                "--client-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-endpoint-client-short-key"),
            vec![
                "endpoint-rebind-client".to_string(),
                "--server-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-endpoint-client-long-key"),
            vec![
                "endpoint-rebind-client".to_string(),
                "--server-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-periodic-server-hex"),
            vec![
                "periodic-server".to_string(),
                "--client-key".into(),
                "zz".into(),
            ],
        ),
        (
            tmp("invalid-config-periodic-server-short-key"),
            vec![
                "periodic-server".to_string(),
                "--client-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-periodic-server-long-key"),
            vec![
                "periodic-server".to_string(),
                "--client-key".into(),
                "00".repeat(33),
            ],
        ),
        (
            tmp("invalid-config-periodic-server-bind"),
            vec![
                "periodic-server".to_string(),
                "--client-key".into(),
                "00".repeat(32),
                "--bind".into(),
                "not-an-address".into(),
            ],
        ),
        (
            tmp("invalid-config-periodic-server-bind-port"),
            vec![
                "periodic-server".to_string(),
                "--port".into(),
                "40080".into(),
                "--bind".into(),
                "127.0.0.1:40081".into(),
                "--client-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-periodic-client-hex"),
            vec![
                "periodic-client".to_string(),
                "--addr".into(),
                "127.0.0.1:40080".into(),
                "--server-key".into(),
                "zz".into(),
            ],
        ),
        (
            tmp("invalid-config-periodic-client-address"),
            vec![
                "periodic-client".to_string(),
                "--addr".into(),
                "not-an-address".into(),
                "--server-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-periodic-client-address-port"),
            vec![
                "periodic-client".to_string(),
                "--port".into(),
                "40080".into(),
                "--addr".into(),
                "127.0.0.1:40081".into(),
                "--server-key".into(),
                "00".repeat(32),
            ],
        ),
        (
            tmp("invalid-config-periodic-client-short-key"),
            vec![
                "periodic-client".to_string(),
                "--addr".into(),
                "127.0.0.1:40080".into(),
                "--server-key".into(),
                "00".repeat(31),
            ],
        ),
        (
            tmp("invalid-config-periodic-client-long-key"),
            vec![
                "periodic-client".to_string(),
                "--addr".into(),
                "127.0.0.1:40080".into(),
                "--server-key".into(),
                "00".repeat(33),
            ],
        ),
    ];
    for (identity, mut args) in cases {
        let _ = fs::remove_file(&identity);
        args.extend(["--identity".into(), identity.to_string_lossy().into_owned()]);
        let output = Command::new(bin).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(
            !identity.exists(),
            "invalid configuration created {identity:?}"
        );
    }
}

#[test]
fn invalid_bind_never_emits_ready() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("invalid-bind-server");
    let cp = tmp("invalid-bind-client");
    let ck = key(bin, &cp);
    let out = Command::new(bin)
        .args([
            "server",
            "--transport",
            "tcp",
            "--port",
            "40080",
            "--bind",
            "not-an-address",
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
        ])
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(!out.status.success());
    assert!(!log.contains("lifecycle_state=READY"), "{log}");
}

#[test]
fn sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40084u16), ("udp", 40085u16)] {
        let sp = tmp(&format!("{transport}-sigterm-server"));
        let cp = tmp(&format!("{transport}-sigterm-client"));
        let ck = key(bin, &cp);
        let server = start_server(bin, transport, port, &sp, &ck);
        signal_term(&server.child);
        let (status, log) = finish_server(server);
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
        assert!(status.success(), "{log}");
        assert!(
            log.contains("lifecycle_state=READY readiness=true"),
            "{log}"
        );
        let ready = log.find("lifecycle_state=READY readiness=true").unwrap();
        let draining = log
            .find("lifecycle_state=DRAINING readiness=false")
            .unwrap();
        let stopped = log.find("lifecycle_state=STOPPED readiness=false").unwrap();
        assert!(ready < draining && draining < stopped, "{log}");
        assert_eq!(
            log.matches("lifecycle_state=DRAINING readiness=false")
                .count(),
            1,
            "{log}"
        );
        assert_eq!(
            log.matches("lifecycle_state=STOPPED readiness=false")
                .count(),
            1,
            "{log}"
        );
        if transport == "tcp" {
            drop(TcpListener::bind(("127.0.0.1", port)).unwrap());
        } else {
            drop(UdpSocket::bind(("127.0.0.1", port)).unwrap());
        }
    }
}

#[test]
fn udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("preauth-churn-server");
    let cp = tmp("preauth-churn-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "1",
            "--bytes",
            "16",
            "--duration",
            "5",
            "--diagnostic",
            "--experiment-id",
            "preauth-churn-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = server.id();
    let server = ready_failover_server(server);
    let before = process_resource_snapshot(pid);
    let malformed = [b'N', b'1', 1, 1, 0];
    const ATTEMPTS: usize = 8;
    let senders: Vec<_> = (0..ATTEMPTS)
        .map(|_| UdpSocket::bind("127.0.0.1:0").unwrap())
        .collect();
    let source_ports: std::collections::BTreeSet<_> = senders
        .iter()
        .map(|socket| socket.local_addr().unwrap().port())
        .collect();
    assert_eq!(source_ports.len(), ATTEMPTS);
    for sender in &senders {
        sender.send_to(&malformed, ("127.0.0.1", udp)).unwrap();
    }
    thread::sleep(Duration::from_millis(100));
    let after = process_resource_snapshot(pid);
    if let (Some((before_fd, before_rss)), Some((after_fd, after_rss))) = (before, after) {
        assert!(
            after_fd <= before_fd + 1,
            "fd growth: {before_fd} -> {after_fd}"
        );
        assert!(
            after_rss <= before_rss + 4096,
            "rss growth KiB: {before_rss} -> {after_rss}"
        );
    }
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--count",
            "1",
            "--bytes",
            "16",
            "--duration",
            "3",
        ])
        .output()
        .unwrap();
    let (status, log) = finish_server(server);
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(status.success(), "{log}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("failover_client_ok"));
    assert!(log.contains("failover_server_ok"));
    drop(UdpSocket::bind(("127.0.0.1", udp)).unwrap());
    drop(TcpListener::bind(("127.0.0.1", tcp)).unwrap());
    fs::remove_file(&sp).unwrap();
    fs::remove_file(&cp).unwrap();
    assert!(!sp.exists());
    assert!(!cp.exists());
}

#[test]
fn executable_loopback_controlled_udp_stop_tcp_resume() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("failover-server");
    let cp = tmp("failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover",
            "--role",
            "server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "5",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--diagnostic",
            "--experiment-id",
            "primary-a-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover",
            "--role",
            "client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "3",
            "--diagnostic",
            "--experiment-id",
            "primary-a-client",
        ])
        .output()
        .unwrap();
    let (_server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        out.status.success(),
        "status={:?} stdout={} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("controlled_udp_stop=true"));
    let client_log = String::from_utf8_lossy(&out.stdout);
    for event in [
        "udp_authenticated",
        "controlled_udp_stop",
        "tcp_resume_guard",
        "ordered_records_complete",
    ] {
        assert!(
            client_log.contains(&format!("carrier_event name={event}")),
            "missing client event {event}: {client_log}"
        );
    }
    assert!(server_log.contains("carrier_event name=udp_authenticated"));
    assert!(server_log.contains("carrier_event name=tcp_resumed"));
    assert!(client_log.contains("\"event\":\"udp_delivery_ack_validated\""));
    assert_eq!(
        client_log
            .matches("\"event\":\"tcp_delivery_ack_validated\"")
            .count(),
        2
    );
    assert!(client_log.contains("\"event\":\"udp_uncertain_range_sent\""));
    assert!(!client_log.contains("udp_ack_observed"));
    assert!(
        !server_log.contains("duplicates="),
        "server must not report an unmeasured duplicate constant: {server_log}"
    );
    assert!(server_log.contains("records=3 application_bytes_total=48"));
    assert!(server_log.contains("controlled_udp_stop=true"));
    assert!(server_log.contains("\"count\":3"));
    assert!(server_log.contains("\"record_payload_bytes\":16"));
    assert!(server_log.contains(&format!("\"udp_port\":{udp}")));
    assert!(server_log.contains(&format!("\"tcp_port\":{tcp}")));
    assert!(server_log.contains("\"max_seconds\":5"));
    assert!(client_log.contains("\"application_bytes_total\":48"));
    assert!(
        server_log.contains(&format!("bytes_hex={}", "78".repeat(48))),
        "incomplete ordered bytes: {server_log}"
    );
}

#[test]
fn executable_loopback_health_threshold_drives_udp_to_tcp() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("health-failover-server");
    let cp = tmp("health-failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "5",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--cease-udp-replies-after",
            "1",
            "--send-malformed-after-cessation",
            "--diagnostic",
            "--experiment-id",
            "health-failover-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "3",
            "--automatic-health-failover",
            "--cold-health-failover",
            "--diagnostic",
            "--experiment-id",
            "health-failover-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        server_status.success(),
        "stdout={server_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !client_log.contains("carrier_event name=controlled_udp_stop"),
        "{client_log}"
    );
    assert!(
        client_log.contains("controlled_udp_stop=false"),
        "{client_log}"
    );
    assert!(client_log.contains("carrier_event name=udp_health_failed session=7001 generation=1 threshold=3 reason=udp_path_degraded diagnostic_cause=authenticated_delivery_ack_timeout"), "{client_log}");
    assert_eq!(
        client_log.matches("\"event\":\"udp_health_event\"").count(),
        3
    );
    assert!(
        client_log.contains("\"state\":\"degraded\""),
        "{client_log}"
    );
    assert!(client_log.contains("\"state\":\"failed\""), "{client_log}");
    assert!(!client_log.contains("\"rtt_us\""), "{client_log}");
    assert!(!client_log.contains("\"loss_per_mille\""), "{client_log}");
    assert!(
        client_log.contains("\"fallback_class\":\"cold\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"promotion_gate\":\"cold_authenticated_resume\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"cold_promotion_ready_us\":"),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"readiness_satisfied_us\":"),
        "cold fallback must not advertise D064/warm readiness: {client_log}"
    );
    assert!(
        client_log.contains("carrier_event name=tcp_resume_guard"),
        "{client_log}"
    );
    assert_eq!(
        client_log
            .matches("\"event\":\"tcp_delivery_ack_validated\"")
            .count(),
        2
    );
    assert!(
        client_log.contains("failover_mode=automatic_health_failure"),
        "{client_log}"
    );
    assert!(
        server_log.contains("\"event\":\"udp_reply_ceased\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("carrier_event name=tcp_resumed"),
        "{server_log}"
    );
    assert!(
        server_log.contains("records=3 application_bytes_total=48"),
        "{server_log}"
    );
    assert!(
        server_log.contains(&format!("bytes_hex={}", "78".repeat(48))),
        "{server_log}"
    );
}
#[test]
fn reliable_udp_failover_settles_packet_acks_to_zero_in_flight() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-failover-server");
    let cp = tmp("r9-failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "8",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-server-test-01",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "6",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-client-test-01",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(server_status.success(), "stdout={server_log}");
    // R9-2: reliable-UDP packet ACKs retire all in-flight to zero; Session
    // DeliveryAck stays a separate logical confirmation; packet ACK is emitted
    // as authenticated Carrier feedback on the server side.
    assert!(
        client_log.contains("\"event\":\"r9_udp_in_flight_settled\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    // M-R9-004: genuinely multi-record — record 1 (offset 16) also rides the
    // reliable-UDP runtime AND is NOT also sent on the legacy untracked
    // udp_uncertain_range_sent path (no double owner for one Session range).
    assert!(
        client_log.contains("\"event\":\"r9_udp_record_sent\""),
        "{client_log}"
    );
    assert!(client_log.contains("\"offset\":16"), "{client_log}");
    // Record 1 receives its own independent Session DeliveryAck.
    assert!(
        client_log.contains("\"event\":\"r9_udp_delivery_ack_validated\""),
        "{client_log}"
    );
    // Under --reliable-udp records[1] (offset 16) is reliable-owned, so the
    // uncertain direct-send must start at records[2] (offset 32), not offset 16.
    assert!(
        client_log.contains("\"event\":\"udp_uncertain_range_sent\",\"seq\":2,\"ciphertext_bytes\":96,\"stream\":1,\"offset\":32"),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"udp_uncertain_range_sent\",\"seq\":2,\"ciphertext_bytes\":96,\"stream\":1,\"offset\":16"),
        "{client_log}"
    );
    assert!(
        server_log.contains("\"event\":\"udp_packet_ack_sent\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("\"event\":\"udp_delivery_ack_sent\""),
        "{server_log}"
    );
}
#[test]
fn reliable_udp_incomplete_settlement_fails_not_settled() {
    // H-R9-010: a reliable-UDP run whose Carrier settlement ends with
    // remaining in-flight must emit r9_udp_settlement_incomplete and fail —
    // never emit r9_udp_in_flight_settled nor continue into health/failover.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-incomplete-server");
    let cp = tmp("r9-incomplete-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    // Server authenticates and answers Session DeliveryAck but withholds every
    // Carrier packet ACK — the client's reliable in-flight never drains.
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "8",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--suppress-r9-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-incomplete-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "3",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-incomplete-cli",
        ])
        .output()
        .unwrap();
    let _ = server.child.kill();
    let _ = server.child.wait();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // The operation must FAIL (nonzero) and emit the incomplete marker; it must
    // NOT emit the successful settled marker.
    assert!(
        !out.status.success(),
        "expected nonzero exit, stdout={client_log} stderr={client_err}"
    );
    assert!(
        client_log.contains("r9_udp_settlement_incomplete")
            || client_err.contains("settlement incomplete")
            || client_log.contains("settlement_incomplete"),
        "{client_log} {client_err}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_in_flight_settled\""),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_migration_back_reserves_final_record() {
    // M-R9-008 (H-R9-005): with --reliable-udp + --migration-back the reserved
    // final logical record is neither Recovery-tracked nor sent on the legacy
    // udp_uncertain_range_sent path before post-promotion return-to-UDP.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-mig-server");
    let cp = tmp("r9-mig-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-mig-srv-01",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "8",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-mig-cli-01",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    // Both processes must succeed for the positive P2 evidence chain.
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(server_status.success(), "stdout={server_log}");
    // Server-side causal chain for the post-return ownership transition:
    // recovery owner started -> recovery validated -> Session DeliveryAck sent
    // -> Carrier packet ACK sent for the post-return record.
    // M-R9-009/010: server causal milestones in strict observed order —
    // recovery owner starts, the challenge is validated, then Session
    // DeliveryAck is sent, then the Carrier packet ACK for the post-return
    // record is emitted.
    let owner_pos = server_log
        .find("udp_recovery_owner_started")
        .unwrap_or(usize::MAX);
    let srv_val_pos = server_log
        .find("\"event\":\"udp_recovery_validated\"")
        .unwrap_or(usize::MAX);
    let srv_dack_pos = server_log
        .find("\"event\":\"udp_return_delivery_ack_sent\"")
        .unwrap_or(usize::MAX);
    let srv_pack_pos = server_log
        .find("\"event\":\"udp_return_packet_ack_sent\"")
        .unwrap_or(usize::MAX);
    assert!(owner_pos < srv_val_pos, "{server_log}");
    assert!(srv_val_pos < srv_dack_pos, "{server_log}");
    assert!(srv_dack_pos < srv_pack_pos, "{server_log}");
    // C3: server post-return events are exactly one each and bound to the
    // offset-48 reserved record with full stream/offset/len contract.
    let srv_dack: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_delivery_ack_sent\""))
        .collect();
    assert_eq!(srv_dack.len(), 1, "{server_log}");
    assert!(
        srv_dack[0].contains("\"stream\":1")
            && srv_dack[0].contains("\"offset\":48")
            && srv_dack[0].contains("\"len\":16"),
        "{server_log}"
    );
    let srv_pack: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert_eq!(srv_pack.len(), 1, "{server_log}");
    // Carrier-domain identity is the packet number — bind the server's
    // packet_number to the client's Recovery packet_number exactly.
    let client_pn = client_log
        .lines()
        .find(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .and_then(|l| {
            l.split("\"packet_number\":").nth(1).and_then(|v| {
                v.trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u64>()
                    .ok()
            })
        })
        .unwrap_or(u64::MAX);
    let server_pn = srv_pack[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    assert_eq!(
        client_pn, server_pn,
        "pn mismatch {client_log} {server_log}"
    );
    // Client recovery order: challenge sent before validated, validated before
    // migration-back. C2: exactly once each, strict order.
    let chal_lines: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_recovery_challenge_sent\""))
        .collect();
    assert_eq!(chal_lines.len(), 1, "{client_log}");
    let val_lines: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_recovery_validated\""))
        .collect();
    assert_eq!(val_lines.len(), 1, "{client_log}");
    let mig_lines: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_migrated_back\""))
        .collect();
    assert_eq!(mig_lines.len(), 1, "{client_log}");
    let cli_chal = client_log
        .find("\"event\":\"udp_recovery_challenge_sent\"")
        .unwrap_or(usize::MAX);
    let cli_val = client_log
        .find("\"event\":\"udp_recovery_validated\"")
        .unwrap_or(usize::MAX);
    let cli_mig = client_log
        .find("\"event\":\"udp_migrated_back\"")
        .unwrap_or(usize::MAX);
    assert!(cli_chal < cli_val, "{client_log}");
    assert!(cli_val < cli_mig, "{client_log}");
    // H-R9-020: final failover accounting must reflect the true ownership
    // partition — 2 UDP-confirmed (reliable 0,16), 1 uncertain TCP replay
    // (offset 32), 1 post-return reliable (offset 48), 4 confirmed total.
    assert!(
        client_log.contains(
            "\"udp_confirmed_records\":2,\"udp_confirmed_bytes\":32,\"uncertain_records\":1,\"uncertain_bytes\":16,\"replayed_records\":1,\"replayed_bytes\":16,\"confirmed_records\":4,\"confirmed_bytes\":64"
        ),
        "{client_log}"
    );
    // The reserved final record must NOT appear on the legacy uncertain
    // direct-send path in reliable mode (it is reliable-owned, not uncertain).
    // udp_uncertain_range_sent may still appear for the non-reserved middle
    // record; assert only that no uncertain send carries the reserved offset.
    // With 4 records of 16 bytes: offsets are 0, 16, 32, 48; reserved is index 3
    // (offset 48). Reliable records are 0 and 16; uncertain TCP replay is 32.
    assert!(
        !client_log.contains("\"event\":\"udp_uncertain_range_sent\",\"seq\":3,\"ciphertext_bytes\":96,\"stream\":1,\"offset\":48"),
        "{client_log}"
    );
    // M-R9-008 P2 (positive evidence): the post-return path MUST execute —
    // the reserved record is reliable-owned, sent via r9_udp_post_return_sent,
    // confirmed by Session DeliveryAck AND Carrier packet ACK, settling
    // recovery in_flight to zero. This is no longer conditional.
    {
        assert!(out.status.success(), "stdout={client_log}");
        assert!(
            client_log.contains("\"event\":\"udp_recovery_validated\""),
            "{client_log}"
        );
        assert!(
            client_log.contains("\"event\":\"udp_migrated_back\""),
            "{client_log}"
        );
        // C2: exact cardinality — exactly one post-return send at offset 48,
        // and migrated_back strictly before it. The send must carry a
        // packet_number field for the cross-process binding below.
        let post_sends: Vec<&str> = client_log
            .lines()
            .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
            .collect();
        assert_eq!(post_sends.len(), 1, "{client_log}");
        assert!(post_sends[0].contains("\"packet_number\":"), "{client_log}");
        let mig_pos = client_log
            .find("\"event\":\"udp_migrated_back\"")
            .unwrap_or(usize::MAX);
        let send_pos = client_log
            .find("\"event\":\"r9_udp_post_return_sent\"")
            .unwrap_or(usize::MAX);
        assert!(mig_pos < send_pos, "{client_log}");
        // C2: offset 48 must be absent from pre-promotion reliable record sends
        // (r9_udp_record_sent covers offsets 0 and 16 only).
        assert!(
            !client_log.contains("\"event\":\"r9_udp_record_sent\",\"seq\":0,\"offset\":48"),
            "{client_log}"
        );
        assert!(
            client_log.contains(
                "\"event\":\"r9_udp_post_return_sent\",\"seq\":0,\"stream\":1,\"offset\":48"
            ),
            "{client_log}"
        );
        // C4: exactly one Session DeliveryAck AND exactly one Carrier packet
        // ACK, each bound to stream 1 / offset 48 — the dual-domain proof.
        let dack_lines: Vec<&str> = client_log
            .lines()
            .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
            .collect();
        assert_eq!(dack_lines.len(), 1, "{client_log}");
        assert!(
            dack_lines[0].contains("\"stream\":1")
                && dack_lines[0].contains("\"offset\":48")
                && dack_lines[0].contains("\"len\":16"),
            "{client_log}"
        );
        let pack_lines: Vec<&str> = client_log
            .lines()
            .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
            .collect();
        assert_eq!(pack_lines.len(), 1, "{client_log}");
        assert!(
            pack_lines[0].contains("\"applied\":true")
                && pack_lines[0].contains("\"retired\":true"),
            "{client_log}"
        );
        // C4 three-way packet bind: client send pn == server ACK pn == client
        // actual Recovery retirement pn.
        let retire_pn = pack_lines[0]
            .split("\"packet_number\":")
            .nth(1)
            .and_then(|v| {
                v.trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(u64::MAX);
        assert_eq!(retire_pn, client_pn, "{client_log}");
        // C4 negatives: no rejected and no accepted-empty on the ordinary
        // positive post-return path.
        assert!(
            !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\"")
                && !client_log.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""),
            "{client_log}"
        );
        // Dedicated post-return terminal evidence: exact stream/offset and
        // authoritative post-return remaining_in_flight=0. H-R9-035: exactly
        // one settled event — not merely "contains".
        let settled: Vec<&str> = client_log
            .lines()
            .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
            .collect();
        assert_eq!(settled.len(), 1, "{client_log}");
        assert!(
            settled[0].contains("\"stream\":1")
                && settled[0].contains("\"offset\":48")
                && settled[0].contains("\"remaining_in_flight\":0"),
            "{client_log}"
        );
    }
    // M-R9-008 P2-C1: TCP replay identity — exactly one tcp_delivery_ack_validated
    // at seq 2 (offset 32), stream 1; no replay of reliable-owned 0/16 or
    // reserved 48. Server must also emit exactly one tcp_delivery_ack_sent
    // for seq 2 / offset 32 / stream 1, and no TCP evidence for wrong offsets.
    let tcp_acks: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"tcp_delivery_ack_validated\""))
        .collect();
    assert_eq!(tcp_acks.len(), 1, "{client_log}");
    assert!(
        tcp_acks[0].contains("\"stream\":1")
            && tcp_acks[0].contains("\"offset\":32")
            && tcp_acks[0].contains("\"seq\":2"),
        "{client_log}"
    );
    let srv_tcp_acks: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"tcp_delivery_ack_sent\""))
        .collect();
    assert_eq!(srv_tcp_acks.len(), 1, "{server_log}");
    assert!(
        srv_tcp_acks[0].contains("\"stream\":1")
            && srv_tcp_acks[0].contains("\"offset\":32")
            && srv_tcp_acks[0].contains("\"seq\":2"),
        "{server_log}"
    );
    // No replay evidence for reliable-owned 0/16 or reserved 48 on TCP —
    // check BOTH client and server sides.
    for off in ["\"offset\":0", "\"offset\":16", "\"offset\":48"] {
        assert!(
            !client_log
                .lines()
                .any(|l| l.contains("tcp_delivery_ack") && l.contains(off)),
            "{client_log}"
        );
        assert!(
            !server_log
                .lines()
                .any(|l| l.contains("tcp_delivery_ack") && l.contains(off)),
            "{server_log}"
        );
    }
    // P2-C4: settled event must appear strictly after both actual ACK-domain
    // transitions — the mutation event r9_udp_return_delivery_ack and the
    // Carrier event r9_udp_return_packet_ack — not the post-loop summary.
    let dack_pos = client_log
        .find("\"event\":\"r9_udp_return_delivery_ack\"")
        .unwrap_or(usize::MAX);
    let pack_pos = client_log
        .find("\"event\":\"r9_udp_return_packet_ack\"")
        .unwrap_or(usize::MAX);
    let settled_pos = client_log
        .find("\"event\":\"r9_udp_post_return_settled\"")
        .unwrap_or(usize::MAX);
    assert!(dack_pos < settled_pos, "{client_log}");
    assert!(pack_pos < settled_pos, "{client_log}");
}
#[test]
fn reliable_udp_reversed_ack_order_confirms_in_order() {
    // M-R9-008 P1: with --reverse-ack-order the server emits record-1's Session
    // DeliveryAck before record-0's. The client must buffer record-1, apply
    // record-0 first on arrival, then drain buffered record-1 — evidence order
    // must match mutation order (offset 0 applied before offset 16).
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-rev-server");
    let cp = tmp("r9-rev-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--reverse-ack-order",
            "--diagnostic",
            "--experiment-id",
            "r9-rev-srv-01",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "8",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-rev-cli-01",
        ])
        .output()
        .unwrap();
    let (_st, _sl) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    // P1 acceptance: the client process itself completes successfully on the
    // reversed-order reliable path.
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Record-1 ACK is observed/buffered first while the watermark is still 0.
    let buffered: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_delivery_ack_buffered\""))
        .collect();
    assert_eq!(buffered.len(), 1, "{client_log}");
    assert!(
        buffered[0].contains("\"offset\":16") && buffered[0].contains("\"watermark\":0"),
        "{client_log}"
    );
    // No covered-shortcut may substitute for the two real confirmations.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_delivery_ack_covered\""),
        "{client_log}"
    );
    // Both exact offsets appear as applied confirmations.
    assert!(client_log.contains("\"offset\":0"), "{client_log}");
    assert!(client_log.contains("\"offset\":16"), "{client_log}");
    // Order proof uses the exact structured event lines already collected:
    // the buffered-observation line appears before the first applied event,
    // and the buffered applied event (buffered=true) appears last.
    let buf_pos = client_log
        .find("\"event\":\"r9_udp_delivery_ack_buffered\"")
        .unwrap_or(0);
    let app0_pos = client_log
        .find("\"event\":\"r9_udp_delivery_ack_validated\"")
        .unwrap_or(usize::MAX);
    let app16_pos = client_log.rfind("\"buffered\":true").unwrap_or(usize::MAX);
    assert!(buf_pos < app0_pos, "{client_log}");
    assert!(app0_pos < app16_pos, "{client_log}");
    // H-R9-013/014: exactly two applied r9_udp_delivery_ack_validated events —
    // stream 1 / offset 0 (non-buffered, applied first) and stream 1 /
    // offset 16 buffered=true (applied second after the watermark advances).
    let applied_events: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_delivery_ack_validated\""))
        .collect();
    assert_eq!(applied_events.len(), 2, "{client_log}");
    assert!(
        applied_events[0].contains("\"stream\":1")
            && applied_events[0].contains("\"offset\":0")
            && !applied_events[0].contains("\"buffered\":true"),
        "{client_log}"
    );
    assert!(
        applied_events[1].contains("\"stream\":1")
            && applied_events[1].contains("\"offset\":16")
            && applied_events[1].contains("\"buffered\":true"),
        "{client_log}"
    );
    // In-flight settles to zero through the terminal settlement event (not a
    // broad substring — the settled marker itself carries the authoritative
    // remaining_in_flight=0).
    assert!(
        client_log
            .contains("\"event\":\"r9_udp_in_flight_settled\",\"seq\":0,\"remaining_in_flight\":0"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_malformed_budget_persists_across_carrier_ack() {
    // M-R9-008 P3: the operation-wide malformed budget persists across a valid
    // Carrier/Session ACK — malformed #1 and #2 arrive, a valid ACK lands
    // between them, and malformed #3 must hit MAX_POST_HANDSHAKE_MALFORMED
    // rather than the budget resetting after valid feedback.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-malf-server");
    let cp = tmp("r9-malf-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--malformed-budget-test",
            "--diagnostic",
            "--experiment-id",
            "r9-malf-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-malf-cli",
        ])
        .output()
        .unwrap();
    let (_st, _sl) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // The malformed budget must terminate the operation — the third malformed
    // hits the bound and the process reports a typed failure rather than
    // spinning or succeeding.
    // H-R9-022/023: exactly one applied Carrier ACK while the malformed
    // counter was already at 2 (between malformed #2 and #3), and no rejected
    // packet-ACK event for this fixture.
    let applied_events: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_applied\""))
        .collect();
    assert_eq!(applied_events.len(), 1, "{client_log}");
    assert!(
        applied_events[0].contains("\"malformed\":2"),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_packet_ack_rejected\""),
        "{client_log}"
    );
    // The operation must terminate typed on the exact malformed bound —
    // unconditional nonzero exit and the existing terminal error text.
    assert!(
        !out.status.success(),
        "expected nonzero exit, stdout={client_log} stderr={client_err}"
    );
    assert!(
        client_err.contains("UDP delivery acknowledgement malformed bound exceeded"),
        "{client_err}"
    );
    // No successful settlement markers and no downstream health/failover
    // continuation after the malformed bound terminates the operation.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_in_flight_settled\""),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_settled\""),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"udp_health_event\"")
            && !client_log.contains("tcp_warm")
            && !client_log.contains("udp_migrated_back"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_post_return_incomplete_is_terminal() {
    // M-R9-008 P4: under --reliable-udp + --migration-back + --suppress-r9-dack
    // the post-return Session DeliveryAck never arrives — logical ownership
    // stays outstanding even though the Carrier packet ACK lands. The client
    // must terminate typed/nonzero, never emit r9_udp_post_return_settled or
    // r9_udp_in_flight_settled, and must not continue health/failover.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-p4-server");
    let cp = tmp("r9-p4-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--migration-back",
            "--suppress-r9-dack",
            "--diagnostic",
            "--experiment-id",
            "r9-p4-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-p4-cli",
        ])
        .output()
        .unwrap();
    let (_st, _sl) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // The post-return logical confirmation is suppressed — the client must
    // fail typed, never emit a successful post-return/settled marker, and
    // never continue as if settlement completed.
    assert!(
        !out.status.success(),
        "expected nonzero exit, stdout={client_log} stderr={client_err}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_settled\""),
        "{client_log}"
    );
    assert!(
        client_err.contains("post-return")
            || client_err.contains("acknowledgement")
            || client_log.contains("settlement_incomplete")
            || client_err.contains("settlement"),
        "{client_log} {client_err}"
    );
}
#[test]
fn reliable_udp_stale_ack_is_accepted_empty_not_rejected() {
    // H-R9-026: a duplicate/stale canonical Carrier ACK is accepted by
    // Recovery with an empty outcome — it must not be counted or emitted as
    // a rejection, and must not produce a positive packet transition.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-stale-server");
    let cp = tmp("r9-stale-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--send-stale-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-stale-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-stale-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // H-R9-028: both processes must succeed.
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Exactly one accepted-empty classification for the one-shot injected
    // duplicate, zero rejection on the initial reliable path.
    // Exactly one accepted-empty classification proves the injected duplicate
    // reached the shared Carrier owner as a non-event — absent injection would
    // produce zero. The two real sends still produce applied events.
    let empty: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_accepted_empty\""))
        .collect();
    let applied: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_applied\""))
        .collect();
    let rejected: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_rejected\""))
        .collect();
    assert_eq!(empty.len(), 1, "{client_log}");
    assert_eq!(applied.len(), 2, "{client_log}");
    // The duplicate is consumed silently by the shared demux owner — no
    // applied, no rejected, no extra event beyond the two real sends.
    assert_eq!(
        rejected.len(),
        0,
        "expected zero r9_udp_packet_ack_rejected: {client_log}"
    );
    // Exactly two reliable-path Session confirmations (records 0,1) — the
    // uncertain TCP replays confirm through tcp_delivery_ack_validated instead.
    // The injected duplicate must not add or remove a logical confirmation.
    let dack: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_delivery_ack_validated\""))
        .collect();
    assert_eq!(dack.len(), 2, "{client_log}");
    assert!(dack[0].contains("\"offset\":0"), "{client_log}");
    assert!(dack[1].contains("\"offset\":16"), "{client_log}");
    // Final Recovery zero via the settlement marker.
    assert!(
        client_log
            .contains("\"event\":\"r9_udp_in_flight_settled\",\"seq\":0,\"remaining_in_flight\":0"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_future_ack_is_typed_rejected() {
    // H-R9-026: a canonical ACK whose largest exceeds largest_sent is a typed
    // rejection — it must emit r9_udp_packet_ack_rejected and must not mutate
    // recovery state or produce a positive transition.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-fut-server");
    let cp = tmp("r9-fut-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--send-future-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-fut-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-fut-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // H-R9-028: both processes must succeed.
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Exactly one typed rejection for the one-shot injected future ACK; zero
    // accepted-empty mis-classification on the initial reliable path.
    let rejected: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_rejected\""))
        .collect();
    assert_eq!(rejected.len(), 1, "{client_log}");
    // The injected future ACK must not manufacture a positive retirement.
    // Exactly two real applied Carrier ACKs (records 0 and 1) still land.
    let applied: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_applied\""))
        .collect();
    assert_eq!(applied.len(), 2, "{client_log}");
    // Unchanged Session logical completion: exactly two reliable-path
    // confirmations (records 0,1); uncertain replays confirm via TCP.
    let dack: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_delivery_ack_validated\""))
        .collect();
    assert_eq!(dack.len(), 2, "{client_log}");
    assert!(dack[0].contains("\"offset\":0"), "{client_log}");
    assert!(dack[1].contains("\"offset\":16"), "{client_log}");
    // Final Recovery zero via the settlement marker.
    assert!(
        client_log
            .contains("\"event\":\"r9_udp_in_flight_settled\",\"seq\":0,\"remaining_in_flight\":0"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_stale_ack_settlement_phase_is_accepted_empty() {
    // H-R9-034: --send-stale-ack-late injects the semantic duplicate after the
    // last reliable record's Session ACK, so the client's settlement
    // continuation (not the initial loop) consumes and classifies it.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-stlate-server");
    let cp = tmp("r9-stlate-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--send-stale-ack-late",
            "--diagnostic",
            "--experiment-id",
            "r9-stlate-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--diagnostic",
            "--experiment-id",
            "r9-stlate-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    // H-R9-034: both processes must succeed.
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Exactly one accepted-empty classification — the duplicate must reach the
    // settlement continuation, not the initial loop.
    let empty: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_accepted_empty\""))
        .collect();
    assert_eq!(empty.len(), 1, "{client_log}");
    // Two real applied Carrier ACKs (records 0 and 1).
    let applied: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_packet_ack_applied\""))
        .collect();
    assert_eq!(applied.len(), 2, "{client_log}");
    // Zero typed rejection.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_packet_ack_rejected\""),
        "{client_log}"
    );
    // Exactly two reliable-path Session confirmations (records 0,1).
    let dack: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_delivery_ack_validated\""))
        .collect();
    assert_eq!(dack.len(), 2, "{client_log}");
    assert!(dack[0].contains("\"offset\":0"), "{client_log}");
    assert!(dack[1].contains("\"offset\":16"), "{client_log}");
    // The accepted-empty event must appear AFTER both Session confirmations —
    // proving it was consumed by the settlement continuation, not the initial
    // loop.
    let empty_pos = client_log
        .find("\"event\":\"r9_udp_packet_ack_accepted_empty\"")
        .unwrap_or(usize::MAX);
    let dack0_pos = client_log
        .find("\"event\":\"r9_udp_delivery_ack_validated\"")
        .unwrap_or(usize::MAX);
    let dack1_pos = client_log
        .rfind("\"event\":\"r9_udp_delivery_ack_validated\"")
        .unwrap_or(usize::MAX);
    assert!(
        dack0_pos < empty_pos && dack1_pos < empty_pos,
        "accepted-empty must appear after both Session confirmations: {client_log}"
    );
    // Final Recovery zero via the settlement marker.
    assert!(
        client_log
            .contains("\"event\":\"r9_udp_in_flight_settled\",\"seq\":0,\"remaining_in_flight\":0"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_post_return_carrier_ack_withheld_fails() {
    // READY_LOCAL 1 P4: Session DeliveryAck arrives but the post-return
    // Carrier packet ACK is withheld. Client must exit typed/nonzero with
    // Session complete but Recovery still nonzero — no settled premise.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-p4a-server");
    let cp = tmp("r9-p4a-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--suppress-r9-post-return-pack",
            "--diagnostic",
            "--experiment-id",
            "r9-p4a-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-p4a-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(srv_status.success(), "{server_log}");
    // Exactly one positive client Session transition stream=1 offset=48 len=16.
    let dack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack_ev.len(), 1, "{client_log}");
    assert!(
        dack_ev[0].contains("\"stream\":1")
            && dack_ev[0].contains("\"offset\":48")
            && dack_ev[0].contains("\"len\":16"),
        "{client_log}"
    );
    // Exactly one client post-return send (identity for the withheld domain).
    let send_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(send_ev.len(), 1, "{client_log}");
    // Zero positive Carrier transition for the post-return packet.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack\""),
        "{client_log}"
    );
    // Zero rejected/accepted-empty substitute on the withheld-Carrier path.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\"")
            && !client_log.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""),
        "{client_log}"
    );
    // Residual evidence: Session complete, Recovery still nonzero. The
    // bounded PTO/recovery loop keeps running until the operation deadline,
    // so the terminal residual is the LAST diagnostic, not the only one.
    assert!(
        client_log.contains("\"event\":\"r9_udp_post_return_residual\""),
        "{client_log}"
    );
    let residual: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_residual\""))
        .collect();
    let terminal = residual.last().expect("residual diagnostics");
    assert!(
        terminal.contains("\"session_outstanding\":0"),
        "{client_log}"
    );
    assert!(
        !terminal.contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    // Client exits nonzero on the bounded post-return deadline — no settled
    // premise, no Carrier transition.
    assert!(!out.status.success(), "{client_log} {client_err}");
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_settled\""),
        "{client_log}"
    );
    // Server proved the post-return owner was reached and sent the Session
    // ACK but no Carrier packet ACK. The bounded recovery loop may retransmit
    // the same Session ACK — every emission must bind the same exact range.
    let srv_dack: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_delivery_ack_sent\""))
        .collect();
    assert!(!srv_dack.is_empty(), "{server_log}");
    assert!(
        srv_dack.iter().all(|l| l.contains("\"stream\":1")
            && l.contains("\"offset\":48")
            && l.contains("\"len\":16")),
        "{server_log}"
    );
    assert!(
        !server_log.contains("\"event\":\"udp_return_packet_ack_sent\""),
        "{server_log}"
    );
}
#[test]
fn reliable_udp_post_return_session_ack_withheld_fails() {
    // READY_LOCAL 2 P4: post-return Carrier packet ACK arrives but the Session
    // DeliveryAck is withheld. Client must exit typed/nonzero — Recovery may
    // settle but Session stays outstanding, no settled premise.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-p4b-server");
    let cp = tmp("r9-p4b-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--suppress-r9-dack",
            "--diagnostic",
            "--experiment-id",
            "r9-p4b-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-p4b-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    // H-R9-039: server must also succeed — the suppressed Session ACK is a
    // deliberate fault injection, not a server lifecycle failure.
    assert!(srv_status.success(), "{server_log}");
    // Carrier packet ACK arrived and retired the packet; Session ACK withheld.
    // Exactly one positive Carrier retirement; zero Session transition.
    let pack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
        .collect();
    assert_eq!(pack_ev.len(), 1, "{client_log}");
    assert!(
        pack_ev[0].contains("\"applied\":true") && pack_ev[0].contains("\"retired\":true"),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_delivery_ack\""),
        "{client_log}"
    );
    // Residual: Recovery settled but Session still outstanding. The bounded
    // PTO loop keeps running until the operation deadline — the terminal
    // residual is the LAST diagnostic.
    let residual: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_residual\""))
        .collect();
    let terminal = residual.last().expect("residual diagnostics");
    assert!(
        terminal.contains("\"remaining_in_flight\":0")
            && terminal.contains("\"session_outstanding\":1"),
        "{client_log}"
    );
    // Three-way packet bind: client send == server ACK == client retire.
    let send_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(send_ev.len(), 1, "{client_log}");
    let client_pn = send_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    let srv_pack_ev: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert_eq!(srv_pack_ev.len(), 1, "{server_log}");
    let server_pn = srv_pack_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    let retire_pn = pack_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    assert_eq!(client_pn, server_pn, "{client_log} {server_log}");
    assert_eq!(client_pn, retire_pn, "{client_log}");
    // Zero rejected/accepted-empty on this ordinary positive path.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\"")
            && !client_log.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""),
        "{client_log}"
    );
    // Typed nonzero exit on the bounded deadline — no settled premise, no
    // downstream success continuation.
    assert!(!out.status.success(), "{client_log}");
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_settled\""),
        "{client_log}"
    );
    // Server sent the Carrier ACK but no Session DeliveryAck.
    assert!(
        server_log.contains("\"event\":\"udp_return_packet_ack_sent\""),
        "{server_log}"
    );
    assert!(
        !server_log.contains("\"event\":\"udp_return_delivery_ack_sent\""),
        "{server_log}"
    );
}
#[test]
fn reliable_udp_post_return_data_loss_recovers_via_pto_retransmit() {
    // R9-3: suppress the post-return Data after congestion admission and
    // Recovery ownership commit. PTO must fire, retransmit with a fresh packet
    // number/nonce but stable frame/logical identity, and the run must settle.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-dl-server");
    let cp = tmp("r9-dl-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "15",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-dl-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--drop-r9-data",
            "--diagnostic",
            "--experiment-id",
            "r9-dl-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // The suppressed original packet was recovery-owned before the drop.
    let send_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(send_ev.len(), 1, "{client_log}");
    let orig_pn = send_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    // PTO fired at or after the computed deadline — timing evidence. Bounded
    // repeated probes are legal before the first positive Carrier retirement
    // (H-R9-043): a later PTO may become due while the overlap lifecycle is
    // still outstanding. We require at least one, all fired at/after their
    // deadline.
    let pto_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_pto_fired\""))
        .collect();
    assert!(!pto_ev.is_empty(), "{client_log}");
    // H-R9-044: EVERY PTO event must fire at/after its own deadline, not just
    // the first. Parse and check each one.
    for e in &pto_ev {
        let d = e
            .split("\"deadline_us\":")
            .nth(1)
            .and_then(|v| {
                v.trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(u64::MAX);
        let f = e
            .split("\"fired_at_us\":")
            .nth(1)
            .and_then(|v| {
                v.trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(u64::MAX);
        assert!(f >= d, "{client_log}");
    }
    // H-R9-045: parse EVERY retransmit — each packet number must be fresh
    // relative to the original, pairwise distinct across siblings, and carry
    // the same stable frame/logical identity (frame=48).
    let re_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_retransmit_sent\""))
        .collect();
    assert!(!re_ev.is_empty(), "{client_log}");
    let mut re_pns = std::collections::BTreeSet::new();
    for e in &re_ev {
        let pn = e
            .split("\"packet_number\":")
            .nth(1)
            .and_then(|v| {
                v.trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(u64::MAX);
        assert_ne!(pn, u64::MAX, "{client_log}");
        assert_ne!(pn, orig_pn, "{client_log}");
        assert!(re_pns.insert(pn), "duplicate retransmit pn {client_log}");
        assert!(e.contains("\"frame\":48"), "{client_log}");
    }
    // Exactly one Session transition for the post-return range.
    let dack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack_ev.len(), 1, "{client_log}");
    assert!(
        dack_ev[0].contains("\"stream\":1")
            && dack_ev[0].contains("\"offset\":48")
            && dack_ev[0].contains("\"len\":16"),
        "{client_log}"
    );
    // Positive Carrier retirement(s) for retransmitted packet(s) — at least
    // one; every retire event retires a real retransmit packet identity.
    let pack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
        .collect();
    assert!(!pack_ev.is_empty(), "{client_log}");
    assert!(
        pack_ev
            .iter()
            .all(|l| l.contains("\"applied\":true") && l.contains("\"retired\":true")),
        "{client_log}"
    );
    // Bind by VALUE, not by position: the packet that receives positive
    // Carrier retirement must be a real client retransmit, and the server
    // must have sent a Carrier ACK for that same packet number.
    let retire_pns: Vec<u64> = pack_ev
        .iter()
        .map(|l| {
            l.split("\"packet_number\":")
                .nth(1)
                .and_then(|v| {
                    v.trim_end_matches(|c: char| !c.is_ascii_digit())
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(u64::MAX)
        })
        .collect();
    for rp in &retire_pns {
        assert!(
            re_pns.contains(rp),
            "retired pn {rp} not a client retransmit {client_log}"
        );
    }
    // H-R9-042: server Carrier ACK packet numbers bound to the same
    // retransmitted identities — every retired packet has a matching server ACK.
    let srv_ack_ev: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert!(!srv_ack_ev.is_empty(), "{server_log}");
    let srv_ack_pns: Vec<u64> = srv_ack_ev
        .iter()
        .map(|l| {
            l.split("\"packet_number\":")
                .nth(1)
                .and_then(|v| {
                    v.trim_end_matches(|c: char| !c.is_ascii_digit())
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(u64::MAX)
        })
        .collect();
    // Every positively-retired client retransmit has a matching server Carrier
    // ACK for that exact packet number — bind by value, not by log position.
    for rp in &retire_pns {
        assert!(
            srv_ack_pns.contains(rp),
            "retired pn {rp} has no server Carrier ACK {server_log}"
        );
    }
    // H-R9-044: typed rejection stays zero, but a sibling/late Carrier ACK
    // may legitimately classify accepted-empty under repeated PTO — it is a
    // classification-only outcome that must not create a second Session
    // transition or a new ownership transition.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\""),
        "{client_log}"
    );
    // H-R9-045 order/terminal: identify the first lifecycle-resolving positive
    // Carrier retirement event — a pack_ev line whose packet number is in the
    // client retransmit set (the value-bind above). The final settlement must
    // occur AFTER both the exact Session confirmation and that retirement.
    let resolve_pos = client_log
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
        .filter(|(_, l)| {
            let pn = l
                .split("\"packet_number\":")
                .nth(1)
                .and_then(|v| {
                    v.trim_end_matches(|c: char| !c.is_ascii_digit())
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(u64::MAX);
            re_pns.contains(&pn)
        })
        .map(|(i, _)| i)
        .next()
        .expect("lifecycle-resolving Carrier retirement present");
    // H-R9-046: every retransmit diagnostic occurs strictly before the
    // lifecycle-resolving retirement — after the frame is positively ACKed,
    // no further retransmit is emitted for it.
    for (i, l) in client_log.lines().enumerate() {
        if l.contains("\"event\":\"r9_udp_retransmit_sent\"") {
            assert!(
                i < resolve_pos,
                "post-retirement retransmit at line {i} {client_log}"
            );
        }
    }
    let dack_pos = client_log
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .map(|(i, _)| i)
        .expect("Session transition present");
    let settled_pos = client_log
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .map(|(i, _)| i)
        .expect("settled event present");
    assert!(settled_pos > dack_pos, "{client_log}");
    assert!(settled_pos > resolve_pos, "{client_log}");
    // Exactly one settled event with remaining_in_flight=0.
    let settled: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .collect();
    assert_eq!(settled.len(), 1, "{client_log}");
    assert!(
        settled[0].contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_ack_loss_delayed_original_reorder_settles() {
    // R9-4: server withholds the FIRST post-return Carrier ACK long enough to
    // force a legitimate PTO/retransmission, then releases the delayed
    // original ACK BEFORE the fresh copy's ACK — deterministic ACK-loss +
    // delayed/sibling reorder challenging duplicate handling.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-alo-server");
    let cp = tmp("r9-alo-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "15",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--delay-r9-ack-reorder",
            "--diagnostic",
            "--experiment-id",
            "r9-alo-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-alo-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // PTO fired and a fresh retransmit went out on a fresh packet number.
    let pto_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_pto_fired\""))
        .collect();
    assert!(!pto_ev.is_empty(), "{client_log}");
    let re_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_retransmit_sent\""))
        .collect();
    assert!(!re_ev.is_empty(), "{client_log}");
    // Exactly one Session transition for the post-return range.
    let dack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack_ev.len(), 1, "{client_log}");
    // Delayed original + fresh-copy Carrier ACKs both observed by the client —
    // the delayed original is classification-only (accepted-empty or retired)
    // and must not create a second Session transition or a rejection.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\""),
        "{client_log}"
    );
    // H-R9-060: the retransmitted range's duplicate Session DeliveryAck takes
    // the repaired exact-witness accepted-empty path — classification-only
    // evidence, never malformed/unexpected.
    assert!(
        client_log.contains("\"event\":\"accepted_empty_logical_ack\""),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"unexpected_logical_ack\""),
        "{client_log}"
    );
    // Server emitted at least two post-return Carrier ACKs (delayed original +
    // fresh copy), released in the reorder (delayed first).
    let srv_ack_ev: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert!(srv_ack_ev.len() >= 2, "{server_log}");
    // H-R9-060: every positive client Carrier retirement names an actually
    // transmitted Recovery-owned identity — the server-emitted packet_number
    // set must cover every client r9_udp_return_packet_ack packet_number.
    let srv_acked_pns: std::collections::BTreeSet<String> = srv_ack_ev
        .iter()
        .filter_map(|l| {
            l.split("\"packet_number\":")
                .nth(1)
                .and_then(|v| v.split(',').next())
                .map(|v| v.trim_end_matches('}').trim().to_string())
        })
        .collect();
    let cli_retired_pns: Vec<String> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
        .filter_map(|l| {
            l.split("\"packet_number\":")
                .nth(1)
                .and_then(|v| v.split(',').next())
                .map(|v| v.trim_end_matches('}').trim().to_string())
        })
        .collect();
    assert!(
        cli_retired_pns.iter().all(|pn| srv_acked_pns.contains(pn)),
        "client retired packet_number must be in server ACK set: {cli_retired_pns:?} vs {srv_acked_pns:?} {client_log} {server_log}"
    );
    // Terminal: exactly one zero-in-flight settlement.
    let settled: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .collect();
    assert_eq!(settled.len(), 1, "{client_log}");
    assert!(
        settled[0].contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    assert!(client_log.contains("failover_client_ok"), "{client_log}");
}
#[test]
fn reliable_udp_first_send_socket_failure_rolls_back() {
    // H-R9-062: inject a socket error on the PRODUCTION post-return first-send
    // owner — positive sent evidence must not escape, Recovery ownership rolls
    // back, and the failed PN stays non-ACK-valid.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-fsf-server");
    let cp = tmp("r9-fsf-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "15",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-fsf-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--fail-r9-first-send",
            "--diagnostic",
            "--experiment-id",
            "r9-fsf-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, _server_log) = finish_server(server);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let _ = srv_status;
    // Production first-send owner emitted the rollback classification, never a
    // positive sent event for the failed packet.
    assert!(
        client_log.contains("\"event\":\"r9_udp_post_return_send_failed\""),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_sent\""),
        "{client_log}"
    );
    // No legitimate Session transition settled — the datagram never reached
    // the socket, so the client cannot fabricate a positive settlement.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_post_return_settled\""),
        "{client_log}"
    );
    // H-R9-067: the production owner must be terminal-failed — a nonzero
    // client exit, never a misclassified final success. A regression that
    // emits residual evidence but then reports success must turn red here.
    assert!(
        !out.status.success(),
        "client must not exit 0 after first-send socket failure: {client_log}"
    );
    assert!(
        !client_log.contains("\"event\":\"failover_client_ok\"")
            && !client_log.contains("\"event\":\"summary\"")
            && !client_log.contains("ordered_records_complete"),
        "no final success evidence: {client_log}"
    );
    // The residual shows zero Recovery in-flight for the aborted copy and no
    // PTO/retransmit ever fired for it.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_retransmit_sent\"")
            && !client_log.contains("\"event\":\"r9_udp_pto_fired\""),
        "{client_log}"
    );
    if let Some(residual) = client_log
        .lines()
        .find(|l| l.contains("r9_udp_post_return_residual"))
    {
        assert!(residual.contains("\"remaining_in_flight\":0"), "{residual}");
    }
}
#[test]
fn reliable_udp_post_return_reversed_ack_order_settles() {
    // READY_LOCAL 2: pure order challenge — Carrier packet ACK arrives BEFORE
    // the Session DeliveryAck on the post-return owner. Settlement still
    // requires both actual ACK-domain transitions.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-rord-server");
    let cp = tmp("r9-rord-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--reverse-post-return-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-rord-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-rord-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Client-observed order: Carrier ACK (packet transition) arrives BEFORE
    // the Session DeliveryAck transition.
    let pack_pos = client_log
        .find("\"event\":\"r9_udp_return_packet_ack\"")
        .unwrap_or(usize::MAX);
    let dack_pos = client_log
        .find("\"event\":\"r9_udp_return_delivery_ack\"")
        .unwrap_or(usize::MAX);
    assert!(pack_pos < dack_pos, "{client_log}");
    // Exact cardinality + three-way packet bind: exactly one positive Carrier
    // retirement and one Session transition, all bound to the same packet.
    let pack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack\""))
        .collect();
    assert_eq!(pack_ev.len(), 1, "{client_log}");
    assert!(
        pack_ev[0].contains("\"applied\":true") && pack_ev[0].contains("\"retired\":true"),
        "{client_log}"
    );
    let dack_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack_ev.len(), 1, "{client_log}");
    assert!(
        dack_ev[0].contains("\"stream\":1")
            && dack_ev[0].contains("\"offset\":48")
            && dack_ev[0].contains("\"len\":16"),
        "{client_log}"
    );
    // H-R9-037: true three-way packet-number equality — client send ==
    // server Carrier ACK == client retirement.
    let send_ev: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(send_ev.len(), 1, "{client_log}");
    let client_pn = send_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    let srv_pack_ev: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert_eq!(srv_pack_ev.len(), 1, "{server_log}");
    let server_pn = srv_pack_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    assert_eq!(
        client_pn, server_pn,
        "client/server pn {client_log} {server_log}"
    );
    let retire_pn = pack_ev[0]
        .split("\"packet_number\":")
        .nth(1)
        .and_then(|v| {
            v.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(u64::MAX);
    assert_eq!(client_pn, retire_pn, "{client_log}");
    // Settlement strictly after both, remaining_in_flight=0.
    let settled: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .collect();
    assert_eq!(settled.len(), 1, "{client_log}");
    assert!(
        settled[0].contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    let settled_pos = client_log
        .find("\"event\":\"r9_udp_post_return_settled\"")
        .unwrap_or(usize::MAX);
    assert!(dack_pos < settled_pos, "{client_log}");
    assert!(pack_pos < settled_pos, "{client_log}");
    // No rejected / accepted-empty on the ordinary reversed path.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\"")
            && !client_log.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""),
        "{client_log}"
    );
}
#[test]
fn reliable_udp_post_return_stale_ack_is_accepted_empty() {
    // H-R9-029/H-R9-030: after migration-back the post-return receive owner
    // classifies a re-sealed duplicate of the canonical current Carrier ACK
    // as accepted-empty — not a rejection, not a positive retirement.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-pstale-server");
    let cp = tmp("r9-pstale-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--send-stale-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-pstale-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-pstale-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Exactly one accepted-empty, zero post-return rejected.
    let empty_events: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""))
        .collect();
    assert_eq!(empty_events.len(), 1, "{client_log}");
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_rejected\""),
        "{client_log}"
    );
    // H-R9-032: cross-process packet-number binding — client send, server ACK
    // send, and client positive retirement must all reference the same pn.
    let client_sent: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(client_sent.len(), 1, "{client_log}");
    let client_pn = packet_number(client_sent[0]).expect("client packet_number");
    let server_pack: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert_eq!(server_pack.len(), 1, "{server_log}");
    let server_pn = packet_number(server_pack[0]).expect("server packet_number");
    let retire: Vec<&str> = client_log
        .lines()
        .filter(|l| {
            l.contains("\"event\":\"r9_udp_return_packet_ack\"") && l.contains("\"retired\":true")
        })
        .collect();
    assert_eq!(retire.len(), 1, "{client_log}");
    let retire_pn = packet_number(retire[0]).expect("retire packet_number");
    assert_eq!(
        client_pn, server_pn,
        "client-send vs server-ack pn mismatch"
    );
    assert_eq!(
        client_pn, retire_pn,
        "client-send vs client-retire pn mismatch"
    );
    // Exactly one Session transition stream=1 offset=48 len=16.
    let dack: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack.len(), 1, "{client_log}");
    assert!(
        dack[0].contains("\"stream\":1")
            && dack[0].contains("\"offset\":48")
            && dack[0].contains("\"len\":16"),
        "{client_log}"
    );
    // Exactly one settlement with remaining_in_flight=0, strictly after both
    // ACK-domain transitions (compare line positions, not just existence).
    let settled: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .collect();
    assert_eq!(settled.len(), 1, "{client_log}");
    assert!(
        settled[0].contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    let dack_pos = client_log
        .find("\"event\":\"r9_udp_return_delivery_ack\"")
        .unwrap_or(usize::MAX);
    let pack_pos = client_log
        .find("\"event\":\"r9_udp_return_packet_ack\",\"seq\":0,\"applied\":true")
        .unwrap_or(usize::MAX);
    let settled_pos = client_log
        .find("\"event\":\"r9_udp_post_return_settled\"")
        .unwrap_or(usize::MAX);
    assert!(dack_pos < settled_pos, "{client_log}");
    assert!(pack_pos < settled_pos, "{client_log}");
}
#[test]
fn reliable_udp_post_return_future_ack_is_rejected() {
    // H-R9-029/H-R9-030: a post-return canonical ACK with largest >
    // largest_sent is a typed rejection — no false current-packet retirement.
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("r9-pfut-server");
    let cp = tmp("r9-pfut-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "12",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--send-future-ack",
            "--diagnostic",
            "--experiment-id",
            "r9-pfut-srv",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "4",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--reliable-udp",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "r9-pfut-cli",
        ])
        .output()
        .unwrap();
    let (srv_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let client_err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{client_log} {client_err}");
    assert!(srv_status.success(), "{server_log}");
    // Exactly one typed post-return rejection; then the real Carrier ACK still
    // retires the post-return packet and settlement completes.
    let rejected: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_packet_ack_rejected\""))
        .collect();
    assert_eq!(rejected.len(), 1, "{client_log}");
    // The injected future ACK must not produce a positive retirement, and no
    // accepted-empty classification may appear for it.
    assert!(
        !client_log.contains("\"event\":\"r9_udp_return_packet_ack_accepted_empty\""),
        "{client_log}"
    );
    // H-R9-032: cross-process packet-number binding — client send, server ACK
    // send, and client positive retirement must all reference the same pn.
    let client_sent: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_sent\""))
        .collect();
    assert_eq!(client_sent.len(), 1, "{client_log}");
    let client_pn = packet_number(client_sent[0]).expect("client packet_number");
    let server_pack: Vec<&str> = server_log
        .lines()
        .filter(|l| l.contains("\"event\":\"udp_return_packet_ack_sent\""))
        .collect();
    assert_eq!(server_pack.len(), 1, "{server_log}");
    let server_pn = packet_number(server_pack[0]).expect("server packet_number");
    let retire: Vec<&str> = client_log
        .lines()
        .filter(|l| {
            l.contains("\"event\":\"r9_udp_return_packet_ack\"") && l.contains("\"retired\":true")
        })
        .collect();
    assert_eq!(retire.len(), 1, "{client_log}");
    let retire_pn = packet_number(retire[0]).expect("retire packet_number");
    assert_eq!(
        client_pn, server_pn,
        "client-send vs server-ack pn mismatch"
    );
    assert_eq!(
        client_pn, retire_pn,
        "client-send vs client-retire pn mismatch"
    );
    // Exactly one Session transition stream=1 offset=48 len=16.
    let dack: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_return_delivery_ack\""))
        .collect();
    assert_eq!(dack.len(), 1, "{client_log}");
    assert!(
        dack[0].contains("\"stream\":1")
            && dack[0].contains("\"offset\":48")
            && dack[0].contains("\"len\":16"),
        "{client_log}"
    );
    // Exactly one settlement with remaining_in_flight=0, strictly after both
    // ACK-domain transitions (compare line positions, not just existence).
    let settled: Vec<&str> = client_log
        .lines()
        .filter(|l| l.contains("\"event\":\"r9_udp_post_return_settled\""))
        .collect();
    assert_eq!(settled.len(), 1, "{client_log}");
    assert!(
        settled[0].contains("\"remaining_in_flight\":0"),
        "{client_log}"
    );
    let dack_pos = client_log
        .find("\"event\":\"r9_udp_return_delivery_ack\"")
        .unwrap_or(usize::MAX);
    let pack_pos = client_log
        .find("\"event\":\"r9_udp_return_packet_ack\",\"seq\":0,\"applied\":true")
        .unwrap_or(usize::MAX);
    let settled_pos = client_log
        .find("\"event\":\"r9_udp_post_return_settled\"")
        .unwrap_or(usize::MAX);
    assert!(dack_pos < settled_pos, "{client_log}");
    assert!(pack_pos < settled_pos, "{client_log}");
}
#[test]
fn executable_loopback_warm_tcp_precedes_udp_failure_and_data() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("warm-failover-server");
    let cp = tmp("warm-failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--cease-udp-replies-after",
            "1",
            "--send-malformed-after-cessation",
            "--migration-back",
            "--test-readiness-delay-ms",
            "400",
            "--test-tcp-delivery-delay-ms",
            "10",
            "--diagnostic",
            "--experiment-id",
            "warm-failover-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--automatic-health-failover",
            "--migration-back",
            "--diagnostic",
            "--experiment-id",
            "warm-failover-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        server_status.success(),
        "stdout={server_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !client_log.contains("carrier_event name=controlled_udp_stop"),
        "{client_log}"
    );
    assert!(
        client_log.contains("controlled_udp_stop=false"),
        "{client_log}"
    );
    assert!(client_log.contains("carrier_event name=udp_health_failed session=7001 generation=1 threshold=3 reason=udp_path_degraded diagnostic_cause=authenticated_delivery_ack_timeout"), "{client_log}");
    assert_eq!(
        client_log.matches("\"event\":\"udp_health_event\"").count(),
        3
    );
    assert!(
        client_log.contains("\"state\":\"degraded\""),
        "{client_log}"
    );
    assert!(client_log.contains("\"state\":\"failed\""), "{client_log}");
    assert!(
        client_log.contains("\"event\":\"tcp_active_health_observed\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"event\":\"udp_recovery_validated\""),
        "{client_log}"
    );
    assert!(client_log.contains("\"rtt_us\":"), "{client_log}");
    assert!(
        client_log.contains("\"fallback_class\":\"warm\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"promotion_gate\":\"warm_authenticated_resume\""),
        "{client_log}"
    );
    assert!(
        client_log.contains("\"readiness_satisfied_us\":"),
        "{client_log}"
    );
    assert!(
        !client_log.contains("\"cold_promotion_ready_us\":"),
        "{client_log}"
    );
    assert!(
        client_log.contains(
            "carrier_event name=tcp_warm session=7001 generation=1 readiness=3 application_data=0"
        ),
        "{client_log}"
    );
    assert_eq!(
        client_log
            .matches("\"event\":\"tcp_warm_readiness\"")
            .count(),
        3
    );
    assert_eq!(
        client_log.matches("carrier_event name=tcp_warm ").count(),
        1
    );
    let warm = client_log.find("carrier_event name=tcp_warm").unwrap();
    assert!(
        client_log
            .contains("carrier_event name=udp_recovered session=7001 generation=1 validated=true"),
        "{client_log}"
    );
    assert!(
        client_log.contains("carrier_event name=migrated_back_to_udp session=7001 generation=1"),
        "{client_log}"
    );
    let failed = client_log
        .find("carrier_event name=udp_health_failed")
        .unwrap();
    let data = client_log
        .find("\"event\":\"tcp_delivery_ack_validated\"")
        .unwrap();
    assert!(
        warm < failed && failed < data,
        "warm setup/failure/data order violated: {client_log}"
    );
    assert!(
        client_log.contains("carrier_event name=tcp_resume_guard"),
        "{client_log}"
    );
    assert_eq!(
        client_log
            .matches("\"event\":\"tcp_delivery_ack_validated\"")
            .count(),
        1
    );
    assert_eq!(
        client_log
            .matches("\"event\":\"udp_return_delivery_ack_validated\"")
            .count(),
        1
    );
    assert!(
        client_log.contains("failover_mode=automatic_health_failure"),
        "{client_log}"
    );
    assert!(
        server_log.contains("\"event\":\"udp_reply_ceased\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("carrier_event name=tcp_resumed"),
        "{server_log}"
    );
    assert!(
        server_log.contains("carrier_event name=udp_recovery_owner_started"),
        "{server_log}"
    );
    assert!(
        server_log.contains("records=3 application_bytes_total=48"),
        "{server_log}"
    );
    assert!(
        server_log.contains(&format!("bytes_hex={}", "78".repeat(48))),
        "{server_log}"
    );
}

#[test]
fn migration_back_tamper_fails_closed_before_return() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("migration-tamper-server");
    let cp = tmp("migration-tamper-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--cease-udp-replies-after",
            "1",
            "--send-malformed-after-cessation",
            "--migration-back",
            "--test-readiness-delay-ms",
            "400",
            "--test-tcp-delivery-delay-ms",
            "10",
            "--diagnostic",
            "--experiment-id",
            "migration-tamper-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let output = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--automatic-health-failover",
            "--migration-back",
            "--test-migration-back-tamper",
            "--count",
            "3",
            "--bytes",
            "16",
            "--duration",
            "10",
            "--diagnostic",
            "--experiment-id",
            "migration-tamper-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(!output.status.success());
    assert!(!server_status.success(), "{server_log}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tcp_delivery_ack_validated"), "{stdout}");
    assert!(stdout.contains("udp_recovery_challenge_sent"), "{stdout}");
    assert!(stdout.contains("udp_recovery_failed"), "{stdout}");
    assert!(stdout.contains("\"active\":\"tcp\""), "{stdout}");
    assert!(!stdout.contains("migrated_back_to_udp"));
    assert!(!stdout.contains("udp_return_delivery_ack_validated"));
    assert!(!server_log.contains("failover_server_ok"), "{server_log}");
    assert!(
        !server_log.contains("\"event\":\"summary\""),
        "{server_log}"
    );
}

#[test]
fn udp_reply_cessation_seam_is_bounded_and_off_by_default() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let identity = tmp("udp-cessation-invalid-server");
    let output = Command::new(bin)
        .args([
            "failover-server",
            "--count",
            "3",
            "--cease-udp-replies-after",
            "3",
            "--client-key",
            &"00".repeat(32),
            "--udp-port",
            "40082",
            "--tcp-port",
            "40083",
            "--identity",
            identity.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("UDP reply cessation point outside 1..count")
    );
    let _ = fs::remove_file(identity);
}

#[test]
fn first_udp_selection_loss_recovers_from_same_peer_duplicate_hello() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("retry-server");
    let cp = tmp("retry-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp_lease = PortLease::acquire(&[]);
    let tcp_lease = PortLease::acquire(&[udp_lease.port()]);
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "2",
            "--bytes",
            "13",
            "--duration",
            "5",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--drop-first-udp-selection",
            "--diagnostic",
            "--experiment-id",
            "selection-retry-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let client = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "2",
            "--bytes",
            "13",
            "--duration",
            "4",
            "--diagnostic",
            "--experiment-id",
            "selection-retry-client",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The first selection is deliberately lost, so this arrives while the
    // legitimate peer owns the bounded pending slot.
    thread::sleep(Duration::from_millis(30));
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .send_to(b"unrelated", ("127.0.0.1", udp))
        .unwrap();
    let client = client.wait_with_output().unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        client.status.success(),
        "client stdout={} stderr={}",
        String::from_utf8_lossy(&client.stdout),
        String::from_utf8_lossy(&client.stderr)
    );
    assert!(server_status.success(), "server log={server_log}");
    let log = server_log;
    assert!(log.contains("udp_selection_dropped"), "{log}");
    assert!(log.contains("udp_selection_retried"), "{log}");
    // The duplicate pending hello cannot restart negotiation or reset
    // ResumeGuard/session/path/delivery state.
    assert_eq!(log.matches("carrier_event name=udp_negotiated").count(), 1);
    assert_eq!(
        log.matches("carrier_event name=udp_authenticated").count(),
        1
    );
    assert_eq!(
        log.matches("carrier_event name=tcp_resumed session=7001 generation=1")
            .count(),
        1
    );
    assert!(
        log.contains("records=2 application_bytes_total=26"),
        "{log}"
    );
}

#[test]
fn expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_handshake_recovers() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("expired-preprogress-server");
    let cp = tmp("expired-preprogress-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "1",
            "--bytes",
            "16",
            "--duration",
            "7",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--diagnostic",
            "--experiment-id",
            "expired-preprogress-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);

    let expired = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "1",
            "--bytes",
            "16",
            "--duration",
            "2",
            "--test-first-data-delay-ms",
            "1200",
        ])
        .output()
        .unwrap();
    assert!(!expired.status.success());
    assert!(!String::from_utf8_lossy(&expired.stdout).contains("udp_delivery_ack_validated"));

    let recovered = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "1",
            "--bytes",
            "16",
            "--duration",
            "3",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    assert!(
        recovered.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&recovered.stdout),
        String::from_utf8_lossy(&recovered.stderr)
    );
    assert!(server_status.success(), "{server_log}");
    assert!(
        server_log.contains("udp_preprogress_expired"),
        "{server_log}"
    );
    assert_eq!(
        server_log.matches("udp_delivery_ack_sent").count(),
        1,
        "expired Data must not create an ACK; only the recovered exchange may: {server_log}"
    );
    let expiry = server_log.find("udp_preprogress_expired").unwrap();
    let second_auth = server_log[expiry..]
        .find("carrier_event name=udp_authenticated")
        .map(|offset| expiry + offset)
        .unwrap();
    let resume = server_log.find("tcp_resume_validated").unwrap();
    assert!(
        expiry < second_auth && second_auth < resume,
        "only the fresh post-expiry handshake may establish the guard used by resume: {server_log}"
    );
    assert!(String::from_utf8_lossy(&recovered.stdout).contains("failover_client_ok"));
    assert!(server_log.contains("failover_server_ok"));
    fs::remove_file(sp).unwrap();
    fs::remove_file(cp).unwrap();
}

#[test]
fn first_udp_noise_response_loss_replays_without_resetting_session_state() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("noise-retry-server");
    let cp = tmp("noise-retry-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let (udp_lease, tcp_lease) = failover_port_leases();
    let udp = udp_lease.port();
    let tcp = tcp_lease.port();
    udp_lease.release();
    tcp_lease.release();
    let server = Command::new(bin)
        .args([
            "failover-server",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "2",
            "--bytes",
            "17",
            "--duration",
            "5",
            "--udp-bind",
            &format!("127.0.0.1:{udp}"),
            "--tcp-bind",
            &format!("127.0.0.1:{tcp}"),
            "--drop-first-udp-noise-response",
            "--delay-noise-duplicate-until-application",
            "--diagnostic",
            "--experiment-id",
            "noise-retry-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_failover_server(server);
    let client = Command::new(bin)
        .args([
            "failover-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &udp.to_string(),
            "--tcp-port",
            &tcp.to_string(),
            "--server-key",
            &sk,
            "--identity",
            cp.to_str().unwrap(),
            "--count",
            "2",
            "--bytes",
            "17",
            "--duration",
            "4",
            "--diagnostic",
            "--experiment-id",
            "noise-retry-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        client.status.success(),
        "client stdout={} stderr={}",
        String::from_utf8_lossy(&client.stdout),
        String::from_utf8_lossy(&client.stderr)
    );
    assert!(server_status.success(), "server log={server_log}");
    let client_log = String::from_utf8_lossy(&client.stdout);
    assert!(
        server_log.contains("udp_noise_response_dropped"),
        "{server_log}"
    );
    assert!(
        server_log.contains("udp_noise_response_retried"),
        "{server_log}"
    );
    assert_eq!(
        server_log.matches("udp_retained_input_charged").count(),
        2,
        "one cached first-Noise retry and the first non-matching authenticated Data must each be charged exactly once before classification; later post-progress Data is outside pre-auth ownership: {server_log}"
    );
    assert!(
        server_log.contains("udp_noise_response_delayed_duplicate_suppressed"),
        "{server_log}"
    );
    assert!(
        !client_log.contains("duplicate_noise_response"),
        "{client_log}"
    );
    // A duplicate Noise first-message must replay only the cached response. It
    // must not renegotiate, reauthenticate, replace ResumeGuard/session state,
    // increment path generation, or reset delivery state.
    assert_eq!(
        server_log
            .matches("carrier_event name=udp_negotiated")
            .count(),
        1
    );
    assert_eq!(
        server_log
            .matches("carrier_event name=udp_authenticated")
            .count(),
        1
    );
    assert_eq!(
        server_log
            .matches("carrier_event name=tcp_resumed session=7001 generation=1")
            .count(),
        1
    );
    assert!(client_log.contains("carrier_event name=tcp_resume_guard session=7001 generation=1"));
    assert!(client_log.contains("\"event\":\"udp_delivery_ack_validated\""));
    assert!(client_log.contains("\"event\":\"tcp_delivery_ack_validated\""));
    assert!(
        server_log.contains("records=2 application_bytes_total=34"),
        "{server_log}"
    );
}

#[test]
fn failover_udp_handshake_timeout_reports_last_success_stage() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let dir = tmp("failover-timeout");
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--json",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            "40099",
            "--tcp-port",
            "40100",
            "--server-key",
            &"00".repeat(32),
            "--identity",
            dir.to_str().unwrap(),
            "--duration",
            "1",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success());
    assert!(stdout.contains(r#""stage":"socket_bind""#));
    assert!(stdout.contains(r#""stage":"client_hello_sent""#));
    assert!(
        stdout.contains(r#""stage":"timeout","last_success_stage":"client_hello_sent""#),
        "{stdout}"
    );
    let _ = fs::remove_file(dir);
}

#[test]
fn udp_handshake_diagnostic_stages_are_deterministic() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let dir = tmp("diagnostic-timeout");
    let out = Command::new(bin)
        .args([
            "failover-client",
            "--json",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            "40099",
            "--tcp-port",
            "40100",
            "--server-key",
            &"00".repeat(32),
            "--identity",
            dir.to_str().unwrap(),
            "--duration",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let log = String::from_utf8_lossy(&out.stdout);
    for stage in ["socket_bind", "client_send"] {
        assert!(
            log.contains(&format!("\"stage\":\"{stage}\"")),
            "missing {stage}: {log}"
        );
    }
    assert!(log.contains("last_success_stage\":\"client_hello_sent"));
    let _ = fs::remove_file(dir);
}

const PROBE_DOMAIN: &[u8] = b"nekomusume-vps-probe";
const PROBE_SCOPE: &[u8] = b"probe";

fn test_context() -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 1,
        direction: 0,
    }
}

fn load_test_identity(path: &std::path::Path) -> LocalIdentity {
    let text = fs::read_to_string(path).unwrap();
    let (private, public) = text.trim().split_once(':').unwrap();
    let decode = |value: &str| {
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    LocalIdentity::from_keypair(&decode(private), &decode(public)).unwrap()
}

fn authenticated_udp_client_with_data_delay(
    port: u16,
    client_identity_path: &std::path::Path,
    server_key: &str,
    delay: Duration,
) -> std::io::Result<Vec<u8>> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect(("127.0.0.1", port))?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Client, &[NEGOTIATION_VERSION]).unwrap();
    socket.send(&negotiation.client_hello().unwrap())?;
    let mut buffer = [0; 2048];
    let len = socket.recv(&mut buffer)?;
    negotiation.client_accept_response(&buffer[..len]).unwrap();
    let binding = negotiation.authenticated_binding().unwrap();
    let identity = load_test_identity(client_identity_path);
    let mut handshake = InitiatorHandshake::new_with_prologue_binding(
        &identity,
        &decode_key(server_key),
        PROBE_SCOPE,
        PROBE_DOMAIN,
        binding.as_bytes(),
    )
    .unwrap();
    socket.send(&handshake.first_message().unwrap())?;
    let len = socket.recv(&mut buffer)?;
    let mut session = handshake.finish(&buffer[..len], test_context()).unwrap();
    negotiation.admit_data().unwrap();
    thread::sleep(delay);
    let payload = b"delayed authenticated application data";
    socket.send(&session.seal_unreliable(payload).unwrap())?;
    let len = socket.recv(&mut buffer)?;
    Ok(session.open_unreliable(&buffer[..len]).unwrap())
}

#[test]
fn udp_application_wait_uses_overall_duration_not_poll_interval() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("udp-delayed-data-server");
    let cp = tmp("udp-delayed-data-client");
    let server_key = key(bin, &sp);
    let client_key = key(bin, &cp);
    let server = start_server(bin, "udp", 40082, &sp, &client_key);
    let echoed = authenticated_udp_client_with_data_delay(
        40082,
        &cp,
        &server_key,
        Duration::from_millis(250),
    )
    .unwrap();
    let (status, log) = finish_server(server);
    assert_eq!(echoed, b"delayed authenticated application data");
    assert!(status.success(), "{log}");
    assert!(log.contains("lifecycle_state=STOPPED readiness=false"));
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn udp_application_wait_fails_at_bounded_overall_deadline() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("udp-data-deadline-server");
    let cp = tmp("udp-data-deadline-client");
    let server_key = key(bin, &sp);
    let client_key = key(bin, &cp);
    let mut server = start_server_for(bin, "udp", 40083, &sp, &client_key, "1");
    // Complete negotiation/authentication, then intentionally cross the server's
    // one-second application deadline before emitting the first data record.
    let started = Instant::now();
    let result = authenticated_udp_client_with_data_delay(
        40083,
        &cp,
        &server_key,
        Duration::from_millis(1_250),
    );
    let status = server.child.wait().unwrap();
    let elapsed = started.elapsed();
    let mut log = server.startup_log;
    server.stdout.read_to_string(&mut log).unwrap();
    assert!(result.is_err());
    assert!(!status.success(), "{log}");
    assert!(elapsed >= Duration::from_millis(900));
    assert!(
        elapsed < Duration::from_secs(3),
        "unbounded failure: {elapsed:?}"
    );
    let stderr = server
        .child
        .stderr
        .take()
        .map(|mut stream| {
            let mut text = String::new();
            stream.read_to_string(&mut text).unwrap();
            text
        })
        .unwrap_or_default();
    assert!(stderr.contains("data timeout"), "{stderr}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

fn run_client(
    bin: &str,
    transport: &str,
    port: u16,
    identity: &std::path::Path,
    server_key: &str,
) -> std::process::Output {
    Command::new(bin)
        .args([
            "client",
            "--transport",
            transport,
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--server-key",
            server_key,
            "--identity",
            identity.to_str().unwrap(),
            "--duration",
            "2",
        ])
        .output()
        .unwrap()
}

#[test]
fn tcp_and_udp_reject_malformed_unsupported_and_duplicate_negotiation_before_echo() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (case, message) in [
        ("malformed", vec![b'N', b'1', 1, 1, 0]),
        ("unsupported", vec![b'N', b'1', 1, 1, 0, 1]),
    ] {
        for (transport, port) in [("tcp", 40091u16), ("udp", 40092u16)] {
            let sp = tmp(&format!("{transport}-{case}-server"));
            let cp = tmp(&format!("{transport}-{case}-client"));
            let ck = key(bin, &cp);
            let server = start_server(bin, transport, port, &sp, &ck);
            if transport == "tcp" {
                let mut socket = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
                frame_write_test(&mut socket, &message);
                socket
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut byte = [0; 1];
                assert_eq!(
                    socket.read(&mut byte).unwrap(),
                    0,
                    "{case} reached TCP response/data"
                );
            } else {
                let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                socket.send_to(&message, ("127.0.0.1", port)).unwrap();
                let mut byte = [0; 1];
                assert!(
                    socket.recv(&mut byte).is_err(),
                    "{case} reached UDP response/data"
                );
            }
            let (status, log) = finish_server(server);
            assert!(!status.success(), "{case} unexpectedly admitted: {log}");
            assert!(
                !log.contains("lifecycle_state=STOPPED"),
                "{case} produced successful echo lifecycle: {log}"
            );
            let _ = fs::remove_file(sp);
            let _ = fs::remove_file(cp);
        }
    }

    for (transport, port) in [("tcp", 40093u16), ("udp", 40094u16)] {
        let sp = tmp(&format!("{transport}-duplicate-server"));
        let cp = tmp(&format!("{transport}-duplicate-client"));
        let ck = key(bin, &cp);
        let server = start_server(bin, transport, port, &sp, &ck);
        let hello = [b'N', b'1', 1, 1, 0, 0];
        if transport == "tcp" {
            let mut socket = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            frame_write_test(&mut socket, &hello);
            assert_eq!(frame_read_test(&mut socket), [b'N', b'1', 2, 0, 0, 0]);
            frame_write_test(&mut socket, &hello);
            socket
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut byte = [0; 1];
            assert_eq!(
                socket.read(&mut byte).unwrap(),
                0,
                "duplicate hello reached TCP data"
            );
        } else {
            let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            socket.send_to(&hello, ("127.0.0.1", port)).unwrap();
            let mut response = [0; 64];
            let n = socket.recv(&mut response).unwrap();
            assert_eq!(&response[..n], &[b'N', b'1', 2, 0, 0, 0]);
            socket.send_to(&hello, ("127.0.0.1", port)).unwrap();
            assert!(
                socket.recv(&mut response).is_err(),
                "duplicate hello reached UDP data"
            );
        }
        let (status, log) = finish_server(server);
        assert!(!status.success(), "duplicate negotiation admitted: {log}");
        assert!(
            !log.contains("lifecycle_state=STOPPED"),
            "duplicate produced successful echo: {log}"
        );
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
    }
}

fn frame_write_test(stream: &mut std::net::TcpStream, frame: &[u8]) {
    stream
        .write_all(&(frame.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(frame).unwrap();
}
fn frame_read_test(stream: &mut std::net::TcpStream) -> Vec<u8> {
    let mut length = [0; 4];
    stream.read_exact(&mut length).unwrap();
    let mut frame = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut frame).unwrap();
    frame
}

#[test]
fn tcp_and_udp_transcript_mismatch_rejects_before_application_echo() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40095u16), ("udp", 40096u16)] {
        let sp = tmp(&format!("{transport}-mismatch-server"));
        let cp = tmp(&format!("{transport}-mismatch-client"));
        let server_key = key(bin, &sp);
        let client_key = key(bin, &cp);
        let server_identity = load_test_identity(&sp);
        let client_public = decode_key(&client_key);
        let peer = thread::spawn(move || {
            let policy = TrustPolicy::new(vec![TrustRecord {
                version: 1,
                public_key: client_public,
                scope: PROBE_SCOPE.to_vec(),
                status: TrustStatus::Active,
            }]);
            if transport == "tcp" {
                let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
                let (mut socket, _) = listener.accept().unwrap();
                let hello = frame_read_test(&mut socket);
                let mut negotiation =
                    VersionNegotiator::new(NegotiationRole::Server, &[NEGOTIATION_VERSION])
                        .unwrap();
                let response = negotiation.server_accept_hello(&hello).unwrap();
                frame_write_test(&mut socket, &response);
                let mut binding = negotiation
                    .authenticated_binding()
                    .unwrap()
                    .as_bytes()
                    .to_vec();
                binding[0] ^= 1;
                let handshake = ResponderHandshake::new_with_prologue_binding(
                    &server_identity,
                    policy,
                    PROBE_DOMAIN,
                    &binding,
                )
                .unwrap();
                let first = frame_read_test(&mut socket);
                assert!(handshake.receive_first(&first, test_context()).is_err());
            } else {
                let socket = UdpSocket::bind(("127.0.0.1", port)).unwrap();
                let mut buf = [0; 2048];
                let (n, peer) = socket.recv_from(&mut buf).unwrap();
                let mut negotiation =
                    VersionNegotiator::new(NegotiationRole::Server, &[NEGOTIATION_VERSION])
                        .unwrap();
                let response = negotiation.server_accept_hello(&buf[..n]).unwrap();
                socket.send_to(&response, peer).unwrap();
                let mut binding = negotiation
                    .authenticated_binding()
                    .unwrap()
                    .as_bytes()
                    .to_vec();
                binding[0] ^= 1;
                let handshake = ResponderHandshake::new_with_prologue_binding(
                    &server_identity,
                    policy,
                    PROBE_DOMAIN,
                    &binding,
                )
                .unwrap();
                let (n, same_peer) = socket.recv_from(&mut buf).unwrap();
                assert_eq!(same_peer, peer);
                assert!(handshake.receive_first(&buf[..n], test_context()).is_err());
            }
        });
        thread::sleep(Duration::from_millis(50));
        let output = run_client(bin, transport, port, &cp, &server_key);
        peer.join().unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("probe_ok"));
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
    }
}

fn decode_key(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn tcp_and_udp_reject_unsupported_selected_version_before_noise() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40097u16), ("udp", 40098u16)] {
        let sp = tmp(&format!("{transport}-selected-server"));
        let cp = tmp(&format!("{transport}-selected-client"));
        let server_key = key(bin, &sp);
        let peer = thread::spawn(move || {
            if transport == "tcp" {
                let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
                let (mut socket, _) = listener.accept().unwrap();
                assert_eq!(frame_read_test(&mut socket), [b'N', b'1', 1, 1, 0, 0]);
                frame_write_test(&mut socket, &[b'N', b'1', 2, 0, 0, 1]);
                socket
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut byte = [0; 1];
                assert_eq!(
                    socket.read(&mut byte).unwrap(),
                    0,
                    "client emitted Noise/data"
                );
            } else {
                let socket = UdpSocket::bind(("127.0.0.1", port)).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut buf = [0; 64];
                let (n, peer) = socket.recv_from(&mut buf).unwrap();
                assert_eq!(&buf[..n], &[b'N', b'1', 1, 1, 0, 0]);
                socket.send_to(&[b'N', b'1', 2, 0, 0, 1], peer).unwrap();
                assert!(
                    socket.recv_from(&mut buf).is_err(),
                    "client emitted Noise/data"
                );
            }
        });
        thread::sleep(Duration::from_millis(50));
        let output = run_client(bin, transport, port, &cp, &server_key);
        peer.join().unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("probe_ok"));
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
    }
}

static TEST_PORT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct PortLease {
    tcp: TcpListener,
    udp: UdpSocket,
    port: u16,
}

impl PortLease {
    fn acquire(excluded: &[u16]) -> Self {
        for port in 40080..=40100 {
            if excluded.contains(&port) {
                continue;
            }
            if let (Ok(tcp), Ok(udp)) = (
                TcpListener::bind(("127.0.0.1", port)),
                UdpSocket::bind(("127.0.0.1", port)),
            ) {
                return Self { tcp, udp, port };
            }
        }
        panic!("no test port is locally available");
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn release(self) {
        drop(self.tcp);
        drop(self.udp);
    }
}

fn failover_port_leases() -> (PortLease, PortLease) {
    let udp = PortLease::acquire(&[]);
    let tcp = PortLease::acquire(&[udp.port()]);
    (udp, tcp)
}

fn periodic_test_port() -> PortLease {
    PortLease::acquire(&[])
}

fn start_periodic_server(
    bin: &str,
    port: u16,
    identity: &std::path::Path,
    client_key: &str,
    extra: &[&str],
) -> ReadyServer {
    let mut command = Command::new(bin);
    command
        .args([
            "periodic-server",
            "--port",
            &port.to_string(),
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--identity",
            identity.to_str().unwrap(),
            "--client-key",
            client_key,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "500",
        ])
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut startup_log = String::new();
    loop {
        let mut line = String::new();
        assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "{startup_log}");
        startup_log.push_str(&line);
        if line.contains("periodic_server_ready") {
            break;
        }
    }
    ReadyServer {
        child,
        stdout,
        startup_log,
    }
}

#[test]
fn periodic_session_delayed_confirmations_are_counted_on_one_session() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-delay-server");
    let cp = tmp("periodic-delay-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--test-ack-delay-ms", "150"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "500",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(log.contains("periodic_client_authenticated session=7201 stream=1"));
    assert!(log.contains("attempted=3 confirmed=3 missing=0"), "{log}");
    let (status, server_log) = finish_server(server);
    assert!(status.success(), "{server_log}");
    assert_eq!(
        server_log.matches("periodic_server_authenticated").count(),
        1
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_session_synchronized_key_update_crosses_authenticated_socket() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-key-update-server");
    let cp = tmp("periodic-key-update-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--key-update-after", "1"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "500",
            "--key-update-after",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        client_log.contains("periodic_client_key_update seq=1 key_phase=1"),
        "{client_log}"
    );
    assert!(
        client_log.contains("attempted=3 confirmed=3 missing=0"),
        "{client_log}"
    );
    let (status, server_log) = finish_server(server);
    assert!(status.success(), "{server_log}");
    assert!(
        server_log.contains("periodic_server_key_update seq=1 key_phase=1"),
        "{server_log}"
    );
    assert!(
        server_log.contains("received=3 confirmed=3"),
        "{server_log}"
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_session_mismatched_key_update_schedule_fails_closed() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-key-update-mismatch-server");
    let cp = tmp("periodic-key-update-mismatch-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--key-update-after", "1"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "500",
            "--key-update-after",
            "2",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        !client_log.contains("attempted=3 confirmed=3 missing=0"),
        "{client_log}"
    );
    let (status, server_log) = finish_server(server);
    assert!(!status.success(), "{server_log}");
    assert!(
        !server_log.contains("received=3 confirmed=3"),
        "{server_log}"
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_session_accounts_missing_ack_and_fails_closed() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-missing-server");
    let cp = tmp("periodic-missing-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--test-drop-ack", "3"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "2",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "200",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(log.contains("attempted=3 confirmed=2 missing=1"), "{log}");
    let (status, server_log) = finish_server(server);
    assert!(status.success(), "status={status:?} {server_log}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_session_duplicate_ack_is_authenticated_and_idempotent() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-duplicate-server");
    let cp = tmp("periodic-duplicate-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--test-duplicate-ack"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "500",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(
        log.contains("attempted=3 confirmed=3 missing=0 duplicates=2"),
        "{log}"
    );
    let (status, server_log) = finish_server(server);
    assert!(status.success(), "{server_log}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_setup_timeout_is_separate_from_ack_timeout() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-setup-separated-server");
    let cp = tmp("periodic-setup-separated-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--test-setup-delay-ms", "300"]);
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "5",
            "--count",
            "3",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "100",
            "--setup-timeout-ms",
            "1000",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("attempted=3 confirmed=3 missing=0"));
    let (status, server_log) = finish_server(server);
    assert!(status.success(), "{server_log}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_setup_timeout_fails_before_application_records() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-setup-timeout-server");
    let cp = tmp("periodic-setup-timeout-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(
        bin,
        port,
        &sp,
        &ck,
        &["--setup-timeout-ms", "200", "--test-setup-delay-ms", "400"],
    );
    let out = Command::new(bin)
        .args([
            "periodic-client",
            "--port",
            &port.to_string(),
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--duration",
            "2",
            "--count",
            "1",
            "--bytes",
            "16",
            "--interval-ms",
            "100",
            "--ack-timeout-ms",
            "100",
            "--setup-timeout-ms",
            "200",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let (status, server_log) = finish_server(server);
    assert!(!status.success(), "{server_log}");
    assert!(
        !server_log.contains("periodic_server_authenticated"),
        "{server_log}"
    );
    assert!(
        !server_log.contains("periodic_server_interval"),
        "{server_log}"
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_malformed_setup_fails_unauthenticated() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-malformed-server");
    let cp = tmp("periodic-malformed-client");
    let ck = key(bin, &cp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &["--setup-timeout-ms", "500"]);
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.write_all(&[0, 0, 0, 1, 0xff]).unwrap();
    drop(stream);
    let (status, server_log) = finish_server(server);
    assert!(!status.success(), "{server_log}");
    assert!(
        !server_log.contains("periodic_server_authenticated"),
        "{server_log}"
    );
    assert!(
        !server_log.contains("periodic_server_interval"),
        "{server_log}"
    );
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn periodic_server_signal_cleanup_is_bounded() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-signal-server");
    let cp = tmp("periodic-signal-client");
    let ck = key(bin, &cp);
    let port_lease = periodic_test_port();
    let port = port_lease.port();
    port_lease.release();
    let server = start_periodic_server(bin, port, &sp, &ck, &[]);
    signal_term(&server.child);
    let (status, log) = finish_server(server);
    assert!(status.success(), "{log}");
    assert!(log.contains("cleanup=verified"), "{log}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("endpoint-rebind-server");
    let cp = tmp("endpoint-rebind-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let port = 40090u16;
    let server = Command::new(bin)
        .args([
            "endpoint-rebind-server",
            "--udp-port",
            &port.to_string(),
            "--udp-bind",
            &format!("127.0.0.1:{port}"),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "2",
            "--bytes",
            "16",
            "--duration",
            "5",
            "--test-promotion-delay-ms",
            "100",
            "--diagnostic",
            "--experiment-id",
            "endpoint-rebind-test-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_endpoint_rebind_server(server);
    let out = Command::new(bin)
        .args([
            "endpoint-rebind-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &port.to_string(),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--count",
            "2",
            "--bytes",
            "16",
            "--duration",
            "5",
            "--test-stale-old-endpoint",
            "--diagnostic",
            "--experiment-id",
            "endpoint-rebind-test-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{client_log} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(server_status.success(), "{server_log}");
    assert!(
        client_log.contains("\"source_endpoint_changed\":true"),
        "{client_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_candidate_seen\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_challenge_sent\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_promoted\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_promotion_sync_sent\""),
        "{server_log}"
    );
    assert!(
        client_log.contains("\"event\":\"endpoint_promotion_sync_received\""),
        "{client_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_promotion_delay_held\""),
        "{server_log}"
    );
    assert!(
        server_log.contains("\"event\":\"endpoint_stale_source_rejected\""),
        "{server_log}"
    );
    assert!(
        client_log.contains("endpoint_post_delivery_ack_validated"),
        "{client_log}"
    );
    assert!(
        server_log.contains("records=2 application_bytes_total=32"),
        "{server_log}"
    );
}

#[test]
fn endpoint_rebind_wrong_challenge_fails_after_candidate_without_success() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("endpoint-rebind-negative-server");
    let cp = tmp("endpoint-rebind-negative-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let port = 40090u16;
    let server = Command::new(bin)
        .args([
            "endpoint-rebind-server",
            "--udp-port",
            &port.to_string(),
            "--udp-bind",
            &format!("127.0.0.1:{port}"),
            "--identity",
            sp.to_str().unwrap(),
            "--client-key",
            &ck,
            "--count",
            "2",
            "--bytes",
            "16",
            "--duration",
            "3",
            "--diagnostic",
            "--experiment-id",
            "endpoint-rebind-negative-server",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let server = ready_endpoint_rebind_server(server);
    let out = Command::new(bin)
        .args([
            "endpoint-rebind-client",
            "--addr",
            "127.0.0.1",
            "--udp-port",
            &port.to_string(),
            "--identity",
            cp.to_str().unwrap(),
            "--server-key",
            &sk,
            "--count",
            "2",
            "--bytes",
            "16",
            "--duration",
            "3",
            "--test-wrong-rebind-challenge",
            "--diagnostic",
            "--experiment-id",
            "endpoint-rebind-negative-client",
        ])
        .output()
        .unwrap();
    let (server_status, server_log) = finish_server(server);
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success(), "{client_log}");
    assert!(!server_status.success(), "{server_log}");
    assert!(
        server_log.contains("endpoint_candidate_seen"),
        "{server_log}"
    );
    assert!(
        server_log.contains("endpoint_challenge_sent"),
        "{server_log}"
    );
    assert!(
        server_log.contains("endpoint_rebind_failed"),
        "{server_log}"
    );
    assert!(
        !server_log.contains("endpoint_promotion_sync_sent"),
        "{server_log}"
    );
    assert!(!server_log.contains("endpoint_promoted"), "{server_log}");
    assert!(
        !server_log.contains("endpoint_rebind_server_ok"),
        "{server_log}"
    );
    assert!(
        !client_log.contains("endpoint_post_delivery_ack_validated"),
        "{client_log}"
    );
}

#[test]
fn warm_readiness_failures_close_before_admission_or_application_data() {
    let _port_lock = TEST_PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (name, seam, server_seam) in [
        (
            "wrong-tuple",
            "--test-readiness-wrong-tuple",
            "--diagnostic",
        ),
        (
            "admitted-false",
            "--diagnostic",
            "--test-readiness-unadmitted",
        ),
        (
            "replayed-challenge",
            "--test-readiness-replay",
            "--diagnostic",
        ),
        (
            "tampered-ciphertext",
            "--test-readiness-tamper",
            "--diagnostic",
        ),
        ("fewer-than-three", "--test-readiness-short", "--diagnostic"),
        ("readiness-stall", "--test-readiness-stall", "--diagnostic"),
        (
            "probe-over-one-second",
            "--diagnostic",
            "--test-readiness-delay-1200",
        ),
        (
            "sequence-over-three-seconds",
            "--test-readiness-sequence-pause-200",
            "--test-readiness-delay-950",
        ),
    ] {
        let sp = tmp(&format!("negative-{name}-server"));
        let cp = tmp(&format!("negative-{name}-client"));
        let sk = key(bin, &sp);
        let ck = key(bin, &cp);
        let (udp_lease, tcp_lease) = failover_port_leases();
        let udp = udp_lease.port();
        let tcp = tcp_lease.port();
        udp_lease.release();
        tcp_lease.release();
        let started = Instant::now();
        let server = Command::new(bin)
            .args([
                "failover-server",
                "--udp-port",
                &udp.to_string(),
                "--tcp-port",
                &tcp.to_string(),
                "--identity",
                sp.to_str().unwrap(),
                "--client-key",
                &ck,
                "--count",
                "3",
                "--bytes",
                "16",
                "--duration",
                "3",
                "--udp-bind",
                &format!("127.0.0.1:{udp}"),
                "--tcp-bind",
                &format!("127.0.0.1:{tcp}"),
                "--cease-udp-replies-after",
                "1",
                "--diagnostic",
                "--experiment-id",
                &format!("negative-{name}-server"),
                if matches!(
                    server_seam,
                    "--test-readiness-delay-1200" | "--test-readiness-delay-950"
                ) {
                    "--test-readiness-delay-ms"
                } else {
                    server_seam
                },
            ])
            .args(if server_seam == "--test-readiness-delay-1200" {
                vec!["1200"]
            } else if server_seam == "--test-readiness-delay-950" {
                vec!["950"]
            } else {
                vec![]
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(150));
        let client = Command::new(bin)
            .args([
                "failover-client",
                "--addr",
                "127.0.0.1",
                "--udp-port",
                &udp.to_string(),
                "--tcp-port",
                &tcp.to_string(),
                "--server-key",
                &sk,
                "--identity",
                cp.to_str().unwrap(),
                "--count",
                "3",
                "--bytes",
                "16",
                "--duration",
                "2",
                "--automatic-health-failover",
                if seam == "--test-readiness-sequence-pause-200" {
                    "--test-readiness-sequence-pause-ms"
                } else {
                    seam
                },
            ])
            .args(if seam == "--test-readiness-sequence-pause-200" {
                vec!["200"]
            } else {
                vec![]
            })
            .output()
            .unwrap();
        let server = server.wait_with_output().unwrap();
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
        let client_log = format!(
            "{}{}",
            String::from_utf8_lossy(&client.stdout),
            String::from_utf8_lossy(&client.stderr)
        );
        let server_log = format!(
            "{}{}",
            String::from_utf8_lossy(&server.stdout),
            String::from_utf8_lossy(&server.stderr)
        );
        assert!(!client.status.success(), "{name}: {client_log}");
        assert!(!server.status.success(), "{name}: {server_log}");
        assert!(started.elapsed() < Duration::from_secs(5), "{name} stalled");
        for forbidden in [
            "carrier_event name=tcp_resource_admitted",
            "carrier_event name=tcp_warm",
            "carrier_event name=tcp_resumed",
            "tcp_delivery_ack_sent",
        ] {
            assert!(!server_log.contains(forbidden), "{name}: {server_log}");
            assert!(!client_log.contains(forbidden), "{name}: {client_log}");
        }
    }
}

#[test]
fn lab_reliable_udp_scenarios_are_truthful_and_legacy_lab_is_preserved() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let legacy = Command::new(bin).args(["lab", "--json"]).output().unwrap();
    assert!(legacy.status.success());
    let legacy_out = String::from_utf8_lossy(&legacy.stdout);
    assert!(legacy_out.contains(r#""demo":"failover""#), "{legacy_out}");

    let clean = Command::new(bin)
        .args([
            "lab",
            "--scenario",
            "reliable-udp",
            "--rounds",
            "8",
            "--drop-every",
            "0",
            "--settle-ms",
            "1500",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let s = String::from_utf8_lossy(&clean.stdout);
    assert!(s.contains(r#""ok":true"#), "{s}");
    assert!(s.contains(r#""delivered":8"#), "{s}");
    assert!(s.contains(r#""conflicts":0"#), "{s}");
    assert!(s.contains(r#""pto_fired":0"#), "{s}");
    assert!(s.contains(r#""retransmit_wire_sent":0"#), "{s}");
    assert!(s.contains(r#""remaining_in_flight":0"#), "{s}");
    assert!(s.contains(r#""cleanup":"verified""#), "{s}");
    assert!(s.contains(r#""manager_active_path":"udp""#), "{s}");
    assert!(
        s.contains(
            r#""manager_switch_scope":"manager decision only; this command opens no TCP socket""#
        ),
        "{s}"
    );

    let lossy = Command::new(bin)
        .args([
            "lab",
            "--scenario",
            "reliable-udp",
            "--rounds",
            "8",
            "--drop-every",
            "4",
            "--settle-ms",
            "4000",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        lossy.status.success(),
        "{}",
        String::from_utf8_lossy(&lossy.stderr)
    );
    let s = String::from_utf8_lossy(&lossy.stdout);
    assert!(s.contains(r#""ok":true"#), "{s}");
    assert!(s.contains(r#""delivered":8"#), "{s}");
    assert!(s.contains(r#""duplicates":0"#), "{s}");
    assert!(s.contains(r#""suppressed":2"#), "{s}");
    assert!(s.contains(r#""pto_fired":2"#), "{s}");
    assert!(s.contains(r#""retransmit_wire_sent":2"#), "{s}");
    assert!(s.contains(r#""remaining_in_flight":0"#), "{s}");

    let ackloss = Command::new(bin)
        .args([
            "lab",
            "--scenario",
            "reliable-udp",
            "--rounds",
            "8",
            "--drop-every",
            "0",
            "--drop-ack-every",
            "3",
            "--settle-ms",
            "4000",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        ackloss.status.success(),
        "{}",
        String::from_utf8_lossy(&ackloss.stderr)
    );
    let s = String::from_utf8_lossy(&ackloss.stdout);
    assert!(s.contains(r#""ok":true"#), "{s}");
    assert!(s.contains(r#""delivered":8"#), "{s}");
    assert!(s.contains(r#""duplicates":3"#), "{s}");
    assert!(
        s.contains(r#""acks":{"emitted":11,"wire_sent":8,"suppressed":3,"failed":0,"applied":8,"rejected":0}"#),
        "{s}"
    );
    assert!(s.contains(r#""pto_fired":3"#), "{s}");
    assert!(s.contains(r#""remaining_in_flight":0"#), "{s}");
}

#[test]
fn lab_reliable_udp_reports_partial_and_exits_nonzero_when_delivery_is_unfinished() {
    // The declared invariant is that ok=true is unreachable with unfinished
    // delivery. Force the bound to expire before recovery settles and pin the
    // truthful outcome: partial, ok=false, nonzero exit.
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let out = Command::new(bin)
        .args([
            "lab",
            "--scenario",
            "reliable-udp",
            "--rounds",
            "8",
            "--drop-every",
            "0",
            "--drop-ack-every",
            "1",
            "--settle-ms",
            "1",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(r#""ok":false"#), "{s}");
    assert!(s.contains(r#""settled":false"#), "{s}");
    assert!(s.contains(r#""partial":true"#), "{s}");
    assert!(!s.contains(r#""ok":true"#), "{s}");
}

#[test]
fn lab_scenario_arguments_fail_closed_before_sockets() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for args in [
        vec!["lab", "--scenario", "bogus"],
        vec![
            "lab",
            "--scenario",
            "reliable-udp",
            "--scenario",
            "reliable-udp",
        ],
        vec!["lab", "--scenario"],
        vec!["lab", "--scenario", "reliable-udp", "--rounds", "3"],
        vec!["lab", "--scenario", "reliable-udp", "--rounds", "65"],
        vec!["lab", "--scenario", "reliable-udp", "--drop-every", "1"],
        vec![
            "lab",
            "--scenario",
            "reliable-udp",
            "--drop-ack-every",
            "17",
        ],
        vec!["lab", "--scenario", "reliable-udp", "--settle-ms", "0"],
    ] {
        let out = Command::new(bin).args(&args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "args={args:?} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stdout.is_empty(), "args={args:?}");
    }
}
