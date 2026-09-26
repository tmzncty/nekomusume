//! Bounded CLI-level pinning for the capability report, the dispatch usage
//! text, and the argument-validation surface that is only observable by
//! running the binary (every rejection below exits from inside the process).
use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

/// A syntactically valid 32-byte peer key, so later checks are reached.
const PEER: &str = "abababababababababababababababababababababababababababababababab";

struct Out {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[String]) -> Out {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("the neko-cli binary must run");
    Out {
        ok: out.status.success(),
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn argv(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

fn tmp(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("neko-capabilities-{}-{name}", std::process::id()));
    p
}

/// Every bounded rejection exits with the documented status and message.
fn assert_rejected(args: &[String], message: &str) {
    let out = run(args);
    assert_eq!(out.code, Some(2), "args={args:?} stderr={:?}", out.stderr);
    assert!(
        out.stderr.contains(message),
        "args={args:?} expected {message:?} in {:?}",
        out.stderr
    );
}

/// A client invocation built from exactly one occurrence of every flag.
/// `parse` reads the FIRST occurrence of a key, so a duplicate would silently
/// shadow the value under test - which is why `over` replaces rather than adds.
fn client_args(over: &[(&str, &str)]) -> Vec<String> {
    let mut pairs: Vec<(&str, &str)> = vec![
        ("--identity", "/tmp/neko-capabilities-missing.id"),
        ("--transport", "tcp"),
        ("--port", "40080"),
        ("--addr", "127.0.0.1:9"),
        ("--server-key", PEER),
    ];
    for (k, v) in over {
        match pairs.iter_mut().find(|(pk, _)| pk == k) {
            Some(slot) => *slot = (k, v),
            None => pairs.push((k, v)),
        }
    }
    let mut out = vec!["client".to_string()];
    for (k, v) in pairs {
        out.push(k.to_string());
        out.push(v.to_string());
    }
    out
}

#[test]
fn json_capabilities_report_every_documented_field() {
    let out = run(&argv(&["capabilities", "--json"]));
    assert!(out.ok, "{:?}", out.stderr);
    let s = out.stdout;
    assert!(
        s.contains("\"schema\":\"nekomusume.capabilities.v1\""),
        "{s}"
    );
    assert!(s.contains("\"package_version\":\"0.1.0\""), "{s}");
    assert!(s.contains("\"target_os\":\"linux\""), "{s}");
    assert!(s.contains("\"target_arch\":\"x86_64\""), "{s}");
    assert!(s.contains("\"secret_free\":true"), "{s}");
    assert!(
        s.contains("\"defaults\":{\"bytes\":32,\"count\":1,\"duration_seconds\":10}"),
        "{s}"
    );
    assert!(s.contains("\"bytes_max\":1200"), "{s}");
    assert!(s.contains("\"count_max\":64"), "{s}");
    assert!(s.contains("\"duration_seconds_max\":30"), "{s}");
    assert!(s.contains("\"workload_duration_seconds_max\":600"), "{s}");
    assert!(s.contains("\"port_min\":40080"), "{s}");
    assert!(s.contains("\"port_max\":40100"), "{s}");
    for (name, maturity) in [
        ("client", "research"),
        ("server", "research"),
        ("probe", "research"),
        ("health-observe", "experimental"),
        ("failover", "experimental"),
        ("multistream", "experimental"),
        ("scheduler-fairness", "fixture"),
        ("key-update", "fixture"),
        ("periodic-server", "research"),
        ("periodic-client", "research"),
        ("lab", "fixture"),
        ("workload", "fixture"),
        ("endpoint-rebind-server", "experimental"),
        ("endpoint-rebind-client", "experimental"),
        ("keygen", "utility"),
        ("capabilities", "utility"),
    ] {
        assert!(
            s.contains(&format!(
                "{{\"name\":\"{name}\",\"maturity\":\"{maturity}\"}}"
            )),
            "missing {name}={maturity} in {s}"
        );
    }
}

#[test]
fn human_capabilities_state_the_limits_and_the_secret_free_property() {
    let out = run(&argv(&["capabilities"]));
    assert!(out.ok, "{:?}", out.stderr);
    let s = out.stdout;
    assert!(
        s.contains("defaults bytes=32 count=1 duration_seconds=10"),
        "{s}"
    );
    assert!(
        s.contains("limits bytes=1-1200 count=1-64 duration_seconds=1-30 workload_duration_seconds=1-600 ports=40080-40100"),
        "{s}"
    );
    assert!(
        s.contains("research=client,server,probe,periodic-server,periodic-client"),
        "{s}"
    );
    assert!(s.contains("aliases=failover-server,failover-client"), "{s}");
    assert!(s.contains("report_secret_free=true"), "{s}");
    assert!(
        s.contains(
            "scope=bounded-loopback-controlled-suppression-manager-decision-only-no-tcp-socket"
        ),
        "{s}"
    );
}

#[test]
fn capabilities_accepts_only_the_json_flag() {
    assert!(run(&argv(&["capabilities", "--json"])).ok);
    assert!(run(&argv(&["capabilities"])).ok);
    assert_rejected(
        &argv(&["capabilities", "--json", "--extra"]),
        "capabilities accepts only --json",
    );
    assert_rejected(
        &argv(&["capabilities", "--verbose"]),
        "capabilities accepts only --json",
    );
}

#[test]
fn usage_text_describes_the_bounded_scope() {
    let out = run(&argv(&["--help"]));
    assert!(out.ok, "{:?}", out.stderr);
    let s = out.stdout;
    assert!(s.contains("probe --matrix"), "{s}");
    assert!(s.contains("local-loopback only"), "{s}");
    assert!(s.contains("lab --scenario reliable-udp"), "{s}");
    assert!(s.contains("failover --role server|client"), "{s}");
    assert!(
        s.contains("failover-server|failover-client: legacy aliases for failover"),
        "{s}"
    );
    assert!(
        s.contains("capabilities [--json]: secret-free build, command, default, and limit report"),
        "{s}"
    );
    assert!(
        s.contains("Bounded authenticated research probe only; no proxy/tunnel behavior."),
        "{s}"
    );
    assert_rejected(&argv(&["not-a-command"]), "unknown command");
}

#[test]
fn transport_port_and_payload_bounds_are_inclusive_at_the_edges() {
    assert_rejected(
        &client_args(&[("--transport", "sctp")]),
        "transport must be tcp or udp",
    );
    // Port window: 40080 and 40100 are legal, one step outside is not.
    assert_rejected(
        &client_args(&[("--port", "40079")]),
        "port outside 40080-40100",
    );
    assert_rejected(
        &client_args(&[("--port", "40101")]),
        "port outside 40080-40100",
    );
    // Payload window: 1 and 1200 legal, 0 and 1201 not.
    assert_rejected(&client_args(&[("--bytes", "0")]), "bytes outside 1-1200");
    assert_rejected(&client_args(&[("--bytes", "1201")]), "bytes outside 1-1200");
    // Duration window: 1 and 30 legal, 0 and 31 not.
    assert_rejected(
        &client_args(&[("--duration", "0")]),
        "duration outside 1-30",
    );
    assert_rejected(
        &client_args(&[("--duration", "31")]),
        "duration outside 1-30",
    );
    // Exchange count: 1 and 64 legal, 0 and 65 not.
    assert_rejected(&client_args(&[("--count", "0")]), "count outside 1-64");
    assert_rejected(&client_args(&[("--count", "65")]), "count outside 1-64");
    // The upper edges are accepted: validation moves on to the transport
    // attempt, which fails only because nothing is listening on port 9. That is
    // what makes the windows inclusive rather than always-rejecting.
    let edge = client_args(&[
        ("--port", "40100"),
        ("--bytes", "1200"),
        ("--duration", "30"),
        ("--count", "64"),
    ]);
    let out = run(&edge);
    assert!(
        out.stderr.contains("connect failed"),
        "edges must be accepted: {:?}",
        out.stderr
    );
}

#[test]
fn peer_keys_must_be_hex_and_exactly_thirty_two_bytes() {
    // Two distinct rejections: an odd digit count never pairs up, while an
    // even count of non-hex characters fails the radix conversion.
    assert_rejected(&client_args(&[("--server-key", "zz")]), "invalid hex");
    assert_rejected(&client_args(&[("--server-key", "zzzz")]), "invalid hex");
    assert_rejected(&client_args(&[("--server-key", "abc")]), "hex length");
    // An empty value is even-length, so it reaches the size check instead.
    assert_rejected(
        &client_args(&[("--server-key", "")]),
        "peer public key must be 32 bytes",
    );
    assert_rejected(
        &client_args(&[("--server-key", "abcd")]),
        "peer public key must be 32 bytes",
    );
    // Too long is refused as well, not truncated to the first 32 bytes.
    let long = "ab".repeat(33);
    assert_rejected(
        &client_args(&[("--server-key", &long)]),
        "peer public key must be 32 bytes",
    );
}

#[test]
fn keygen_emits_lowercase_zero_padded_hex() {
    let path = tmp("keygen.id");
    let _ = std::fs::remove_file(&path);
    let out = run(&argv(&["keygen", "--identity", path.to_str().unwrap()]));
    assert!(out.ok, "{:?}", out.stderr);
    let line = out.stdout.trim();
    let key = line
        .strip_prefix("client_public_key=")
        .expect("keygen prints the client public key");
    assert_eq!(key.len(), 64, "a 32-byte key is 64 hex digits: {key}");
    assert!(
        key.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "hex must be lowercase and zero-padded, never upper-case or unpadded: {key}"
    );
    let _ = std::fs::remove_file(&path);
}
