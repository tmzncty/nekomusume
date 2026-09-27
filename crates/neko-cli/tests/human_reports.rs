//! Bounded CLI-level pinning for the HUMAN (non-JSON) report of several
//! commands. Each of these has an `else` branch next to `if json_mode(args)`, and
//! that branch is a separate output surface: the JSON side of most of them was
//! pinned earlier, but the human side was not - `scheduler_fairness_ok`,
//! `workload_ok`, `key_update_ok` and the two `lab` human reports had **zero**
//! occurrences in the whole test tree, and `endpoint_rebind_server_ok` appeared
//! only in a negative assertion.
//!
//! All of these are deterministic (verified across repeated runs) and need no
//! shared server, so the file is fast and stable.
use std::net::TcpListener;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

fn run(args: &[&str]) -> (bool, i32, String) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("the neko-cli binary must run");
    (
        out.status.success(),
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout)
            .trim_end_matches('\n')
            .to_owned(),
    )
}

#[test]
fn scheduler_fairness_human_report_is_exactly_the_published_line() {
    // `rounds=3` differs from every other value on the line (9/3/3/3), so the
    // label/source wiring is pinned even where two counters coincide.
    let (ok, code, line) = run(&["scheduler-fairness", "--rounds", "3", "--bytes", "8"]);
    assert!(ok, "code={code}");
    assert_eq!(
        line,
        "scheduler_fairness_ok rounds=3 interactive_frames=9 bulk_frames=3 \
         max_interactive_burst=3 forced_bulk_services=3"
    );
    // A second value pins `rounds` as the flag rather than a constant.
    let (ok, _, other) = run(&["scheduler-fairness", "--rounds", "5", "--bytes", "8"]);
    assert!(ok);
    assert_eq!(
        other,
        "scheduler_fairness_ok rounds=5 interactive_frames=15 bulk_frames=5 \
         max_interactive_burst=3 forced_bulk_services=5"
    );
}

#[test]
fn workload_and_key_update_human_reports_are_exactly_the_published_lines() {
    let (ok, code, line) = run(&[
        "workload",
        "--duration",
        "1",
        "--concurrency",
        "2",
        "--records",
        "3",
        "--bytes",
        "8",
    ]);
    assert!(ok, "code={code}");
    assert_eq!(
        line,
        "workload_ok duration_seconds=1 concurrency=2 records=6 application_bytes=48 cleanup=verified"
    );

    let (ok, code, line) = run(&["key-update"]);
    assert!(ok, "code={code}");
    assert_eq!(
        line,
        concat!(
            "key_update_ok exchanges=12 key_phase=1 events=",
            "session_opened,key_phase_0,",
            "exchange,exchange,exchange,exchange,exchange,exchange,",
            "key_update_committed,old_phase_rejected,",
            "exchange,exchange,exchange,exchange,exchange,exchange,",
            "session_complete"
        )
    );
}

#[test]
fn lab_human_report_is_exactly_the_published_block() {
    // The human report is six lines with DIFFERENT label styles (`key=value` on
    // the first, bare words on the rest), so the JSON assertions do not cover it.
    let (ok, code, block) = run(&[
        "lab",
        "--scenario",
        "reliable-udp",
        "--rounds",
        "8",
        "--drop-every",
        "0",
        "--settle-ms",
        "1500",
    ]);
    assert!(ok, "code={code}");
    assert_eq!(
        block,
        concat!(
            "scenario=reliable-udp ok=true rounds=8 settled=true partial=false\n",
            "session delivered=8 duplicates=0 conflicts=0 application_bytes=40\n",
            "initial admitted=8 wire_sent=8 suppressed=0 cwnd_refusals=0\n",
            "acks emitted=8 wire_sent=8 suppressed=0 failed=0 applied=8 rejected=0\n",
            "recovery pto_due=0 pto_fired=0 retransmit_attempts=0 retransmit_wire_sent=0 ",
            "retransmit_refused=0 newly_lost=0 remaining_in_flight=0\n",
            "manager_switch=false manager_active_path=udp ",
            "manager_switch_scope=manager-decision-only-no-tcp-socket cleanup=verified"
        )
    );

    // The legacy failover demo's human timeline is a different branch again.
    let (ok, code, block) = run(&["lab"]);
    assert!(ok, "code={code}");
    assert_eq!(
        block,
        concat!(
            "step=0 carrier=udp event=active bytes=0\n",
            "step=1 carrier=udp event=pto bytes=1\n",
            "step=2 carrier=udp event=uncertain bytes=8192\n",
            "step=3 carrier=tcp event=validated bytes=8192\n",
            "step=4 carrier=tcp event=migrated bytes=8192\n",
            "step=5 carrier=tcp event=duplicate_dedup bytes=1024\n",
            "step=6 carrier=tcp event=recovered bytes=9216"
        )
    );
}

#[test]
fn matrix_probe_human_report_distinguishes_pass_from_fail() {
    // `probe --matrix` without `--json` prints one of two text lines. Neither had
    // any assertion, and they are the human-facing counterpart of the artifact
    // whose structure was pinned earlier.
    let (ok, code, line) = run(&[
        "probe",
        "--matrix",
        "--target",
        "127.0.0.1:9",
        "--transport",
        "tcp",
        "--ip-version",
        "ipv4",
        "--timeout-ms",
        "100",
    ]);
    assert!(!ok && code == 1, "closed port must fail: code={code}");
    assert_eq!(line, "fail: 喵呜呜呜呜…");

    // A listening socket makes the same probe pass.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local listener");
    let target = listener.local_addr().unwrap().to_string();
    let (ok, code, line) = run(&[
        "probe",
        "--matrix",
        "--target",
        &target,
        "--transport",
        "tcp",
        "--ip-version",
        "ipv4",
        "--timeout-ms",
        "1000",
    ]);
    assert!(ok, "a listening target must pass: code={code}");
    assert_eq!(line, "pass: 喵~！");
}
