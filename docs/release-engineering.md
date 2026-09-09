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

Before querying build metadata or creating output, the script rejects staged changes, unstaged tracked changes, and non-ignored untracked files, so emitted `git_commit` provenance names source state equal to HEAD. Ignored generated paths such as `target/` and repository-root `dist/` do not make a clean checkout unbuildable. The script then uses `Cargo.lock`, disables incremental compilation, normalizes archive order/owner/group/mtime, suppresses gzip timestamps, and emits `*.build.json`. Equal archive hashes are evidence for the same source, toolchain, target and environment—not a promise across different compiler/linker versions.

## Package smoke and permissions

```sh
scripts/release/smoke-package.sh dist-a/nekomusume-*.tar.gz
```

The smoke validates the complete expected archive shape before extraction, rejects absolute/traversal paths and non-regular members (including links and special files), extracts without archive ownership or permission inheritance, verifies `SHA256SUMS`, requires mode `0755` for the executable and `0644` for documentation, then runs secret-free `capabilities --json` when target equals host. Synthetic adversarial archive regressions are wired into `scripts/check.sh`. The CLI already creates identity files as `0600` on Unix; operators must never package, copy, log, or overwrite an existing identity.

Current deterministic CLI tests cover externally emitted signal-driven `READY -> DRAINING -> STOPPED` lifecycle and same-address rebind after shutdown for bounded TCP and UDP servers at exact `7cebe6b`. The prior exact-`14be1c8` installed-package attempt remains an incomplete orchestration negative; the single changed-hypothesis exact-`7cebe6b` VPS invocation is separately anchored by its sanitized result and provenance erratum.

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

## Current-tree revalidation

Exact `f19ad28` was rebuilt twice with identical archive identity and exercised through isolated authenticated TCP/UDP package smokes plus a same-tree immutable-directory symlink switch/rollback. See [`package-operator-current-tree-f19ad28-20260909.md`](package-operator-current-tree-f19ad28-20260909.md). This supplements rather than replaces the historical distinct-version N5 A→B→A evidence.

Exact `a9928c8` was then installed in a dedicated temporary path on the self-owned VPS and exercised once over authenticated TCP and once over authenticated UDP, with installed-binary hash verification and verified remote cleanup. See [`package-operator-vps-a9928c8-20260909.md`](package-operator-vps-a9928c8-20260909.md). This remains bounded self-owned VPS evidence, not public reachability, release, or production evidence.

A separate bounded rehearsal used genuinely different exact-tree packages in the sequence `91a735c -> dc90c5f -> 91a735c`, retained external temporary identity/state for the bounded probe contract, returned the selected release to A, and verified cleanup. See [`package-operator-distinct-version-91a735c-dc90c5f-20260909.md`](package-operator-distinct-version-91a735c-dc90c5f-20260909.md). It does not establish arbitrary state-schema compatibility or production upgrade policy.

## Honest remaining K

- native aarch64 cross-build and execution evidence;
- pinned compiler/linker/container provenance and cross-environment reproducibility;
- signed checksums, signing-key custody, SBOM/provenance, and publication workflow;
- installed-package VPS SIGTERM, listener release, and same-port restart/rebind evidence is recorded for exact `7cebe6b`: the changed-hypothesis run completed the bounded TCP and UDP lifecycle with ordered `READY -> DRAINING -> STOPPED`, same-port authenticated restart exchanges, and verified cleanup; this remains operator evidence, not daemon/service-manager, release, or production approval;
- service-manager sandboxing, least-privilege runtime profile, upgrade/state compatibility tests;
- independent security/release review and all blocked M5 WAN/production gates.
