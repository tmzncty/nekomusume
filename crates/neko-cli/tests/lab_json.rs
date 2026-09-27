//! Bounded CLI-level pinning for the `lab --scenario reliable-udp --json`
//! artifact. The other lab assertions live in the large `probe.rs` suite, which
//! is slow; this file exercises only this one command so the artifact's SCHEMA
//! and its full field set are pinned directly, fast, and independently.
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");

fn lab_json(args: &[&str]) -> (bool, String) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("the neko-cli binary must run");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout)
            .trim_end_matches('\n')
            .to_owned(),
    )
}

#[test]
fn the_clean_scenario_artifact_is_exactly_the_published_line() {
    // Every existing assertion on this artifact is a `contains` on one field
    // fragment, so the schema string, the section key names, the field key
    // names and the separators between them were all unwitnessed - a renamed
    // key or a dropped comma would leave every fragment present and every test
    // passing. The clean scenario is deterministic (verified across repeated
    // runs), so the whole line is pinned byte for byte.
    let (ok, s) = lab_json(&[
        "lab",
        "--scenario",
        "reliable-udp",
        "--rounds",
        "8",
        "--drop-every",
        "0",
        "--settle-ms",
        "1500",
        "--json",
    ]);
    assert!(ok, "{s}");
    assert_eq!(
        s,
        concat!(
            "{\"ok\":true,",
            "\"schema\":\"nekomusume.reliable-udp-lab.v1\",",
            "\"scenario\":\"reliable-udp\",",
            "\"mode\":\"loopback-controlled\",",
            "\"rounds\":8,",
            "\"drop_every\":0,",
            "\"drop_ack_every\":0,",
            "\"offered\":8,",
            "\"settled\":true,",
            "\"partial\":false,",
            "\"session\":{\"delivered\":8,\"duplicates\":0,\"conflicts\":0,\"application_bytes\":40},",
            "\"initial\":{\"admitted\":8,\"wire_sent\":8,\"suppressed\":0,\"cwnd_refusals\":0},",
            "\"acks\":{\"emitted\":8,\"wire_sent\":8,\"suppressed\":0,\"failed\":0,\"applied\":8,\"rejected\":0},",
            "\"recovery\":{\"pto_due\":0,\"pto_fired\":0,\"retransmit_attempts\":0,",
            "\"retransmit_wire_sent\":0,\"retransmit_refused\":0,\"newly_lost\":0,",
            "\"remaining_in_flight\":0},",
            "\"unauthenticated\":0,",
            "\"recv_rejected\":0,",
            "\"manager_switch\":false,",
            "\"manager_active_path\":\"udp\",",
            "\"manager_switch_scope\":\"manager decision only; this command opens no TCP socket\",",
            "\"cleanup\":\"verified\"}"
        ),
        "the lab artifact is a published contract"
    );
}

#[test]
fn the_legacy_lab_failover_demo_timeline_is_exactly_the_published_line() {
    // `lab` without `--scenario` emits a different artifact from the
    // reliable-udp one: a failover demo. The only assertion on it anywhere is
    // `contains("\"demo\":\"failover\"")`, so the WHOLE `timeline` array - its key,
    // every entry's `step`/`carrier`/`event`/`bytes` keys and values, the
    // separators between entries and the closing brackets - was unasserted. It
    // is deterministic, so the whole line is pinned.
    let out = Command::new(BIN)
        .args(["lab", "--json"])
        .output()
        .expect("the neko-cli binary must run");
    assert!(out.status.success());
    let line = String::from_utf8_lossy(&out.stdout)
        .trim_end_matches('\n')
        .to_owned();
    assert_eq!(
        line,
        concat!(
            "{\"ok\":true,\"demo\":\"failover\",\"timeline\":[",
            "{\"step\":0,\"carrier\":\"udp\",\"event\":\"active\",\"bytes\":0},",
            "{\"step\":1,\"carrier\":\"udp\",\"event\":\"pto\",\"bytes\":1},",
            "{\"step\":2,\"carrier\":\"udp\",\"event\":\"uncertain\",\"bytes\":8192},",
            "{\"step\":3,\"carrier\":\"tcp\",\"event\":\"validated\",\"bytes\":8192},",
            "{\"step\":4,\"carrier\":\"tcp\",\"event\":\"migrated\",\"bytes\":8192},",
            "{\"step\":5,\"carrier\":\"tcp\",\"event\":\"duplicate_dedup\",\"bytes\":1024},",
            "{\"step\":6,\"carrier\":\"tcp\",\"event\":\"recovered\",\"bytes\":9216}",
            "]}"
        ),
        "the legacy lab failover demo is a published contract"
    );
}

#[test]
fn the_lossy_scenario_artifact_is_exactly_the_published_line() {
    // The lossy scenario exercises the fields the clean one leaves at zero
    // (suppressed, pto_*, retransmit_*, newly_lost), so it is where those key
    // names and their wiring are pinned. It is deterministic too.
    let (ok, s) = lab_json(&[
        "lab",
        "--scenario",
        "reliable-udp",
        "--rounds",
        "8",
        "--drop-every",
        "4",
        "--settle-ms",
        "4000",
        "--json",
    ]);
    assert!(ok, "{s}");
    assert_eq!(
        s,
        concat!(
            "{\"ok\":true,",
            "\"schema\":\"nekomusume.reliable-udp-lab.v1\",",
            "\"scenario\":\"reliable-udp\",",
            "\"mode\":\"loopback-controlled\",",
            "\"rounds\":8,",
            "\"drop_every\":4,",
            "\"drop_ack_every\":0,",
            "\"offered\":8,",
            "\"settled\":true,",
            "\"partial\":false,",
            "\"session\":{\"delivered\":8,\"duplicates\":0,\"conflicts\":0,\"application_bytes\":40},",
            "\"initial\":{\"admitted\":8,\"wire_sent\":6,\"suppressed\":2,\"cwnd_refusals\":0},",
            "\"acks\":{\"emitted\":8,\"wire_sent\":8,\"suppressed\":0,\"failed\":0,\"applied\":8,\"rejected\":0},",
            "\"recovery\":{\"pto_due\":2,\"pto_fired\":2,\"retransmit_attempts\":2,",
            "\"retransmit_wire_sent\":2,\"retransmit_refused\":0,\"newly_lost\":2,",
            "\"remaining_in_flight\":0},",
            "\"unauthenticated\":0,",
            "\"recv_rejected\":0,",
            "\"manager_switch\":false,",
            "\"manager_active_path\":\"udp\",",
            "\"manager_switch_scope\":\"manager decision only; this command opens no TCP socket\",",
            "\"cleanup\":\"verified\"}"
        ),
        "the lossy lab artifact is the same contract with the impairment fields set"
    );
}

#[test]
fn the_documented_flags_are_reflected_in_the_artifact() {
    // `rounds`/`offered`/`drop_every` echo the flags, and `offered` is the
    // admission count rather than a copy of an unrelated field.
    let (ok, s) = lab_json(&[
        "lab",
        "--scenario",
        "reliable-udp",
        "--rounds",
        "6",
        "--drop-every",
        "3",
        "--settle-ms",
        "3000",
        "--json",
    ]);
    assert!(ok, "{s}");
    assert!(s.contains("\"rounds\":6"), "{s}");
    assert!(s.contains("\"offered\":6"), "{s}");
    assert!(s.contains("\"drop_every\":3"), "{s}");
    assert!(s.contains("\"drop_ack_every\":0"), "{s}");
}
