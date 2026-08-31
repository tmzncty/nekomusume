use std::{
    fs,
    io::{BufRead, BufReader, Read},
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neko-cli-{name}-{}", std::process::id()))
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
            "5",
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

fn signal_term(child: &Child) {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
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
fn authenticated_tcp_and_udp_loopback_probe_starts_after_ready() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40080u16), ("udp", 40081u16)] {
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
fn invalid_bind_never_emits_ready() {
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
        assert!(
            log.contains("lifecycle_state=STOPPED readiness=false"),
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
fn executable_loopback_udp_blackhole_tcp_resume() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("failover-server");
    let cp = tmp("failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp = 40089u16;
    let tcp = 40090u16;
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
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(150));
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
        ])
        .output()
        .unwrap();
    let server_out = server.wait_with_output().unwrap();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        out.status.success(),
        "status={:?} stdout={} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("udp_blackhole=true"));
    let client_log = String::from_utf8_lossy(&out.stdout);
    let server_log = String::from_utf8_lossy(&server_out.stdout);
    for event in [
        "udp_authenticated",
        "udp_blackhole_injected",
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
    assert!(
        server_log.contains("duplicates=0"),
        "unexpected dedup count: {server_log}"
    );
    assert!(
        server_log.contains(&format!("bytes_hex={}", "78".repeat(48))),
        "incomplete ordered bytes: {server_log}"
    );
}

#[test]
fn failover_udp_handshake_timeout_reports_last_success_stage() {
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
