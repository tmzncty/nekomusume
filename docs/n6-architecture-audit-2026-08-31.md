# N6 architecture audit: first-RC target boundary

**Audited commit:** `91a735c0252e7df5da611da4a70e71a60dbdd44d`
**Audit date:** 2026-08-31
**Decision:** the first release candidate, if and when the independent release gates permit one, targets **`x86_64-unknown-linux-gnu` only**. This audit does not declare an RC and does not change the blocked production/WAN gates.

## Evidence boundary

The repository package script allowlists both `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`, and `docs/release-engineering.md` correctly describes aarch64 as an archive-layout declaration requiring cross-build and execution evidence. The only checked-in build record, `docs/release/first-build-evidence.json`, is x86_64. No checked-in aarch64 artifact, build record, QEMU run, or native ARM run was found.

The authoritative VPS is x86_64 Linux (`uname -m`: `x86_64`) with Rust/Cargo 1.98.0 and host target `x86_64-unknown-linux-gnu`.

## x86_64 native build and run: PASS

At the audited commit, two cleanly staged package invocations used the same commit timestamp as `SOURCE_DATE_EPOCH`. Both produced the same archive SHA-256:

```text
9677bf075c008aa164fa585d3a13feb234ad1dddac4a261ce86c473567a07027
```

`smoke-package.sh` passed archive path, mode, embedded checksum, and native execution checks. The extracted binary was an x86-64 ELF and `neko-cli capabilities --json` ran successfully, reporting schema `nekomusume.capabilities.v1`, `target_os=linux`, `target_arch=x86_64`, and `secret_free=true`.

This is native x86_64 packaging/runtime evidence, not a security or production-readiness approval.

## aarch64 cross-build: declared, not established

The repository accepts `TARGET=aarch64-unknown-linux-gnu`, but the audit host had only the x86_64 Rust target installed and had no `aarch64-linux-gnu-gcc`, `cross`, or Zig toolchain. A direct package cross-build was attempted and failed as expected with missing aarch64 `core`/`std` plus missing `aarch64-linux-gnu-gcc` (exit 101).

Therefore aarch64 is a **declared candidate package target only**. It is not a build-supported RC target at this commit.

## ARM execution: unavailable

Neither `qemu-aarch64` nor `qemu-aarch64-static` was available on the VPS. Docker was available with an amd64 server, but no repository-provided, pinned ARM cross-build/QEMU toolchain or checked-in ARM execution procedure was found. No QEMU execution was claimed.

No native ARM machine was available to this audit. No native ARM execution was claimed or inferred.

## Promotion condition

Adding aarch64 to a later RC target set requires, at minimum:

1. a reproducible or pinned aarch64 cross/native build toolchain and successful locked package build;
2. package-integrity smoke plus actual aarch64 binary execution under documented QEMU or equivalent emulation where available;
3. native ARM package/runtime execution on a recorded host before claiming native ARM support;
4. checked-in machine-readable build evidence tied to the exact candidate commit; and
5. the same full repository gates and independent release/security review required for x86_64.

Until those exist, release metadata and publication must not present aarch64 as an RC-supported architecture.
