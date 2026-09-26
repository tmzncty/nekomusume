//! Bounded CLI-level pinning for the benchmark harness. The artifact contract
//! (schema, units, per-benchmark keys, iteration bounds) is only observable by
//! running the binary, so these assertions drive the real executable.
use std::process::Command;

const BENCHMARKS: [&str; 7] = [
    "encode_decode",
    "outer_record_encode_decode",
    "crypto_seal",
    "crypto_open",
    "scheduler_next_frame",
    "recovery_ack_loss",
    "instrumentation_counter",
];

fn run(iters: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_neko-bench"))
        .env("NEKO_BENCH_ITERS", iters)
        .output()
        .expect("the benchmark binary must run");
    assert!(out.status.success(), "bench exit: {:?}", out.status);
    String::from_utf8(out.stdout).expect("stdout is utf-8")
}

/// Every numeric assertion below anchors on the FOLLOWING delimiter, because a
/// bare `"iterations":1000` also matches the `10000` prefix.
fn banner(stdout: &str) -> &str {
    stdout.lines().next().expect("a banner line")
}

#[test]
fn the_default_run_emits_the_documented_artifact() {
    let stdout = run("3");
    let head = banner(&stdout);
    // The artifact is bounded local evidence: it must say so, name its schema
    // version and its unit, and never claim superiority.
    assert!(
        head.contains("\"schema\":\"era4-i-performance.v1\""),
        "{head}"
    );
    assert!(
        head.contains("\"unit\":\"nanoseconds per operation\""),
        "{head}"
    );
    assert!(
        head.contains("\"claims\":\"bounded local evidence only; no superiority claim\""),
        "{head}"
    );
    assert!(head.contains("\"iterations\":3,"), "{head}");

    let lines: Vec<&str> = stdout.lines().collect();
    let names: Vec<&str> = lines
        .iter()
        .filter(|l| l.contains("\"benchmark\":\""))
        .map(|l| {
            l.split("\"benchmark\":\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
        })
        .collect();
    assert_eq!(names, BENCHMARKS, "benchmark set and order");

    for line in &lines {
        if !line.contains("\"benchmark\":\"") {
            continue;
        }
        // Every row reports the requested iteration count, counts no failures,
        // and lists median before p95 (the field order is part of the artifact).
        assert!(line.contains("\"iterations\":3,"), "{line}");
        assert!(line.contains("\"failures\":0}"), "{line}");
        let median_at = line.find("\"median_ns\":").expect("median field");
        let p95_at = line.find("\"p95_ns\":").expect("p95 field");
        assert!(median_at < p95_at, "median precedes p95: {line}");
        let median: u128 = line
            .split("\"median_ns\":")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let p95: u128 = line
            .split("\"p95_ns\":")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        // p95 is a higher order statistic of the same sorted sample, so it can
        // never fall below the median.
        assert!(p95 >= median, "p95 {p95} >= median {median}: {line}");
    }

    assert!(
        stdout.contains(
            "CPU/memory process-wide accounting is intentionally not reported: this slice has no stable cross-platform sampler."
        ),
        "the resource note must state why accounting is absent"
    );
}

#[test]
fn iteration_bounds_are_clamped_and_a_bad_value_falls_back_to_the_default() {
    // Below the floor the run is clamped up to one iteration.
    assert!(banner(&run("0")).contains("\"iterations\":1,"));
    assert!(banner(&run("1")).contains("\"iterations\":1,"));
    // An unparsable value falls back to the documented default.
    assert!(banner(&run("bogus")).contains("\"iterations\":1000,"));
    assert!(banner(&run("")).contains("\"iterations\":1000,"));
    // Above the ceiling the run is clamped down to the documented maximum.
    assert!(banner(&run("99999")).contains("\"iterations\":10000,"));
}
