//! Bounded CLI-level pinning for three fixture artifacts whose fields were
//! otherwise unasserted: `scheduler-fairness --json`, `workload --json` and
//! `key-update --json`. The large `probe.rs` suite only checks `"ok":true` and
//! the `fixture` name for each of these, so every other field was free. This
//! file exercises only these three commands, so it stays fast and independent.
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

fn run_json(args: &[&str]) -> String {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("the neko-cli binary must run");
    assert!(
        out.status.success(),
        "args={args:?} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .trim_end_matches('\n')
        .to_owned()
}

#[test]
fn scheduler_fairness_artifact_is_exactly_the_published_line() {
    // Deterministic across repeated runs, so the whole line is pinned. The
    // sibling counters are given different values (9 vs 3) so a swap between
    // `interactive_frames` and `bulk_frames` cannot hide.
    let s = run_json(&[
        "scheduler-fairness",
        "--rounds",
        "3",
        "--bytes",
        "8",
        "--json",
    ]);
    assert_eq!(
        s,
        concat!(
            "{\"ok\":true,",
            "\"fixture\":\"scheduler-fairness\",",
            "\"transport\":\"loopback\",",
            "\"rounds\":3,",
            "\"interactive_frames\":9,",
            "\"bulk_frames\":3,",
            "\"max_interactive_burst\":3,",
            "\"forced_bulk_services\":3,",
            "\"bound_interactive_burst\":3}"
        ),
        "scheduler-fairness artifact is a published contract"
    );
    // `rounds` is independently pinned by a second invocation, so a mutation
    // that wires the field to a different (also-3) counter is still caught.
    let other = run_json(&[
        "scheduler-fairness",
        "--rounds",
        "5",
        "--bytes",
        "8",
        "--json",
    ]);
    assert!(
        other.contains("\"rounds\":5"),
        "rounds echoes --rounds: {other}"
    );
    assert!(
        other.contains("\"bound_interactive_burst\":3"),
        "the fairness bound is a fixed documented constant: {other}"
    );
}

#[test]
fn workload_artifact_is_exactly_the_published_line() {
    // `--duration 1` bounds the run at about a second. The values are chosen so
    // that no two fields share a number (1 / 2 / 6 / 48), which is what makes a
    // swapped positional argument visible rather than masked.
    let s = run_json(&[
        "workload",
        "--duration",
        "1",
        "--concurrency",
        "2",
        "--records",
        "3",
        "--bytes",
        "8",
        "--json",
    ]);
    assert_eq!(
        s,
        concat!(
            "{\"ok\":true,",
            "\"fixture\":\"session-workload\",",
            "\"duration_seconds\":1,",
            "\"concurrency\":2,",
            "\"records\":6,",
            "\"application_bytes\":48,",
            "\"cleanup\":\"verified\"}"
        ),
        "workload artifact is a published contract"
    );
}

#[test]
fn key_update_artifact_is_exactly_the_published_line() {
    // The `events` array is pinned in full, which fixes both the key name and
    // the order and identity of the emitted event names.
    let s = run_json(&["key-update", "--json"]);
    assert_eq!(
        s,
        concat!(
            "{\"ok\":true,",
            "\"fixture\":\"secure-session-key-update\",",
            "\"exchanges\":12,",
            "\"key_phase\":1,",
            "\"events\":[",
            "\"session_opened\",\"key_phase_0\",",
            "\"exchange\",\"exchange\",\"exchange\",\"exchange\",\"exchange\",\"exchange\",",
            "\"key_update_committed\",\"old_phase_rejected\",",
            "\"exchange\",\"exchange\",\"exchange\",\"exchange\",\"exchange\",\"exchange\",",
            "\"session_complete\"]}"
        ),
        "key-update artifact is a published contract"
    );
}
