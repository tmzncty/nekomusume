//! D071/D072 socks entry assertions that run on loopback only: measurement
//! gating, whitelist fail-closed (three refusal shapes), startup banner
//! hygiene (marker line, no target addresses), and the D072 global
//! concurrency cap of 16.
//!
//! The positive data path (CONNECT to a whitelisted public target) needs a
//! public egress address and is covered by the W-table first-round mesh
//! measurements, not here: every loopback address is internal-range and the
//! whitelist fail-closes the whole local space by design.
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

static PORT_LOCK: Mutex<()> = Mutex::new(());

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_neko-cli")
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("neko-socks-{name}-{}", std::process::id()))
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
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
            drop(listener);
            return port;
        }
    }
    panic!("no free port in 40080-40100");
}

struct Spawned {
    child: Option<Child>,
}

impl Drop for Spawned {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn spawn_server(args: &[&str], measurement: bool) -> Spawned {
    let mut cmd = Command::new(bin());
    cmd.args(["socks-server"]).args(args);
    if measurement {
        cmd.env("NEKO_MEASUREMENT", "1");
    }
    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    Spawned { child: Some(child) }
}

fn wait_exit(mut spawned: Spawned) -> (i32, String, String) {
    let child = spawned.child.take().unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

const HEX_KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

#[test]
fn socks_is_unreachable_without_measurement_mode() {
    let id = tmp("gate-id");
    let _ = std::fs::remove_file(&id);
    keygen(&id);
    let port = lease_port();
    let _guard = PORT_LOCK.lock().unwrap();

    for args in [
        vec![
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--client-key",
            HEX_KEY,
            "--identity",
            id.to_str().unwrap(),
            "--allow",
            "203.0.113.10:443",
        ],
        vec![],
    ] {
        let server = spawn_server(&args, false);
        let (code, _, stderr) = wait_exit(server);
        assert_ne!(
            code, 0,
            "socks-server must refuse to run without NEKO_MEASUREMENT=1"
        );
        assert!(
            stderr.contains("NEKO_MEASUREMENT"),
            "refusal must name the gate: {stderr}"
        );
    }

    let client = Command::new(bin())
        .args(["socks-client", "--listen", "10800"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (code, _, stderr) = wait_exit(Spawned {
        child: Some(client),
    });
    assert_ne!(code, 0);
    assert!(stderr.contains("NEKO_MEASUREMENT"));
    let _ = std::fs::remove_file(&id);
}

#[test]
fn whitelist_is_fail_closed() {
    let id = tmp("wl-id");
    let _ = std::fs::remove_file(&id);
    keygen(&id);
    let port = lease_port();
    let _guard = PORT_LOCK.lock().unwrap();

    // Three refusal shapes: no --allow at all, an internal-range entry, and
    // an unparseable entry. All must exit non-zero before binding.
    let bind_addr = format!("127.0.0.1:{port}");
    let id_str = id.to_str().unwrap().to_string();
    let shapes: [(&str, Vec<String>); 3] = [
        (
            "no --allow",
            vec![
                "--bind".into(),
                bind_addr.clone(),
                "--client-key".into(),
                HEX_KEY.into(),
                "--identity".into(),
                id_str.clone(),
            ],
        ),
        (
            "internal entry",
            vec![
                "--bind".into(),
                bind_addr.clone(),
                "--client-key".into(),
                HEX_KEY.into(),
                "--identity".into(),
                id_str.clone(),
                "--allow".into(),
                "10.0.0.5:80".into(),
            ],
        ),
        (
            "unparseable entry",
            vec![
                "--bind".into(),
                bind_addr.clone(),
                "--client-key".into(),
                HEX_KEY.into(),
                "--identity".into(),
                id_str.clone(),
                "--allow".into(),
                "not-an-address".into(),
            ],
        ),
    ];
    let port_holder = TcpListener::bind(("127.0.0.1", port)).unwrap();
    for (label, args) in shapes {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let server = spawn_server(&refs, true);
        let (code, _, stderr) = wait_exit(server);
        assert_ne!(code, 0, "{label}: whitelist must fail closed");
        assert!(
            stderr.contains("--allow") || stderr.contains("fail-closed"),
            "{label}: refusal must name --allow: {stderr}"
        );
    }
    drop(port_holder);
    let _ = std::fs::remove_file(&id);
}

#[test]
fn startup_banner_carries_marker_and_no_target_addresses() {
    let id = tmp("banner-id");
    let _ = std::fs::remove_file(&id);
    keygen(&id);
    let port = lease_port();
    let _guard = PORT_LOCK.lock().unwrap();

    let bind = format!("127.0.0.1:{port}");
    let server = spawn_server(
        &[
            "--bind",
            &bind,
            "--client-key",
            HEX_KEY,
            "--identity",
            id.to_str().unwrap(),
            "--allow",
            "203.0.113.10:443",
            "--allow",
            "198.51.100.7:80",
        ],
        true,
    );
    let mut guarded = server;
    let stdout = guarded.child.as_mut().unwrap().stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let marker = lines.next().unwrap().unwrap();
    assert_eq!(marker, "measurement_mode=true relaxed_limits");
    let banner = lines.next().unwrap().unwrap();
    assert!(banner.contains("\"role\":\"socks-server\""), "{banner}");
    assert!(banner.contains("\"allow_entries\":2"), "{banner}");
    // D071 decision 3: the target itself never appears — only the count.
    assert!(!banner.contains("203.0.113.10"), "{banner}");
    assert!(!banner.contains("198.51.100.7"), "{banner}");
    drop(guarded);
    let _ = std::fs::remove_file(&id);
}

#[test]
fn concurrency_cap_is_sixteen_reject_and_continue() {
    let id = tmp("cap-id");
    let _ = std::fs::remove_file(&id);
    keygen(&id);
    let port = lease_port();
    let _guard = PORT_LOCK.lock().unwrap();

    let bind = format!("127.0.0.1:{port}");
    let mut server = spawn_server(
        &[
            "--bind",
            &bind,
            "--client-key",
            HEX_KEY,
            "--identity",
            id.to_str().unwrap(),
            "--allow",
            "203.0.113.10:443",
        ],
        true,
    );
    let stdout = server.child.as_mut().unwrap().stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let _marker = lines.next().unwrap().unwrap();
    let _banner = lines.next().unwrap().unwrap();

    // 16 silent connections occupy the pre-auth slots (each blocks in the
    // staged-frame read until its 5 s deadline). The 17th must be refused
    // at TCP level: connect succeeds but the server shuts it down at once.
    let mut holders = Vec::new();
    for _ in 0..16 {
        holders.push(TcpStream::connect(("127.0.0.1", port)).unwrap());
    }
    for s in holders.iter_mut() {
        s.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    }

    let mut seventeenth = TcpStream::connect(("127.0.0.1", port)).unwrap();
    seventeenth
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut buf = [0_u8; 1];
    let outcome = seventeenth.read(&mut buf);
    match outcome {
        Ok(0) | Err(_) => { /* refused: EOF or reset, both are the D072 cap */ }
        Ok(n) => panic!("17th connection must be refused immediately, got {n} bytes"),
    }

    // Reject-and-continue (D019 form): the server stays alive through the
    // over-cap connection.
    assert!(
        server.child.as_mut().unwrap().try_wait().unwrap().is_none(),
        "server must survive an over-cap connection"
    );
    drop(server);
    let _ = std::fs::remove_file(&id);
}
