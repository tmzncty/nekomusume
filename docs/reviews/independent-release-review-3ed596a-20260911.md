# Independent release/security review — exact `3ed596a`

**Reviewed revision:** `3ed596ac62a3e3bda0fcea494432e0533285cb83` (`origin/main`, clean tree).
**Review type:** independent bounded evidence-index review. **Date:** 2026-09-11 (UTC `2026-09-10T20:12:51Z`).
**Scope:** `docs/release-security-review-packet.md` and its linked evidence, plus focused reproduction of headline claims. This is **not** a comprehensive security audit, cryptanalysis, adversarial-load assessment, penetration test, or production approval.

## Method / reproduction

Host `tmzn@192.168.122.1`, workdir `/media/tmzn/DATA5/nekomusume-work`, `rustc 1.98.0`.

- `git fetch --all --prune`; `git rev-parse HEAD origin/main` -> both `3ed596ac...`; `git status --porcelain` empty.
- Local-link resolution over the packet and `docs/reviews/release-item4-subgates-20260909.md` -> **0 dangling links**.
- `git cat-file -t` over 45 hex tokens cited in both docs -> all real commits reachable (`20260909/10/11` are date tokens, not SHAs).
- `git diff --stat 78111e8..3ed596a` -> only `docs/**.md` changed; **no crate/script/fixture change** since the last full-gated classification tree.
- `python3 scripts/validate-canonical-vectors.py` -> `42 vectors; domains=10; freeze=true`.
- `cargo test -p neko-crypto --all-targets` -> `37 passed; 0 failed`.
- `python3 scripts/check-era4-closure.py` -> `0 open-ready, 16 already-sufficient, 0 dependency-blocked`.
- `bash scripts/check-plan-sync.sh` / `check-status-evidence.sh` / `check-release-boundaries.sh` / `check-markdown-links.sh` -> all pass.
- `bash scripts/release/smoke-package-test.sh` -> `package-smoke-regressions-ok`; `scripts/release/check-clean-source-test.sh` -> `package-clean-source-regressions-ok`.
- `sha256sum docs/package-operator-distinct-version-91a735c-dc90c5f-20260909.result.jsonl` -> `4e54f3205cf295ec82e9186eb6b6271558e6babddae819c5ef34e1595a0ddb09` (5 records), matching the packet's cited anchor.
- Spot-check of `crates/neko-cli/src/main.rs` (identity create/open) against the item-4 security claims.

## Verdict

**BLOCKER: none. HIGH: none found.**

Every packet headline claim that was reproducible held on the reviewed exact tree:

- release flags `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` are consistent across the packet, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and the handoff;
- rolling Era-4 `OPEN_READY` rows = `0`, corroborated independently by `scripts/check-era4-closure.py` and by `closure.open_ready_rows: []` in `docs/era4-ledger-2026-08-30.json`;
- the code at the reviewed head is identical to the last fully gated tree `78111e8` (docs-only delta `78111e8..3ed596a`), so the "provenance-only later commit" framing is accurate;
- the corpus-freeze claim, the 37-test `neko-crypto` matrix, and the five-record anchor hash reproduce exactly;
- `crates/neko-cli/src/main.rs:324` opens identities with `O_NOFOLLOW | O_CLOEXEC`; `:331-343` consumes metadata and bytes from the same descriptor and rejects non-regular files and group/world-readable modes without chmod repair; `:361-381` creates with `create_new` + mode `0o600` before writing private bytes. This matches the item-4 claim as written.

## Findings (non-blocking)

- **[nit]** `docs/era4-ledger-2026-08-30.json` -> `closure.handoff_sha256` `6de20f52b7d7faa7c43961ba64b2e00ac8076385a83219cd5462966daff066a4` corresponds to `docs/CHATGPT_HANDOFF.md` at ancestor `3649492`, not to the handoff at the ledger's own last commit `78111e8` (`ee7d11fe...`) nor at the reviewed head (`41648844...`). The packet explicitly frames the ledger as a rolling overlay and cross-checks current readiness against `IMPLEMENTATION_PLAN.md`/`status.md`, so no release claim depends on this field; it is stale provenance binding only. Suggested minimal fix (only if desired): refresh or drop `closure.handoff_sha256`, or annotate it as classification-time provenance. Not a release blocker.

No other defect was found in the reviewed surfaces.

## Remaining boundaries / open policy or environment blockers (not defects)

- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`; release item 3 and natural-loss evidence remain unchecked; independent signing/key-custody/SBOM, sustained/public reachability, native aarch64 execution, live PMTUD, and HY2 / repeated-warm-failover same-class runs remain open or frozen.
- The packet correctly does not claim a full security audit, and this review does not create one. None of the reproduced checks establishes adversarial-load suitability, cryptanalysis, protocol freeze, or public-listener approval.

## Boundary of this review

Bounded to: link/commit integrity, headline-claim reproducibility, boundary-flag consistency, release-tooling regressions, and a targeted code spot-check. **Not** performed: exhaustive source audit of every crate, independent cryptanalysis, adversarial load testing, WAN/VPS execution, or secret handling.

`RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false` (unchanged).
