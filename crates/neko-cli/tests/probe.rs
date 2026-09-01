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
fn executable_loopback_controlled_udp_stop_tcp_resume() {
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
            "--diagnostic",
            "--experiment-id",
            "primary-a-server",
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
            "--diagnostic",
            "--experiment-id",
            "primary-a-client",
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
    assert!(String::from_utf8_lossy(&out.stdout).contains("controlled_udp_stop=true"));
    let client_log = String::from_utf8_lossy(&out.stdout);
    let server_log = String::from_utf8_lossy(&server_out.stdout);
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
    assert!(server_log.contains("\"udp_port\":40089"));
    assert!(server_log.contains("\"tcp_port\":40090"));
    assert!(server_log.contains("\"max_seconds\":5"));
    assert!(client_log.contains("\"application_bytes_total\":48"));
    assert!(
        server_log.contains(&format!("bytes_hex={}", "78".repeat(48))),
        "incomplete ordered bytes: {server_log}"
    );
}

#[test]
fn executable_loopback_health_threshold_drives_udp_to_tcp() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("health-failover-server");
    let cp = tmp("health-failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp = 40086u16;
    let tcp = 40087u16;
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
            "--automatic-health-failover",
            "--cold-health-failover",
            "--diagnostic",
            "--experiment-id",
            "health-failover-client",
        ])
        .output()
        .unwrap();
    let server_out = server.wait_with_output().unwrap();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let server_log = String::from_utf8_lossy(&server_out.stdout);
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        server_out.status.success(),
        "stdout={server_log} stderr={}",
        String::from_utf8_lossy(&server_out.stderr)
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
fn executable_loopback_warm_tcp_precedes_udp_failure_and_data() {
    let _lock = FAILOVER_TEST_LOCK.lock().unwrap();
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("warm-failover-server");
    let cp = tmp("warm-failover-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp = 40084u16;
    let tcp = 40085u16;
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
            "warm-failover-server",
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
            "--automatic-health-failover",
            "--diagnostic",
            "--experiment-id",
            "warm-failover-client",
        ])
        .output()
        .unwrap();
    let server_out = server.wait_with_output().unwrap();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    let client_log = String::from_utf8_lossy(&out.stdout);
    let server_log = String::from_utf8_lossy(&server_out.stdout);
    assert!(
        out.status.success(),
        "stdout={client_log} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        server_out.status.success(),
        "stdout={server_log} stderr={}",
        String::from_utf8_lossy(&server_out.stderr)
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
    let warm = client_log.find("carrier_event name=tcp_warm").unwrap();
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
fn udp_reply_cessation_seam_is_bounded_and_off_by_default() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
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
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("UDP reply cessation point outside 1..count")
    );
}

#[test]
fn first_udp_selection_loss_recovers_from_same_peer_duplicate_hello() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("retry-server");
    let cp = tmp("retry-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp = 40091u16;
    let tcp = 40092u16;
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
            "2",
            "--bytes",
            "13",
            "--duration",
            "4",
            "--send-late-udp-hello",
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
    let server = server.wait_with_output().unwrap();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        client.status.success(),
        "client stdout={} stderr={}",
        String::from_utf8_lossy(&client.stdout),
        String::from_utf8_lossy(&client.stderr)
    );
    assert!(
        server.status.success(),
        "server stdout={} stderr={}",
        String::from_utf8_lossy(&server.stdout),
        String::from_utf8_lossy(&server.stderr)
    );
    let log = String::from_utf8_lossy(&server.stdout);
    assert!(log.contains("udp_selection_dropped"), "{log}");
    assert!(log.contains("udp_selection_retried"), "{log}");
    // The duplicate pending hello and a late post-auth hello cannot restart
    // negotiation or reset ResumeGuard/session/path/delivery state.
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
fn first_udp_noise_response_loss_replays_without_resetting_session_state() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("noise-retry-server");
    let cp = tmp("noise-retry-client");
    let sk = key(bin, &sp);
    let ck = key(bin, &cp);
    let udp = 40093u16;
    let tcp = 40094u16;
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
    let server = server.wait_with_output().unwrap();
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
    assert!(
        client.status.success(),
        "client stdout={} stderr={}",
        String::from_utf8_lossy(&client.stdout),
        String::from_utf8_lossy(&client.stderr)
    );
    assert!(
        server.status.success(),
        "server stdout={} stderr={}",
        String::from_utf8_lossy(&server.stdout),
        String::from_utf8_lossy(&server.stderr)
    );
    let client_log = String::from_utf8_lossy(&client.stdout);
    let server_log = String::from_utf8_lossy(&server.stdout);
    assert!(
        server_log.contains("udp_noise_response_dropped"),
        "{server_log}"
    );
    assert!(
        server_log.contains("udp_noise_response_retried"),
        "{server_log}"
    );
    assert!(
        server_log.contains("udp_noise_response_delayed_duplicate_sent"),
        "{server_log}"
    );
    assert!(
        client_log.contains("duplicate_noise_response"),
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

static FAILOVER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static PERIODIC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn periodic_test_port() -> u16 {
    40088
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
    let _serial = PERIODIC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-delay-server");
    let cp = tmp("periodic-delay-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port = periodic_test_port();
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
fn periodic_session_accounts_missing_ack_and_fails_closed() {
    let _serial = PERIODIC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-missing-server");
    let cp = tmp("periodic-missing-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port = periodic_test_port();
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
    let _serial = PERIODIC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-duplicate-server");
    let cp = tmp("periodic-duplicate-client");
    let ck = key(bin, &cp);
    let sk = key(bin, &sp);
    let port = periodic_test_port();
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
fn periodic_server_signal_cleanup_is_bounded() {
    let _serial = PERIODIC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let sp = tmp("periodic-signal-server");
    let cp = tmp("periodic-signal-client");
    let ck = key(bin, &cp);
    let port = periodic_test_port();
    let server = start_periodic_server(bin, port, &sp, &ck, &[]);
    signal_term(&server.child);
    let (status, log) = finish_server(server);
    assert!(status.success(), "{log}");
    assert!(log.contains("cleanup=verified"), "{log}");
    let _ = fs::remove_file(sp);
    let _ = fs::remove_file(cp);
}

#[test]
fn warm_readiness_failures_close_before_admission_or_application_data() {
    let _lock = FAILOVER_TEST_LOCK.lock().unwrap();
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
    ] {
        let sp = tmp(&format!("negative-{name}-server"));
        let cp = tmp(&format!("negative-{name}-client"));
        let sk = key(bin, &sp);
        let ck = key(bin, &cp);
        let udp = 40084u16;
        let tcp = 40085u16;
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
                server_seam,
            ])
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
                seam,
            ])
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
