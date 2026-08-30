use std::{net::TcpListener, process::Command, thread, time::Duration};

#[test]
fn bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let bin = env!("CARGO_BIN_EXE_neko-cli");
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
