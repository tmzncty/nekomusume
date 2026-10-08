//! D068 CLI-side assertions: bulk is measurement-gated, self-marking, and
//! bounded by the sealed-frame record ceiling; the exception path is
//! unreachable without NEKO_MEASUREMENT=1 (ADR assertion 5).
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_neko-cli");
const KEY: &str = "ababababababababababababababababababababababababababababababababab";

#[test]
fn bulk_requires_measurement_env() {
    let out = Command::new(BIN)
        .args([
            "bulk-client",
            "--bytes-total",
            "2340",
            "--record-bytes",
            "1170",
            "--addr",
            "127.0.0.1:40080",
            "--server-key",
            KEY,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("NEKO_MEASUREMENT=1"),
        "ungated bulk is a D068 rollback trigger"
    );
    // server side equally gated
    let out = Command::new(BIN)
        .args(["bulk-server", "--port", "40080", "--client-key", KEY])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn bulk_measurement_markers_and_validation() {
    // gated command with env: marker + exception self-label lines print
    // before any connection attempt; bytes-total > 2 GiB rejects by name
    let out = Command::new(BIN)
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--bytes-total",
            "2147483649", // 2 GiB + 1
            "--record-bytes",
            "1170",
            "--addr",
            "127.0.0.1:40080",
            "--server-key",
            KEY,
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("measurement_mode=true relaxed_limits"),
        "marker line missing: {stdout}"
    );
    assert!(
        stdout.contains("measurement_limits_exceptions=total_bytes,queue_records,queue_bytes"),
        "exception self-label missing: {stdout}"
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr.contains("2147483648"),
        "ceiling must be named: {stderr}"
    );
    // record-bytes keeps the #3 sealed-frame ceiling (1170), +1 rejects
    let out = Command::new(BIN)
        .env("NEKO_MEASUREMENT", "1")
        .args([
            "bulk-client",
            "--bytes-total",
            "2342",
            "--record-bytes",
            "1171",
            "--addr",
            "127.0.0.1:40080",
            "--server-key",
            KEY,
        ])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("1-1170"),
        "record ceiling must be the sealed-frame bound"
    );
}

/// D068 assertion 6 (CLI half): compile-time pin — the CLI exception
/// constant is derived from the library's exported ceilings and sits
/// strictly above the default HARD ceiling.
#[test]
fn cli_measurement_ceiling_pins() {
    const _: () = {
        // CLI must not be able to drift from the library's exported set
        assert!(
            neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES
                > neko_session::HARD_MAX_RUNTIME_TOTAL_BYTES
        );
        assert!(neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES == 2 << 30);
        assert!(neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_RECORDS == 1 << 22);
        assert!(neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_BYTES == 2 << 30);
    };
}
