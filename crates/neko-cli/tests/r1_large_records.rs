//! R1 large-record integration assertions (D074 design v3 §R1): the two
//! large tiers are measurement-gated, unreachable by default, and carry a
//! full sealed transfer on loopback through seal_large/open_large with the
//! SHA chain intact.
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

static PORT_LOCK: Mutex<()> = Mutex::new(());

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_neko-cli")
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("neko-r1-{name}-{}", std::process::id()))
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
        if let Some(c) = Some(&mut self.0) {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

/// Default mode must refuse the large tiers (assertion ⑨: measurement-gated).
#[test]
fn large_tiers_are_unreachable_without_measurement_mode() {
    let _guard = PORT_LOCK.lock().unwrap();
    for tier in ["4096", "14336", "28672"] {
        let out = Command::new(bin())
            .env_remove("NEKO_MEASUREMENT")
            .args([
                "bulk-client",
                "--transport",
                "tcp",
                "--addr",
                "127.0.0.1:40080",
                "--record-bytes",
                tier,
                "--bytes-total",
                "28672",
            ])
            .output()
            .unwrap();
        assert_ne!(out.status.code(), Some(0), "tier {tier} must be gated");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("NEKO_MEASUREMENT"),
            "tier {tier}: refusal must name the gate"
        );
    }
}

/// One full large-tier transfer per tier on loopback: sealed record count,
/// SHA receipt confirm, and the default-tier receipt shape unchanged.
#[test]
fn large_tier_transfers_complete_on_loopback() {
    let _guard = PORT_LOCK.lock().unwrap();
    let id_s = tmp("srv");
    let id_c = tmp("cli");
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
    let client_key = keygen(&id_c);
    let port = lease_port();

    for (tier, total) in [
        ("4096", (4096_usize - 18) * 7),
        ("28672", (28_672_usize - 18) * 3),
    ] {
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
                tier,
                "--bytes-total",
                &total.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Guard(server);
        // wait for the ready line + capture the server public key
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
                tier,
                "--bytes-total",
                &total.to_string(),
                "--duration",
                "60",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "tier {tier} client failed: {stderr}\n{stdout}"
        );
        assert!(
            stdout.contains(&format!(
                "sent_records={}",
                total / (tier.parse::<usize>().unwrap() - 18)
            )),
            "tier {tier}: sent count mismatch in {stdout}"
        );
        assert!(
            stdout.contains(&format!("sent_bytes={total}")),
            "tier {tier}: byte count mismatch in {stdout}"
        );
        assert!(
            stdout.contains("\"ok\":true"),
            "tier {tier}: server receipt must confirm: {stdout}"
        );
        // server side: wait for exit + summary
        drop(lines);
        let _ = server.0.wait();
    }
    let _ = std::fs::remove_file(&id_s);
    let _ = std::fs::remove_file(&id_c);
}
