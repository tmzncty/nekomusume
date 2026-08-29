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
        let _ = server.wait();
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
