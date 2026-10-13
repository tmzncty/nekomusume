//! W5 (D077) multi-stream bulk assertions: deterministic global sharding
//! across socket-per-stream connections, per-stream SHA receipts, aggregate
//! conjunction confirm, flag validation, and N=1 dispatching to the legacy
//! byte-compatible single path.
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

static PORT_LOCK: Mutex<()> = Mutex::new(());

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_neko-cli")
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("neko-w5-{name}-{}", std::process::id()))
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

fn lease_port() -> u16 {
    for port in 40080..=40100 {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", port)) {
            drop(l);
            return port;
        }
    }
    panic!("no free port in 40080-40100");
}

struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The stream-count domain is pinned: only {1,2,4,8} and the refusal names
/// the domain (W5 phase 1: fixed counts, no adaptive selection).
#[test]
fn streams_flag_is_validated() {
    for bad in ["3", "5", "0", "16"] {
        let out = Command::new(bin())
            .env("NEKO_MEASUREMENT", "1")
            .args([
                "bulk-client",
                "--transport",
                "tcp",
                "--addr",
                "127.0.0.1:40080",
                "--record-bytes",
                "1170",
                "--bytes-total",
                "46800",
                "--streams",
                bad,
                "--server-key",
                "ababababababababababababababababababababababababababababababababab",
            ])
            .output()
            .unwrap();
        assert_ne!(out.status.code(), Some(0), "--streams {bad} must reject");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("streams must be one of 1, 2, 4, 8"),
            "refusal must name the domain: {stderr}"
        );
    }
    // fewer records than streams is a configuration contradiction
    let out = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--transport",
            "tcp",
            "--addr",
            "127.0.0.1:40080",
            "--record-bytes",
            "1170",
            "--bytes-total",
            "2340", // 2 records
            "--streams",
            "4",
            "--server-key",
            "ababababababababababababababababababababababababababababababababab",
        ])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("fewer records than streams"),
        "contradiction must be named"
    );
}

/// Full N=4 transfer on loopback: global sharding takes effect (per-stream
/// SHA digests differ — each shard covers a distinct global range), the
/// aggregate receipt is the conjunction, and counts/bytes reconcile.
#[test]
fn multistream_transfer_completes_on_loopback() {
    let _guard = PORT_LOCK.lock().unwrap();
    let id_s = tmp("srv");
    let id_c = tmp("cli");
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
    let client_key = keygen(&id_c);
    let port = lease_port();
    // 40 records x 1170B = 46800B total; 4 streams -> 10 records each.
    let total = "46800";

    let server = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-server",
            "--transport",
            "tcp",
            "--port",
            &port.to_string(),
            "--identity",
            id_s.to_str().unwrap(),
            "--client-key",
            &client_key,
            "--record-bytes",
            "1170",
            "--bytes-total",
            total,
            "--streams",
            "4",
            "--duration",
            "60",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut server = Guard(server);
    let stdout = server.0.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let mut srvpub = String::new();
    for _ in 0..50 {
        match lines.next() {
            Some(Ok(line)) => {
                if let Some(key) = line.strip_prefix("server_public_key=") {
                    srvpub = key.trim().to_string();
                    break;
                }
            }
            _ => break,
        }
    }
    assert_eq!(srvpub.len(), 64, "server key hex must be 32 bytes");

    let out = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--transport",
            "tcp",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--server-key",
            &srvpub,
            "--identity",
            id_c.to_str().unwrap(),
            "--record-bytes",
            "1170",
            "--bytes-total",
            total,
            "--streams",
            "4",
            "--duration",
            "60",
        ])
        .output()
        .unwrap();
    let cstdout = String::from_utf8_lossy(&out.stdout);
    let cstderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "N=4 client failed: {cstderr}\n{cstdout}"
    );
    assert!(
        cstdout.contains("streams=4 sent_records=40 sent_bytes=46800"),
        "aggregate counts wrong: {cstdout}"
    );
    assert!(
        cstdout.contains("all_confirmed=true"),
        "every per-stream receipt must confirm: {cstdout}"
    );

    // drain server stdout after kill: per-stream lines + aggregate line
    drop(server);
    let mut srv_lines = Vec::new();
    for l in lines.by_ref().flatten() {
        srv_lines.push(l);
    }
    let stream_shas: Vec<String> = srv_lines
        .iter()
        .filter_map(|l| l.strip_prefix("bulk_server_stream"))
        .filter_map(|l| l.split("sha256=").nth(1).map(|s| s.to_string()))
        .collect();
    assert_eq!(stream_shas.len(), 4, "four per-stream lines: {srv_lines:?}");
    let mut distinct = stream_shas.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        4,
        "shard digests must differ (global sharding)"
    );
    let aggregate = srv_lines
        .iter()
        .find(|l| l.starts_with("bulk_server_summary"))
        .expect("aggregate line");
    assert!(
        aggregate.contains("\"streams\":4") && aggregate.contains("\"records\":40"),
        "aggregate must reconcile: {aggregate}"
    );
    assert!(
        aggregate.contains("\"per_stream\":["),
        "aggregate must carry the conjunction breakdown"
    );
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
}

/// --streams 1 dispatches to the legacy single path: the summary line keeps
/// the exact legacy shape (no streams= token, server_receipt field name).
#[test]
fn streams_one_is_legacy_shape() {
    let _guard = PORT_LOCK.lock().unwrap();
    let id_s = tmp("s1s");
    let id_c = tmp("s1c");
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
    let client_key = keygen(&id_c);
    let port = lease_port();
    let total = "23400"; // 20 records x 1170

    let server = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-server",
            "--transport",
            "tcp",
            "--port",
            &port.to_string(),
            "--identity",
            id_s.to_str().unwrap(),
            "--client-key",
            &client_key,
            "--record-bytes",
            "1170",
            "--bytes-total",
            total,
            "--duration",
            "60",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut server = Guard(server);
    let stdout = server.0.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let mut srvpub = String::new();
    for _ in 0..50 {
        match lines.next() {
            Some(Ok(line)) => {
                if let Some(key) = line.strip_prefix("server_public_key=") {
                    srvpub = key.trim().to_string();
                    break;
                }
            }
            _ => break,
        }
    }
    assert_eq!(srvpub.len(), 64);

    let out = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--transport",
            "tcp",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--server-key",
            &srvpub,
            "--identity",
            id_c.to_str().unwrap(),
            "--record-bytes",
            "1170",
            "--bytes-total",
            total,
            "--streams",
            "1",
            "--duration",
            "60",
        ])
        .output()
        .unwrap();
    let cstdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "streams=1 client failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        cstdout.contains("bulk_client_summary sent_records=20 sent_bytes=23400"),
        "legacy summary shape must be preserved: {cstdout}"
    );
    assert!(
        !cstdout.contains("all_confirmed"),
        "streams=1 must not print multi-stream fields: {cstdout}"
    );
    assert!(
        cstdout.contains("\"ok\":true"),
        "legacy receipt must confirm: {cstdout}"
    );
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
}

/// Fail-closed: a multi-stream client against a legacy (single-connection)
/// server cannot complete — the second sequential connection gets no
/// handshake and the client must die nonzero rather than degrade silently.
#[test]
fn multistream_client_fails_closed_against_legacy_server() {
    let _guard = PORT_LOCK.lock().unwrap();
    let id_s = tmp("fcs");
    let id_c = tmp("fcc");
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
    let client_key = keygen(&id_c);
    let port = lease_port();
    let total = "46800";

    let server = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-server",
            "--transport",
            "tcp",
            "--port",
            &port.to_string(),
            "--identity",
            id_s.to_str().unwrap(),
            "--client-key",
            &client_key,
            "--record-bytes",
            "1170",
            "--bytes-total",
            total, // legacy server: no --streams
            "--duration",
            "30",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut server = Guard(server);
    let stdout = server.0.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let mut srvpub = String::new();
    for _ in 0..50 {
        match lines.next() {
            Some(Ok(line)) => {
                if let Some(key) = line.strip_prefix("server_public_key=") {
                    srvpub = key.trim().to_string();
                    break;
                }
            }
            _ => break,
        }
    }
    assert_eq!(srvpub.len(), 64);

    let out = Command::new(bin())
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--transport",
            "tcp",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--server-key",
            &srvpub,
            "--identity",
            id_c.to_str().unwrap(),
            "--record-bytes",
            "1170",
            "--bytes-total",
            total,
            "--streams",
            "4",
            "--duration",
            "30",
        ])
        .output()
        .unwrap();
    assert_ne!(
        out.status.code(),
        Some(0),
        "multi-stream client must not silently succeed against a single-stream server"
    );
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
}
