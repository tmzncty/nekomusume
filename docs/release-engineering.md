# Era-4 K: bounded packaging and release engineering

**Status:** first packaging slice only. This is not RC, security approval, public-listener approval, or the production gate. The release gate in `docs/spec/m5-release-readiness-gate.md` remains blocked.

## Supported package targets

The package contract declares Linux GNU targets `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. This slice builds and executes the package smoke only on the native VPS target recorded in `docs/release/first-build-evidence.json`. The aarch64 declaration means the archive layout is supported; cross-build and execution evidence is still required before distributing an aarch64 artifact. Other targets fail closed.

Requirements are Rust/Cargo from the locked workspace plus GNU `tar`, `gzip`, and `sha256sum`. Build from a clean, exact commit:

```sh
SOURCE_DATE_EPOCH=$(git show -s --format=%ct HEAD) OUT="$PWD/dist-a" scripts/release/build-package.sh
SOURCE_DATE_EPOCH=$(git show -s --format=%ct HEAD) OUT="$PWD/dist-b" scripts/release/build-package.sh
sha256sum dist-a/*.tar.gz dist-b/*.tar.gz
```

The script uses `Cargo.lock`, disables incremental compilation, normalizes archive order/owner/group/mtime, suppresses gzip timestamps, and emits `*.build.json`. Equal archive hashes are evidence for the same source, toolchain, target and environment—not a promise across different compiler/linker versions.

## Package smoke and permissions

```sh
scripts/release/smoke-package.sh dist-a/nekomusume-*.tar.gz
```

The smoke rejects absolute/traversal paths, extracts without archive ownership or permission inheritance, verifies `SHA256SUMS`, requires mode `0755` for the executable and `0644` for documentation, then runs secret-free `capabilities --json` when target equals host. The CLI already creates identity files as `0600` on Unix; operators must never package, copy, log, or overwrite an existing identity.

Graceful SIGTERM and stale-listener restart are **not claimed by this slice**: current bounded server commands do not expose an explicit signal-readiness or daemon lifecycle contract, and forcing a kill-based test would not prove graceful shutdown. They remain K work.

## Install, upgrade, rollback

Use a dedicated unprivileged service account and a root-owned installation directory. Keep identity/state outside the archive (for example `/var/lib/nekomusume`, directory `0700`, identity `0600`). Never place identities below `/opt/nekomusume/releases`.

```sh
# after package smoke; VERSION names the extracted archive directory
sudo install -d -o root -g root -m 0755 /opt/nekomusume/releases
sudo cp -a --no-preserve=ownership "$VERSION" /opt/nekomusume/releases/
sudo chown -R root:root /opt/nekomusume/releases/"$VERSION"
sudo find /opt/nekomusume/releases/"$VERSION" -type d -exec chmod 0755 {} +
sudo find /opt/nekomusume/releases/"$VERSION" -type f -exec chmod 0644 {} +
sudo chmod 0755 /opt/nekomusume/releases/"$VERSION"/bin/neko-cli
sudo ln -sfn /opt/nekomusume/releases/"$VERSION" /opt/nekomusume/current
```

For upgrade, fully smoke the new archive, install it as a new immutable version directory, stop the bounded process if one was explicitly started, save `readlink /opt/nekomusume/current`, atomically switch the symlink, and run `capabilities --json`. Do not migrate or regenerate identity files during package upgrade.

For rollback, stop the explicitly managed process, atomically point `current` to the saved prior release directory, and rerun capabilities/package smoke. If state format ever changes, a separately reviewed backward-compatibility and backup procedure is required; this slice defines no state migration. Remove an old release only after rollback observation, and never recursively remove the external state/identity path.

## Honest remaining K

- native aarch64 cross-build and execution evidence;
- pinned compiler/linker/container provenance and cross-environment reproducibility;
- signed checksums, signing-key custody, SBOM/provenance, and publication workflow;
- explicit graceful shutdown/readiness semantics and then SIGTERM/stale-listener tests;
- service-manager sandboxing, least-privilege runtime profile, upgrade/state compatibility tests;
- independent security/release review and all blocked M5 WAN/production gates.
