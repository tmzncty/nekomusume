//! Bounded CLI-level pinning for `health-observe`: the flag -> field wiring and
//! the human output line. The JSON FORMAT is already pinned at the
//! `CarrierHealthEvidence::json()` level in neko-carrier (see the ecaab6d slice),
//! but nothing checked that this command wires its flags into that structure, nor
//! what its human-readable line says - the only CLI assertion anywhere was
//! `contains("\"samples\"")`.
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

fn run(args: &[&str]) -> (bool, i32, String, String) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("the neko-cli binary must run");
    (
        out.status.success(),
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn each_flag_reaches_its_own_sample_field() {
    // The four values are pairwise distinct (path 7, rtt 1200, loss 5, pto 1)
    // and `--count 3` differs from all of them, so a swapped flag, a swapped
    // sample field source, a `PathId` wired to a constant, or a loop that runs
    // once instead of `count` times is all visible in the exact line.
    let (ok, code, stdout, stderr) = run(&[
        "health-observe",
        "--path",
        "7",
        "--rtt-us",
        "1200",
        "--loss-per-mille",
        "5",
        "--pto",
        "1",
        "--count",
        "3",
        "--json",
    ]);
    assert!(ok, "code={code} stderr={stderr}");
    let line = stdout.trim_end_matches('\n');
    let sample =
        "{\"path\":7,\"rtt_us\":1200,\"loss_per_mille\":5,\"pto\":1,\"state\":\"healthy\"}";
    assert_eq!(
        line,
        format!("{{\"samples\":[{sample},{sample},{sample}],\"events\":[],\"transitions\":[]}}"),
        "health-observe must wire every flag to its own field"
    );
}

#[test]
fn the_human_line_reports_the_sample_count_and_the_path() {
    // `samples=3` vs `path=9`: the two values differ, so swapping the two
    // placeholders is visible, and `samples` must be the observed count rather
    // than a constant.
    let (ok, code, stdout, stderr) = run(&[
        "health-observe",
        "--path",
        "9",
        "--rtt-us",
        "1200",
        "--loss-per-mille",
        "5",
        "--pto",
        "1",
        "--count",
        "3",
    ]);
    assert!(ok, "code={code} stderr={stderr}");
    assert_eq!(
        stdout.trim_end_matches('\n'),
        "health_observe_ok samples=3 path=9",
        "human health-observe line"
    );
    // The default count is exactly one sample.
    let (ok, _, stdout, _) = run(&[
        "health-observe",
        "--path",
        "9",
        "--rtt-us",
        "1200",
        "--loss-per-mille",
        "5",
        "--pto",
        "1",
    ]);
    assert!(ok);
    assert_eq!(
        stdout.trim_end_matches('\n'),
        "health_observe_ok samples=1 path=9"
    );
}

#[test]
fn the_flag_boundaries_are_inclusive_and_fail_closed() {
    // `loss_per_mille` window: 1000 is legal, 1001 is not (each flag appears
    // exactly once, so the parser cannot silently read an earlier occurrence).
    let (ok, _, _, _) = run(&[
        "health-observe",
        "--path",
        "1",
        "--rtt-us",
        "40",
        "--loss-per-mille",
        "1000",
        "--pto",
        "0",
    ]);
    assert!(ok, "loss-per-mille 1000 must be accepted");
    let (ok, code, _, stderr) = run(&[
        "health-observe",
        "--path",
        "1",
        "--rtt-us",
        "40",
        "--loss-per-mille",
        "1001",
        "--pto",
        "0",
    ]);
    assert!(!ok && code == 2, "1001 must be rejected: {stderr}");
    assert!(stderr.contains("loss-per-mille outside 0-1000"), "{stderr}");
    // `--count` window: 1 and 64 are legal, 0 and 65 are not.
    for good in ["1", "64"] {
        let (ok, _, stdout, stderr) = run(&[
            "health-observe",
            "--path",
            "1",
            "--rtt-us",
            "40",
            "--loss-per-mille",
            "0",
            "--pto",
            "0",
            "--count",
            good,
            "--json",
        ]);
        assert!(ok, "count {good} must be accepted: {stderr}");
        // The evidence ring must hold every requested sample, so the count is
        // observable rather than silently truncated by the sample budget.
        let observed = stdout.matches("\"state\":\"healthy\"").count();
        assert_eq!(
            observed,
            good.parse::<usize>().unwrap(),
            "count {good} must produce that many samples: {stdout}"
        );
    }
    for bad in ["0", "65"] {
        let (ok, code, _, stderr) = run(&[
            "health-observe",
            "--path",
            "1",
            "--rtt-us",
            "40",
            "--loss-per-mille",
            "0",
            "--pto",
            "0",
            "--count",
            bad,
        ]);
        assert!(!ok && code == 2, "count {bad} must be rejected: {stderr}");
        assert!(stderr.contains("count outside 1-64"), "{stderr}");
    }
    // Each required flag is genuinely required, and each is named in its error.
    for (flag, message) in [
        ("--path", "missing --path"),
        ("--rtt-us", "missing --rtt-us"),
        ("--loss-per-mille", "missing --loss-per-mille"),
        ("--pto", "missing --pto"),
    ] {
        let args: Vec<&str> = [
            "health-observe",
            "--path",
            "1",
            "--rtt-us",
            "40",
            "--loss-per-mille",
            "0",
            "--pto",
            "0",
        ]
        .iter()
        .copied()
        .filter(|a| *a != flag)
        .collect();
        let (ok, code, _, stderr) = run(&args);
        assert!(!ok && code == 2, "omitting {flag} must fail: {stderr}");
        assert!(stderr.contains(message), "omitting {flag}: {stderr}");
    }
}
