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
    // The fixture-scenarios line in FULL. Only the trailing `scope=` value was
    // asserted, so the label, `command=`, `scenario=` and `maturity=` - and the
    // line's entire presence - were free: renaming the label, retargeting the
    // scenario, reclassifying the maturity, or dropping the whole prefix all left
    // the old assertion passing.
    assert!(
        s.contains(concat!(
            "fixture_scenarios command=lab scenario=reliable-udp maturity=fixture ",
            "scope=bounded-loopback-controlled-suppression-manager-decision-only-no-tcp-socket"
        )),
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

/// A `failover-client` invocation from the given raw flags. Its port/delay/recovery
/// guards all run BEFORE any socket is created and before `--server-key` is
/// consulted, so none of the cases below need a peer.
fn failover_client_args(extra: &[&str]) -> Vec<String> {
    let mut v = argv(&["failover-client"]);
    v.extend(extra.iter().map(|s| (*s).to_string()));
    v
}

#[test]
fn numeric_flags_distinguish_parse_failures_from_range_failures() {
    // Each numeric flag has TWO failure modes with DIFFERENT messages: an
    // unparseable value (`invalid <flag>`) and an out-of-range value
    // (`<flag> outside <range>`). Only the range mode was asserted anywhere, so a
    // mutant that collapsed the two messages, or reworded the parse failure, was
    // free. The range message is asserted to be ABSENT so the two cannot merge.
    for (flag, unparseable, range_msg) in [
        ("--bytes", "abc", "bytes outside 1-1200"),
        ("--duration", "abc", "duration outside 1-30"),
        ("--count", "abc", "count outside 1-64"),
    ] {
        let out = run(&client_args(&[(flag, unparseable)]));
        assert_eq!(out.code, Some(2), "{flag}={unparseable}: {}", out.stderr);
        assert!(
            out.stderr.contains(&format!("invalid {}", &flag[2..])),
            "{flag}={unparseable} must report a parse failure: {}",
            out.stderr
        );
        assert!(
            !out.stderr.contains(range_msg),
            "{flag}={unparseable} must NOT report a range failure: {}",
            out.stderr
        );
    }
    // `--port` is parsed earlier in `common()`, so it is reachable the same way.
    let out = run(&client_args(&[("--port", "abc")]));
    assert_eq!(out.code, Some(2), "{}", out.stderr);
    assert!(out.stderr.contains("invalid port"), "{}", out.stderr);
    // A value that overflows the target integer is also a parse failure, not a
    // range failure.
    let out = run(&client_args(&[("--bytes", "99999999999999999999")]));
    assert_eq!(out.code, Some(2), "{}", out.stderr);
    assert!(out.stderr.contains("invalid bytes"), "{}", out.stderr);
}

#[test]
fn failover_client_validates_ports_and_delays_before_connecting() {
    for (label, args, msg) in [
        (
            "udp port unparseable",
            &["--udp-port", "abc"][..],
            "invalid UDP port",
        ),
        (
            "tcp port unparseable",
            &["--tcp-port", "99999"][..],
            "invalid TCP port",
        ),
        (
            "udp port below window",
            &["--udp-port", "40079"][..],
            "ports outside 40080-40100",
        ),
        (
            "tcp port above window",
            &["--tcp-port", "40101"][..],
            "ports outside 40080-40100",
        ),
        (
            "first data delay unparseable",
            &["--test-first-data-delay-ms", "abc"][..],
            "invalid first data delay",
        ),
        (
            "first data delay above bound",
            &["--test-first-data-delay-ms", "2001"][..],
            "first data delay outside 0-2000 ms",
        ),
    ] {
        let out = run(&failover_client_args(args));
        assert_eq!(out.code, Some(2), "{label}: {}", out.stderr);
        assert!(out.stderr.contains(msg), "{label}: {}", out.stderr);
    }
    // Both window edges are ACCEPTED: the run passes its own bound and stops at the
    // next guard instead, which is what separates `>=`/`<=` from `>`/`<`.
    for ok in [
        &["--udp-port", "40080"][..],
        &["--tcp-port", "40100"][..],
        &["--test-first-data-delay-ms", "2000"][..],
    ] {
        let out = run(&failover_client_args(ok));
        assert!(
            out.stderr.contains("missing --server-key"),
            "{ok:?} should pass its own bound: {}",
            out.stderr
        );
    }
}

#[test]
fn failover_recovery_flags_enforce_each_precondition_clause() {
    // `--migration-back` is guarded by a TWO-clause condition (needs
    // --automatic-health-failover AND count >= 3), so each clause is checked on its
    // own: weakening the conjunction, or dropping a clause, must be visible.
    let msg = "--migration-back requires --automatic-health-failover and count >= 3";
    for (label, args) in [
        ("neither clause", vec!["--migration-back"]),
        (
            "failover but count 2",
            vec![
                "--migration-back",
                "--automatic-health-failover",
                "--count",
                "2",
            ],
        ),
    ] {
        let out = run(&failover_client_args(&args));
        assert_eq!(out.code, Some(2), "{label}: {}", out.stderr);
        assert!(out.stderr.contains(msg), "{label}: {}", out.stderr);
    }
    // Control: both clauses hold -> the guard must NOT fire, so the run reaches a
    // later guard. Without this a guard that always fires would look pinned.
    let out = run(&failover_client_args(&[
        "--migration-back",
        "--automatic-health-failover",
        "--count",
        "3",
    ]));
    assert!(!out.stderr.contains(msg), "{}", out.stderr);
    assert!(
        out.stderr.contains("missing --server-key"),
        "{}",
        out.stderr
    );

    // `--cold-health-failover` needs only --automatic-health-failover, and has its
    // own message.
    let cmsg = "--cold-health-failover requires --automatic-health-failover";
    let out = run(&failover_client_args(&["--cold-health-failover"]));
    assert_eq!(out.code, Some(2), "{}", out.stderr);
    assert!(out.stderr.contains(cmsg), "{}", out.stderr);
    let out = run(&failover_client_args(&[
        "--cold-health-failover",
        "--automatic-health-failover",
    ]));
    assert!(!out.stderr.contains(cmsg), "{}", out.stderr);
    assert!(
        out.stderr.contains("missing --server-key"),
        "{}",
        out.stderr
    );
}

/// A `multistream` invocation from the given raw flags. Its `config()` bounds,
/// port window, address parse, mode/key requirement and identity requirement all
/// run BEFORE any socket is created, so none of the cases below needs a peer.
fn ms_args(extra: &[&str]) -> Vec<String> {
    let mut v = argv(&["multistream"]);
    v.extend(extra.iter().map(|s| (*s).to_string()));
    v
}

/// A `endpoint-rebind-<side>` invocation from the given raw flags. Its
/// count/bytes/duration clique runs before the server binds and before the client
/// creates any socket.
fn rebind_args(side: &str, extra: &[&str]) -> Vec<String> {
    let mut v = argv(&[&format!("endpoint-rebind-{side}")]);
    v.extend(extra.iter().map(|s| (*s).to_string()));
    v
}

#[test]
fn multistream_validates_every_argument_before_connecting() {
    // Each message is asserted in FULL. The pre-existing assertion in
    // tests/multistream.rs only checked the prefix `streams outside`, so the
    // numbers were free; these also cover the eleven sibling messages, which had
    // no witness at all.
    for (label, args, msg) in [
        ("streams 0", &["--streams", "0"][..], "streams outside 1-16"),
        (
            "streams 17",
            &["--streams", "17"][..],
            "streams outside 1-16",
        ),
        (
            "streams unparseable",
            &["--streams", "abc"][..],
            "invalid --streams",
        ),
        ("records 0", &["--records", "0"][..], "records outside 1-64"),
        (
            "records 65",
            &["--records", "65"][..],
            "records outside 1-64",
        ),
        ("bytes 0", &["--bytes", "0"][..], "bytes outside 1-1024"),
        (
            "bytes 1025",
            &["--bytes", "1025"][..],
            "bytes outside 1-1024",
        ),
        (
            "port 40079",
            &["--port", "40079"][..],
            "port outside 40080-40100",
        ),
        (
            "port 40101",
            &["--port", "40101"][..],
            "port outside 40080-40100",
        ),
        (
            "address unparseable",
            &["--addr", "nonsense"][..],
            "invalid --addr",
        ),
        (
            "server without client key",
            &["--mode", "server"][..],
            "--client-key is required",
        ),
        (
            "client without server key",
            &["--mode", "client"][..],
            "--server-key is required",
        ),
        (
            "mode neither",
            &["--mode", "bogus"][..],
            "mode must be server or client",
        ),
        (
            "client key wrong length",
            &["--mode", "server", "--client-key", "ab"][..],
            "key must be 32-byte hex",
        ),
        (
            "client key not hex",
            &[
                "--mode",
                "server",
                "--client-key",
                "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
            ][..],
            "invalid key hex",
        ),
        (
            "no identity",
            &["--mode", "client", "--server-key", PEER][..],
            "--identity is required",
        ),
    ] {
        let out = run(&ms_args(args));
        assert_eq!(
            out.code,
            Some(2),
            "{label}: code={:?} {}",
            out.code,
            out.stderr
        );
        assert!(
            out.stderr.contains(msg),
            "{label}: {} (want {msg})",
            out.stderr
        );
    }
    // Boundary values that are INSIDE each window are accepted: the run gets past
    // every argument guard and fails later reaching for a peer, which is what
    // separates an inclusive bound from an exclusive one.
    for ok in [
        &["--streams", "1"][..],
        &["--streams", "16"][..],
        &["--records", "64"][..],
        &["--bytes", "1024"][..],
        &["--port", "40080"][..],
        &["--port", "40100"][..],
    ] {
        let mut args = vec!["--mode", "client", "--server-key", PEER];
        args.extend_from_slice(ok);
        let out = run(&ms_args(&args));
        assert!(
            out.stderr.contains("--identity is required"),
            "{ok:?} should pass its bound: {}",
            out.stderr
        );
    }
}

#[test]
fn endpoint_rebind_requires_the_documented_clique_on_both_sides() {
    // A FIVE-clause condition (`count != 2 || bytes == 0 || bytes > MAX ||
    // secs == 0 || secs > MAX_DURATION`) with a single message, so each clause is
    // exercised on its own and the message is issued by BOTH sides.
    const MSG: &str = "endpoint rebind requires count=2, bytes=1-1170, duration=1-30";
    for side in ["server", "client"] {
        for (label, args) in [
            ("count 1", vec!["--count", "1"]),
            ("count 3", vec!["--count", "3"]),
            ("bytes 0", vec!["--count", "2", "--bytes", "0"]),
            ("bytes 1171", vec!["--count", "2", "--bytes", "1171"]),
            ("duration 0", vec!["--count", "2", "--duration", "0"]),
            ("duration 31", vec!["--count", "2", "--duration", "31"]),
        ] {
            let out = run(&rebind_args(side, &args));
            assert_eq!(out.code, Some(2), "{side} {label}: {}", out.stderr);
            assert!(out.stderr.contains(MSG), "{side} {label}: {}", out.stderr);
        }
        // Control: the fully-legal clique does NOT fire the guard - the run stops at
        // the next requirement instead, which differs per side.
        let expected_next = if side == "server" {
            "missing --client-key"
        } else {
            "missing --server-key"
        };
        let out = run(&rebind_args(side, &["--count", "2", "--bytes", "1170"]));
        assert!(!out.stderr.contains(MSG), "{side} control: {}", out.stderr);
        assert!(
            out.stderr.contains(expected_next),
            "{side} control: {}",
            out.stderr
        );
        // Both `--bytes` edges are inside: 1 and 1170 (1200 minus the 30-byte
        // Data frame header) are accepted; 1200 used to be accepted here and
        // then panicked the client at seal time.
        for edge in ["1", "1170"] {
            let out = run(&rebind_args(side, &["--count", "2", "--bytes", edge]));
            assert!(
                !out.stderr.contains(MSG),
                "{side} --bytes {edge} should be legal: {}",
                out.stderr
            );
        }
    }
}

#[test]
fn endpoint_rebind_client_orders_its_own_guards_after_the_clique() {
    // These are only reachable once the clique holds, so they prove the ORDER of
    // the guards as well as the messages.
    let out = run(&rebind_args(
        "client",
        &["--count", "2", "--udp-port", "abc"],
    ));
    assert_eq!(out.code, Some(2), "{}", out.stderr);
    assert!(out.stderr.contains("invalid UDP port"), "{}", out.stderr);
    assert!(
        !out.stderr.contains("endpoint rebind requires"),
        "the clique must be satisfied first: {}",
        out.stderr
    );

    let out = run(&rebind_args(
        "client",
        &["--count", "2", "--addr", "nonsense"],
    ));
    assert_eq!(out.code, Some(2), "{}", out.stderr);
    assert!(out.stderr.contains("bad UDP target"), "{}", out.stderr);

    // Control: with the clique satisfied and both fields parseable, the client gets
    // as far as needing a peer key.
    let out = run(&rebind_args("client", &["--count", "2"]));
    assert!(
        out.stderr.contains("missing --server-key"),
        "{}",
        out.stderr
    );
}

/// Range guards of three socket-free commands whose rejection messages had NO test
/// witness: `lab --scenario reliable-udp`, `scheduler-fairness` and `workload`.
/// Every bound is tested on BOTH sides - the first out-of-range value must exit 2
/// with the exact message, and the last in-range value must succeed - so an
/// off-by-one in either direction, or a swapped message, is visible. Each flag is
/// passed exactly once (`parse` reads the first occurrence).
#[test]
fn socket_free_command_range_guards_reject_and_accept_at_each_bound() {
    let lab = |flag: &str, value: &str| {
        argv(&["lab", "--scenario", "reliable-udp", "--json", flag, value])
    };
    let fairness = |flag: &str, value: &str| argv(&["scheduler-fairness", flag, value]);
    let workload = |flag: &str, value: &str| argv(&["workload", flag, value]);

    let rejected: [(Vec<String>, &str); 20] = [
        (lab("--rounds", "3"), "rounds outside 4-64"),
        (lab("--rounds", "65"), "rounds outside 4-64"),
        (lab("--drop-every", "1"), "drop-every outside 0 or 2-16"),
        (lab("--drop-every", "17"), "drop-every outside 0 or 2-16"),
        (lab("--drop-ack-every", "17"), "drop-ack-every outside 0-16"),
        (lab("--settle-ms", "0"), "settle-ms outside 1-20000"),
        (lab("--settle-ms", "20001"), "settle-ms outside 1-20000"),
        (lab("--rounds", "x"), "invalid rounds"),
        (fairness("--rounds", "0"), "rounds outside 1-64"),
        (fairness("--rounds", "65"), "rounds outside 1-64"),
        (fairness("--bytes", "0"), "bytes outside 1-256"),
        (fairness("--bytes", "257"), "bytes outside 1-256"),
        (fairness("--rounds", "x"), "invalid rounds"),
        (workload("--duration", "0"), "duration outside 1-600"),
        (workload("--duration", "601"), "duration outside 1-600"),
        (workload("--concurrency", "0"), "concurrency outside 1-16"),
        (workload("--concurrency", "17"), "concurrency outside 1-16"),
        (workload("--records", "0"), "records outside 1-10000"),
        (workload("--records", "10001"), "records outside 1-10000"),
        (workload("--concurrency", "x"), "invalid concurrency"),
    ];
    for (args, message) in &rejected {
        assert_rejected(args, message);
        let out = run(args);
        assert_eq!(out.stderr, format!("neko: {message}\n"), "args={args:?}");
        assert!(
            out.stdout.is_empty(),
            "args={args:?} stdout={:?}",
            out.stdout
        );
    }

    // The in-range neighbour of every bound above is accepted.
    let accepted: [Vec<String>; 13] = [
        lab("--rounds", "4"),
        lab("--rounds", "64"),
        lab("--drop-every", "0"),
        lab("--drop-every", "2"),
        lab("--drop-every", "16"),
        lab("--drop-ack-every", "16"),
        lab("--settle-ms", "1"),
        lab("--settle-ms", "20000"),
        fairness("--rounds", "1"),
        fairness("--rounds", "64"),
        fairness("--bytes", "1"),
        fairness("--bytes", "256"),
        argv(&[
            "workload",
            "--duration",
            "1",
            "--concurrency",
            "16",
            "--records",
            "10000",
        ]),
    ];
    for args in &accepted {
        let out = run(args);
        assert!(out.ok, "args={args:?} stderr={:?}", out.stderr);
        assert!(
            out.stderr.is_empty(),
            "args={args:?} stderr={:?}",
            out.stderr
        );
    }
}

/// `probe --matrix` argument rejections. The networked matrix test in `probe.rs`
/// feeds many invalid argument lists but asserts only exit code 2 and empty stdout,
/// so every rejection reason was interchangeable: swapping two messages, dropping
/// the `--flag` name from a message, or defaulting a missing `--transport` /
/// `--ip-version` instead of refusing it all went unnoticed. Every case here is
/// rejected during argument parsing, before any socket is opened, and must print
/// exactly `neko: <reason>` on stderr and nothing on stdout.
#[test]
fn matrix_probe_argument_rejections_name_the_exact_reason() {
    const T: &str = "127.0.0.1:40080";
    let matrix = |extra: &[&str]| {
        let mut args = vec!["probe", "--matrix"];
        args.extend_from_slice(extra);
        argv(&args)
    };
    let full = |extra: &[&str]| {
        let mut args = vec![
            "probe",
            "--matrix",
            "--target",
            T,
            "--transport",
            "tcp",
            "--ip-version",
            "ipv4",
        ];
        args.extend_from_slice(extra);
        argv(&args)
    };

    let cases: [(Vec<String>, &str); 21] = [
        (full(&["--matrix"]), "duplicate --matrix"),
        (full(&["--json", "--json"]), "duplicate --json"),
        (full(&["--target", "127.0.0.1:40081"]), "duplicate --target"),
        (full(&["--transport", "udp"]), "duplicate --transport"),
        (
            full(&["--timeout-ms", "1", "--timeout-ms", "2"]),
            "duplicate --timeout-ms",
        ),
        (matrix(&["--target"]), "missing value for --target"),
        // A following flag is not taken as the value.
        (
            matrix(&["--target", "--json"]),
            "missing value for --target",
        ),
        (full(&["--bytes"]), "missing value for --bytes"),
        (full(&["--bogus"]), "unknown matrix argument: --bogus"),
        (full(&["stray"]), "unknown matrix argument: stray"),
        (
            matrix(&["--transport", "tcp", "--ip-version", "ipv4"]),
            "missing or invalid --target",
        ),
        (
            matrix(&[
                "--target",
                "nonsense",
                "--transport",
                "tcp",
                "--ip-version",
                "ipv4",
            ]),
            "missing or invalid --target",
        ),
        (
            matrix(&["--target", T, "--transport", "sctp", "--ip-version", "ipv4"]),
            "--transport must be tcp or udp",
        ),
        // Absent is refused, not defaulted.
        (
            matrix(&["--target", T, "--ip-version", "ipv4"]),
            "--transport must be tcp or udp",
        ),
        (
            matrix(&["--target", T, "--transport", "udp", "--ip-version", "ipv5"]),
            "--ip-version must be ipv4 or ipv6",
        ),
        (
            matrix(&["--target", T, "--transport", "udp"]),
            "--ip-version must be ipv4 or ipv6",
        ),
        (full(&["--timeout-ms", "x"]), "invalid --timeout-ms"),
        (full(&["--bytes", "x"]), "invalid --bytes"),
        // `reachability::validate` refusals are forwarded verbatim.
        (full(&["--timeout-ms", "5001"]), "timeout must be 1-5000 ms"),
        (full(&["--bytes", "1201"]), "payload must be 1-1200 bytes"),
        (
            matrix(&[
                "--target",
                "8.8.8.8:53",
                "--transport",
                "udp",
                "--ip-version",
                "ipv4",
            ]),
            "target must be loopback; public/WAN probing is disabled",
        ),
    ];
    for (args, message) in &cases {
        let out = run(args);
        assert_eq!(out.code, Some(2), "args={args:?} stderr={:?}", out.stderr);
        assert_eq!(out.stderr, format!("neko: {message}\n"), "args={args:?}");
        assert!(
            out.stdout.is_empty(),
            "args={args:?} stdout={:?}",
            out.stdout
        );
    }
}

/// `failover` and `endpoint-rebind` wrap `--bytes` of payload in a
/// `ProcessMessage::Data` frame (30 bytes of header) before `seal_unreliable`,
/// whose cap is `MAX_UNRELIABLE_DATAGRAM` = 1200 on the ENCODED frame. The CLI
/// used to accept up to 1200 bytes of payload, so 1171-1200 passed validation
/// and then panicked in the client (exit 134) while the server waited out its
/// timeout. The bound is now derived (1200 - 30 = 1170): 1170 must pass argument
/// validation on every side and 1171 must be refused there, before any socket.
#[test]
fn data_frame_commands_cap_bytes_at_the_sealable_payload() {
    let header = neko_session::PROCESS_DATA_HEADER_LEN;
    let cap = neko_crypto::MAX_UNRELIABLE_DATAGRAM;
    assert_eq!(header, 30);
    assert_eq!(
        cap - header,
        1170,
        "CLI bound + frame header == seal_unreliable cap"
    );

    // failover: both roles report the same derived bound.
    for role in ["server", "client"] {
        let rejected = run(&argv(&["failover", "--role", role, "--bytes", "1171"]));
        assert_eq!(rejected.code, Some(2), "{role}: {}", rejected.stderr);
        assert_eq!(
            rejected.stderr, "neko: bytes outside 1-1170\n",
            "{role} 1171 is refused at argument validation"
        );
        // 1170 passes the bytes check; the run then stops at the next missing
        // requirement, which is NOT the bytes message.
        let accepted = run(&argv(&["failover", "--role", role, "--bytes", "1170"]));
        assert_eq!(accepted.code, Some(2), "{role}: {}", accepted.stderr);
        assert!(
            !accepted.stderr.contains("bytes outside"),
            "{role} 1170 must pass the bytes check: {}",
            accepted.stderr
        );
    }
}
