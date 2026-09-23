# Independent bounded review — CLI cross-platform/process evidence at `416fd5e`

## Classification

**NO repository-local correctness finding within the currently evidenced Linux release/package scope; cross-platform runtime evidence remains unproven and must not be inferred.**

Reachable review anchor: `416fd5e60f0b90155acf65ad33927d7accaecaf6` (`main` at review start). This reconciles the older `docs/reviews/independent-cli-portability-8e11de0-20260913.md` with the materially changed current CLI/process test surface after H-I4-097..115.

## Exact-current owners inspected

- `crates/neko-cli/Cargo.toml`
- `crates/neko-cli/src/main.rs`
- `crates/neko-cli/src/multistream.rs`
- `crates/neko-cli/tests/probe.rs`
- `crates/neko-cli/tests/multistream.rs`
- `.github/workflows/ci.yml`
- `scripts/check.sh`
- `README.md`
- `docs/status.md`
- `docs/package-operator-current-tree-f19ad28-20260909.md`
- `docs/release-security-review-packet.md`
- `docs/specs/nekomusume-session-v0.md`
- `docs/reviews/independent-cli-portability-8e11de0-20260913.md`
- `docs/reviews/independent-cli-process-ownership-fcd7a08-20260924.md`

The invariant challenged was not “Windows/macOS are supported”; the current repository does not establish that policy. The challenge was whether current item-4/release evidence accidentally promotes Linux/Unix source or execution behavior into a cross-platform claim, and whether the H-I4 process-hardening helpers contain an obvious OS-specific correctness dependency inside the evidenced Linux test path.

## Findings

### 1. Historical portability note is not current-tree portability evidence

The 2026-09-13 `8e11de0` review remains valid only for its exact historical tree. It must not be reused as a statement that the present CLI has no platform-specific dependencies or paths.

Current `neko-cli` directly depends on `signal-hook` and `libc`. Current `main.rs` registers `SIGTERM` and `SIGINT` handlers through `signal_hook`, and the payload-file benchmark JSON path inventories descriptors by reading `/proc/self/fd`. The H-I4-115 deterministic descendant-held-pipe regression is intentionally Unix-only and invokes `sh` under `#[cfg(unix)]`.

These are sufficient to reject a generic “current CLI is cross-platform because the Rust APIs look portable” inference. They are **not**, by themselves, a release blocker against the repository's currently evidenced target.

### 2. Current authoritative package/CI evidence is Linux-scoped

The package/operator evidence explicitly records target `x86_64-unknown-linux-gnu`. README identifies Linux server + VPS as the real-WAN path. GitHub Actions currently runs both stable checks and fuzz smoke on `ubuntu-latest`; `scripts/check.sh` is a Bash entry point. The release/security packet describes Unix owner-only identity evidence and does not provide Windows/macOS native execution evidence.

Therefore the factual boundary is:

- Linux/x86_64 source, local gates, package rehearsal and selected hosted CI are evidence where their exact anchors say so.
- Source reasoning around `std::process::Child`, `try_wait`, `kill`, TCP/UDP loopback and mpsc timeouts may be portable in shape, but it is not Windows/macOS execution evidence.
- The Unix H-I4-115 inherited-writer regression proves the selected Unix descendant-pipe case only. It must not be cited as Windows process-tree/pipe proof.
- `/proc/self/fd` makes the payload-file benchmark observation explicitly Linux/procfs-dependent in the current implementation. No current release claim requires that observation on another OS.

No current authoritative file inspected claims a Windows/macOS package, native gate, service lifecycle, or benchmark result. Hence there is no exact-current claim contradiction requiring a code repair in this lane.

### 3. H-I4 process-hardening remains valid for the evidenced Linux path

The exact-current `probe.rs`/`multistream.rs` bounded child helpers use standard child ownership plus concurrent stdout/stderr drain, finite direct-child wait, checked terminate/reap classification, and finite reader completion. The immediately preceding bounded ownership review found no remaining concrete Linux test-owned process/socket/thread hang owner after H-I4-097..115.

Nothing in this cross-platform reconciliation reopens those closed Linux findings. It only narrows what may be claimed from them.

## Evidence boundary / exclusions

- No reviewer-local Rust build, test, `scripts/check.sh`, Clippy, or package build was run in this review.
- No Windows, macOS, BSD, aarch64, musl, or other non-Linux execution/compile check was run.
- No cross-compilation toolchain was installed or inferred.
- No change to production signal semantics, process-group policy, JSON schema, package target matrix, or benchmark FD-count contract was selected.
- No decoder/parser/crypto-framing implementation changed; fuzz was not run.
- No VPS/WAN/performance work was run.
- No D019, capacity/security value, signing/key-custody/SBOM/publication, core Session/Carrier/ACK/crypto/wire, RC/freeze/release/production decision was made.

If maintainers later decide that Windows/macOS native support is a release requirement, that is a new explicit target/policy lane: at minimum compile/native process tests, signal handling, file-permission/identity semantics, `/proc/self/fd` benchmark accounting, Bash/package scripts and socket lifecycle need target-specific validation. This review does not choose that release policy.

## Item-4 consequence / next lane

For item-4 factual reconciliation, retain `independent-cli-portability-8e11de0-20260913.md` as historical exact-tree evidence only. Use this note for the current boundary: **Linux-scoped evidence, no cross-platform runtime claim.**

Proceed to algorithmic/resource boundedness reconciliation. In particular, preserve the already classified `SessionRuntime.events` retained-state bound as `POLICY_BLOCKED_RESOURCE_BOUND`; do not invent a history/capacity number. Challenge implemented algorithms for accidental growth/loop work that can be proven against current semantics without adversarial-capacity benchmarking or new policy values.
