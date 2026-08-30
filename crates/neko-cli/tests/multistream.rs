use std::{fs, net::TcpListener, path::PathBuf, process::Command, thread, time::Duration};

fn identity(bin: &str, name: &str) -> (PathBuf, String) {
    let path = std::env::temp_dir().join(format!("neko-multistream-{name}-{}", std::process::id()));
    let out = Command::new(bin)
        .args(["keygen", "--identity", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let public = String::from_utf8_lossy(&out.stdout)
        .trim()
        .strip_prefix("client_public_key=")
        .unwrap()
        .to_owned();
    (path, public)
}

#[test]
fn bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity, server_key) = identity(bin, "server");
    let (client_identity, client_key) = identity(bin, "client");
    let server = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "server",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "3",
            "--records",
            "4",
            "--bytes",
            "17",
            "--identity",
            server_identity.to_str().unwrap(),
            "--client-key",
            &client_key,
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(50));
    let client = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "client",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "3",
            "--records",
            "4",
            "--bytes",
            "17",
            "--identity",
            client_identity.to_str().unwrap(),
            "--server-key",
            &server_key,
        ])
        .output()
        .unwrap();
    assert!(
        client.status.success(),
        "{}",
        String::from_utf8_lossy(&client.stderr)
    );
    let status = server.wait_with_output().unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let evidence = String::from_utf8(client.stdout).unwrap();
    assert!(evidence.contains("\"ok\":true"));
    assert!(evidence.contains("\"streams\":3"));
    assert!(evidence.contains("\"records\":12"));
    assert!(evidence.contains("\"payload_bytes\":204"));
    let _ = fs::remove_file(server_identity);
    let _ = fs::remove_file(client_identity);
}

#[test]
fn unauthorized_client_is_rejected_by_allowlist() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity, server_key) = identity(bin, "negative-server");
    let (allowed_identity, allowed_key) = identity(bin, "allowed");
    let (unauthorized_identity, _) = identity(bin, "unauthorized");
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let server = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "server",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            server_identity.to_str().unwrap(),
            "--client-key",
            &allowed_key,
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(50));
    let client = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "client",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            unauthorized_identity.to_str().unwrap(),
            "--server-key",
            &server_key,
        ])
        .output()
        .unwrap();
    assert!(!client.status.success());
    let status = server.wait_with_output().unwrap();
    assert!(!status.status.success());
    for path in [server_identity, allowed_identity, unauthorized_identity] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn multistream_rejects_unbounded_payload_before_connecting() {
    let out = Command::new(env!("CARGO_BIN_EXE_neko-cli"))
        .args([
            "multistream",
            "--mode",
            "client",
            "--streams",
            "17",
            "--addr",
            "192.0.2.1:40082",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("streams outside"));
}
