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

/// Minimal structural validator: balanced braces/brackets outside strings, an
/// even number of unescaped quotes, and no empty object/array slots. This is the
/// property `contains`-style field checks cannot see - a dropped separator or a
/// missing bracket still leaves every field fragment present.
fn assert_structurally_valid_json(s: &str) {
    let bytes = s.as_bytes();
    let (mut braces, mut brackets) = (0i64, 0i64);
    let mut in_string = false;
    let mut escaped = false;
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'\\' if in_string => escaped = !escaped,
            _ => escaped = false,
        }
        if *b == b'"' && !escaped {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match b {
            b'{' => braces += 1,
            b'}' => braces -= 1,
            b'[' => brackets += 1,
            b']' => brackets -= 1,
            _ => {}
        }
        assert!(
            braces >= 0 && brackets >= 0,
            "unbalanced close at byte {i} in {s}"
        );
    }
    assert!(!in_string, "unterminated string in {s}");
    assert_eq!(braces, 0, "unbalanced braces in {s}");
    assert_eq!(brackets, 0, "unbalanced brackets in {s}");
    // Only patterns that cannot occur in well-formed JSON. `},`, `],` and `}]}`
    // are all legitimate (between array elements, or closing nested containers),
    // so they are not checked here.
    for bad in [",,", ",}", ",]", "{,", "[,"] {
        assert!(!s.contains(bad), "empty slot {bad:?} in {s}");
    }
}

#[test]
fn json_capabilities_is_well_formed_and_is_exactly_the_published_line() {
    // Every existing assertion on this output is a `contains` on one field
    // fragment, so the JSON's STRUCTURE was unwitnessed: dropping a separator
    // between two top-level keys, removing the `commands` array bracket, adding
    // or removing a closing brace, or reordering the top-level keys all left
    // every fragment present and every test passing - yet the output is no
    // longer the documented JSON contract and real consumers cannot parse it.
    let out = run(&argv(&["capabilities", "--json"]));
    assert!(out.ok, "{:?}", out.stderr);
    let line = out.stdout.trim_end_matches('\n');

    // The generated line is exactly this, byte for byte.
    let expected = format!(
        concat!(
            "{{\"schema\":\"nekomusume.capabilities.v1\",",
            "\"package_version\":\"{}\",\"target_os\":\"{}\",\"target_arch\":\"{}\",",
            "\"secret_free\":true,",
            "\"defaults\":{{\"bytes\":32,\"count\":1,\"duration_seconds\":10}},",
            "\"limits\":{{\"bytes_max\":1200,\"count_max\":64,\"duration_seconds_max\":30,",
            "\"workload_duration_seconds_max\":600,\"port_min\":40080,\"port_max\":40100}},",
            "\"commands\":[",
            "{{\"name\":\"client\",\"maturity\":\"research\"}},",
            "{{\"name\":\"server\",\"maturity\":\"research\"}},",
            "{{\"name\":\"probe\",\"maturity\":\"research\"}},",
            "{{\"name\":\"health-observe\",\"maturity\":\"experimental\"}},",
            "{{\"name\":\"failover\",\"maturity\":\"experimental\"}},",
            "{{\"name\":\"multistream\",\"maturity\":\"experimental\"}},",
            "{{\"name\":\"scheduler-fairness\",\"maturity\":\"fixture\"}},",
            "{{\"name\":\"key-update\",\"maturity\":\"fixture\"}},",
            "{{\"name\":\"periodic-server\",\"maturity\":\"research\"}},",
            "{{\"name\":\"periodic-client\",\"maturity\":\"research\"}},",
            "{{\"name\":\"lab\",\"maturity\":\"fixture\"}},",
            "{{\"name\":\"workload\",\"maturity\":\"fixture\"}},",
            "{{\"name\":\"endpoint-rebind-server\",\"maturity\":\"experimental\"}},",
            "{{\"name\":\"endpoint-rebind-client\",\"maturity\":\"experimental\"}},",
            "{{\"name\":\"keygen\",\"maturity\":\"utility\"}},",
            "{{\"name\":\"capabilities\",\"maturity\":\"utility\"}}",
            "]}}"
        ),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    assert_eq!(
        line, expected,
        "the capabilities JSON is a published contract"
    );
    // And it really is well-formed JSON, not merely the right fragments.
    assert_structurally_valid_json(line);
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
    // The WHOLE grouped command line. Only `research=` and `aliases=` were
    // asserted before, so the `experimental=`, `fixtures=` and `utilities=`
    // groups - their labels, their membership and their order - were free: a
    // renamed group label, a command moved between groups, or a command dropped
    // from the line entirely all left the pre-existing assertions passing.
    // (The JSON report does pin each command's maturity; this is the separate
    // human-facing grouping, which is a distinct surface.)
    assert!(
        s.contains(concat!(
            "commands research=client,server,probe,periodic-server,periodic-client ",
            "experimental=health-observe,failover,multistream,",
            "endpoint-rebind-server,endpoint-rebind-client ",
            "fixtures=scheduler-fairness,key-update,lab,workload ",
            "utilities=keygen,capabilities ",
            "aliases=failover-server,failover-client"
        )),
        "{s}"
    );
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

/// Base arguments for the `--payload-file` guards. `benchmark_payload` is called
/// BEFORE any socket is opened (`client()` calls it ahead of
/// `TcpStream::connect_timeout`), so every guard below is reachable with no
/// server listening - the same "pre-connect argument bound" class as the rest of
/// this file. `--identity` points at a temp path so no default identity file is
/// ever created in the working directory.
fn payload_args(
    bytes: &str,
    count: &str,
    transport: &str,
    json: bool,
    payload: &str,
    identity: &str,
) -> Vec<String> {
    let mut v = argv(&[
        "client",
        "--transport",
        transport,
        "--port",
        "40085",
        "--addr",
        "127.0.0.1:40085",
        "--server-key",
        PEER,
        "--bytes",
        bytes,
        "--count",
        count,
        "--identity",
        identity,
        "--payload-file",
        payload,
    ]);
    if json {
        v.push("--json".into());
    }
    v
}

#[test]
fn payload_file_requires_tcp_a_single_exchange_and_json_mode() {
    let f = tmp("payload-guard25.bin");
    let id = tmp("payload-guard.id");
    std::fs::write(&f, vec![b'x'; 25]).unwrap();
    let path = f.to_str().unwrap();
    let idp = id.to_str().unwrap();

    // The flag sits behind a THREE-way conjunction, so each clause is checked on
    // its own: weakening any single one (`||` -> `&&`, or dropping a clause)
    // must be visible. All three share one message.
    const MSG: &str = "--payload-file requires TCP, --count 1, and --json";
    for (label, args) in [
        (
            "transport udp",
            payload_args("25", "1", "udp", true, path, idp),
        ),
        ("count 2", payload_args("25", "2", "tcp", true, path, idp)),
        (
            "json omitted",
            payload_args("25", "1", "tcp", false, path, idp),
        ),
    ] {
        let out = run(&args);
        assert_eq!(
            out.code,
            Some(2),
            "{label}: code={:?} {}",
            out.code,
            out.stderr
        );
        assert!(out.stderr.contains(MSG), "{label}: {}", out.stderr);
    }

    // Control: with every clause satisfied the guard must NOT fire. The run then
    // proceeds past it and fails later for an unrelated reason (nothing is
    // listening), so the guard message must be absent.
    let out = run(&payload_args("25", "1", "tcp", true, path, idp));
    assert!(
        !out.stderr.contains(MSG),
        "guard fired although all three clauses hold: {}",
        out.stderr
    );

    let _ = std::fs::remove_file(&f);
    let _ = std::fs::remove_file(&id);
}

#[test]
fn payload_file_must_be_a_regular_file_of_exactly_the_requested_length() {
    let p25 = tmp("payload-len25.bin");
    let p24 = tmp("payload-len24.bin");
    let p26 = tmp("payload-len26.bin");
    let dir = tmp("payload-dir");
    let absent = tmp("payload-absent.bin");
    let id = tmp("payload-len.id");
    std::fs::write(&p25, vec![b'x'; 25]).unwrap();
    std::fs::write(&p24, vec![b'x'; 24]).unwrap();
    std::fs::write(&p26, vec![b'x'; 26]).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::remove_file(&absent);
    let idp = id.to_str().unwrap();

    // A path that does not exist fails at the metadata step, which has its OWN
    // message - distinct from the shape/length guard below.
    let out = run(&payload_args(
        "25",
        "1",
        "tcp",
        true,
        absent.to_str().unwrap(),
        idp,
    ));
    assert_eq!(out.code, Some(2), "code={:?} {}", out.code, out.stderr);
    assert!(
        out.stderr.contains("payload file unreadable"),
        "absent file: {}",
        out.stderr
    );

    // Not a regular file (a directory), too short, and too long all trip the
    // same guard - checked in both directions so `!=` cannot become `>` or `<`.
    for (label, bytes, payload) in [
        ("a directory", "25", dir.to_str().unwrap()),
        ("one byte short", "25", p24.to_str().unwrap()),
        ("one byte long", "24", p25.to_str().unwrap()),
        ("--bytes too small", "25", p26.to_str().unwrap()),
    ] {
        let out = run(&payload_args(bytes, "1", "tcp", true, payload, idp));
        assert_eq!(
            out.code,
            Some(2),
            "{label}: code={:?} {}",
            out.code,
            out.stderr
        );
        assert!(
            out.stderr
                .contains("payload file length must equal bounded --bytes"),
            "{label}: {}",
            out.stderr
        );
    }

    // The exact-length, regular-file shape is accepted by these guards (the run
    // proceeds past them and fails later, since nothing is listening).
    let out = run(&payload_args(
        "25",
        "1",
        "tcp",
        true,
        p25.to_str().unwrap(),
        idp,
    ));
    assert!(
        !out.stderr
            .contains("payload file length must equal bounded --bytes"),
        "an exactly-sized regular file was rejected: {}",
        out.stderr
    );

    for p in [&p25, &p24, &p26, &absent, &id] {
        let _ = std::fs::remove_file(p);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The metadata step and the `open` step BOTH emit `payload file unreadable`, at
/// two separate `unwrap_or_else` sites. Only a file that is stat-able with the
/// right length but cannot be opened distinguishes them, so this exercises the
/// `open` site. It needs a non-root runner (root can open mode 000), and skips
/// itself rather than failing if the environment cannot express the case.
#[test]
#[cfg(unix)]
fn payload_file_reports_the_open_failure_site_separately() {
    use std::os::unix::fs::PermissionsExt;
    let f = tmp("payload-unreadable25.bin");
    let id = tmp("payload-open.id");
    std::fs::write(&f, vec![b'x'; 25]).unwrap();
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o000)).unwrap();
    let path = f.to_str().unwrap();
    let idp = id.to_str().unwrap();

    if std::fs::read(&f).is_ok() {
        // Cannot make the open fail here (running as root, or an ACL grants
        // access). The sub-case is not exercisable in this environment.
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600)).unwrap();
        let _ = std::fs::remove_file(&f);
        let _ = std::fs::remove_file(&id);
        return;
    }

    // Stat succeeds (correct length, regular file) so the shape/length guard does
    // NOT fire; only the open can fail.
    let out = run(&payload_args("25", "1", "tcp", true, path, idp));
    assert_eq!(out.code, Some(2), "code={:?} {}", out.code, out.stderr);
    assert!(
        out.stderr.contains("payload file unreadable"),
        "open-failure site: {}",
        out.stderr
    );
    assert!(
        !out.stderr
            .contains("payload file length must equal bounded --bytes"),
        "the shape guard fired before the open: {}",
        out.stderr
    );

    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600)).unwrap();
    let _ = std::fs::remove_file(&f);
    let _ = std::fs::remove_file(&id);
}
