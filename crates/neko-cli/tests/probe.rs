use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
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
fn authenticated_tcp_and_udp_loopback_probe() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    for (transport, port) in [("tcp", 40080u16), ("udp", 40081u16)] {
        let sp = tmp(&format!("{transport}-server"));
        let cp = tmp(&format!("{transport}-client"));
        let sk = key(bin, &sp);
        let ck = key(bin, &cp);
        let mut server = Command::new(bin)
            .args([
                "server",
                "--transport",
                transport,
                "--port",
                &port.to_string(),
                "--identity",
                sp.to_str().unwrap(),
                "--client-key",
                &ck,
                "--duration",
                "3",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        let addr = format!("127.0.0.1:{port}");
        let out = Command::new(bin)
            .args([
                "client",
                "--transport",
                transport,
                "--port",
                &port.to_string(),
                "--addr",
                &addr,
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
        let _ = server.kill();
        let _server_out = server.wait_with_output().unwrap();
        let _ = fs::remove_file(sp);
        let _ = fs::remove_file(cp);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("probe_ok"));
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
    let mut server = Command::new(bin)
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
    let _ = server.wait();
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
    assert!(stdout.contains(r#""stage":"udp_bind""#));
    assert!(stdout.contains(r#""stage":"client_hello_sent""#));
    assert!(
        stdout.contains(r#""stage":"timeout","last_success_stage":"client_hello_sent""#),
        "{stdout}"
    );
    let _ = fs::remove_file(dir);
}
