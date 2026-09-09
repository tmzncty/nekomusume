# Release item-4 subgate factual review — exact `0a81d99`

## Scope

Developer-prepared factual review support for reachable exact commit `0a81d99ceb9bd0f950f67e42fdc9e9ee6191c314`. This is not an independent review, security approval, release decision, protocol freeze, RC, public-listener approval, or production authorization. Release flags remain false.

## Verified subgate facts

### Package provenance and rollback

- Exact `a9928c8` package was built reproducibly in the recorded environment; packaged and installed VPS binary hashes matched, and bounded authenticated TCP/UDP package smokes completed with cleanup.
- The package builder rejects staged, unstaged tracked, and non-ignored untracked source state before build/output mutation while allowing ignored generated output; focused regressions are exercised by `scripts/check.sh`.
- Current package smoke validates exact archive shape, member type and archived mode before extraction, rejects path escape, links, special members and unexpected layout, then enforces checksums/modes and native secret-free capabilities. Synthetic adversarial archives, including umask-normalization cases, are exercised by `scripts/check.sh`; this is release-tool validation, not signing or publication trust.
- Exact `7cebe6b` installed-package lifecycle observation completed TCP and UDP `READY -> DRAINING -> STOPPED`, listener release, same-port restart/rebind, authenticated 32-byte post-restart exchanges, and cleanup.
- Historical N5 remains the older distinct-version A→B→A rollback observation. The separate exact `91a735c -> dc90c5f -> 91a735c` rehearsal uses genuinely different package/binary hashes, retains an external temporary identity/state marker for the bounded probe contract, and returns the selected release to A; its retained sanitized phase anchor is five records with exit status `0` and SHA-256 `4e54f3205cf295ec82e9186eb6b6271558e6babddae819c5ef34e1595a0ddb09`, but experiment start/end timing was not retained. Neither observation establishes arbitrary state-schema compatibility, publication signing, SBOM/provenance service, service-manager hardening, or production deployment.

### Canonical-vector status

- Corpus v1 remains content-addressed and mechanically validated at 42 vectors across 10 domains with `freeze=true`.
- That freeze is corpus-scoped only. It is not a full protocol/interoperability freeze or release approval.

### Comparison methodology and HY2

- The paired comparison harness retains ordered raw prefixes, suppresses comparative summaries for incomplete pairs, validates payload/resource/lifecycle identity, and bounds/redacts diagnostics.
- Current exact `13da094` HY2 evidence remains `BLOCKED_HARNESS_CURRENT_LINE_HY2` at `unknown / client_started`: Nekomusume sample 1 succeeded, HY2 sample 1 exited before application bytes, so no complete pair, median/P95, performance, superiority, or general HY2 reliability claim exists. Same-class retry remains frozen.

### Negotiation, Noise, trust, and parser boundaries

- Current deterministic gates cover canonical version negotiation before data admission, exact negotiation/transcript binding into Noise, allowlist/trust rejection, tamper/replay rejection, bounded framing, malformed/truncated/oversize rejection, and fuzz-smoke.
- These are implementation/test facts, not an independent cryptographic/security approval or frozen public protocol claim.

### Pre-auth resource controls and D019

- Bounded non-policy engineering controls remain reviewed: carrier/source projection, admission before expensive work, input/response accounting, source/global state/queue/memory bounds, terminalization/rollback, expiry/release cleanup, aggregate redacted observability, and listener inventory.
- Full RSEC/D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`: terminal release can remove final source state while D019 currently requires source-lifetime accounting not to reset. No TTL, LRU/history capacity, external authority, or weakened no-reset semantics is selected here.
- Deterministic tests do not establish adversarial-load capacity suitability or formal security approval.

## Exact-tree gates

On reachable exact `0a81d99`, developer local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check` passed with final clean-tree verification before this review-text update was committed; minimal provenance is retained in [`../local-ci-0a81d99-20260909.md`](../local-ci-0a81d99-20260909.md). The earlier exact-`188d5a5` local run and GitHub-hosted run `34298672407` are separate evidence. This includes workspace tests, clippy/fmt, status/evidence/governance checks, canonical vector validation, package/HY2/repeated-failover harness regressions, and release-boundary checks. A green gate is not a release/security decision.

## Remaining release boundaries

The repository still lacks formal independent security/release approval, full D019 policy closure, signed publication/key-custody/SBOM workflow, native aarch64 execution evidence, sustained/public/general reachability evidence, natural-loss evidence, and production/service-manager hardening. HY2 and repeated-failover same-class runs remain frozen. Live PMTUD requires an explicitly accepted authenticated wire/security design gate. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
