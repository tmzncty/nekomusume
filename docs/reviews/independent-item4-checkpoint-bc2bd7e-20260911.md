# Independent bounded item-4 evidence challenge — checkpoint at exact `bc2bd7e`

**Reviewed revision:** `bc2bd7ef14e91897bd43f87c734e15fde448942f` (`origin/main`), with the code-bearing ancestor `287e2223d40b7b43dfa2a14ba1992e3bf845a0f2` and the schema-repair ancestor `1222270870763f5a5f053a1ded80266e2ca669a0`. `schema/` and `scripts/` are byte-identical across `1222270`, `287e222`, local/remote tips (verified by blob ids).
**Review type:** independent bounded evidence challenge across the item-4 gate list. **Date:** 2026-09-11. Host `tmzn@192.168.122.1`, workdir `/media/tmzn/DATA5/nekomusume-work`.
**Result: no concrete current correctness/security/evidence contradiction found.** This is a checkpoint classification, not an audit, security approval, release decision, or production authorization.

## Verified at this exact head

- `schema/benchmark-result.v1.json` **now conforms to both real producers** (Draft 2020-12): the generic `compare-hy2.sh` envelope (all-success and one-failed-sample shapes) and the owned-lab `compare-hy2-owned-lab.sh` envelope both validate. The previously reported common-envelope drift is closed by `1222270`, `bounds` is now optional with a documented producer-boundary rationale (`docs/bench/result-schema-v1.md`), and `compare-hy2-test.sh:24-28` adds a real `validate_common` consumer so the drift cannot silently recur.
- `schema/benchmark-blocked-harness.v1.json` now declares bounded `client_diagnostic`; all three retained `artifacts/hy2-owned-lab/*/result.json` (including `13da094`) conform to their own schema.
- **v0 separation holds:** `nekomusume.bench.v0` (deterministic recovery, `docs/bench/latest-deterministic.json`) and `nekomusume.netns-bench.v0` (privileged netns, `docs/bench/latest-netns.json`) are separate producers/artifacts; `result-schema-v1.md` explicitly bounds the common schema to paths emitting `nekomusume.benchmark-result.v1` and declines to migrate the historical surfaces.
- **Canonical corpus scope:** `scripts/validate-canonical-vectors.py` -> `42 vectors; domains=10; freeze=true`; corpus-scoped freeze only, no protocol/interop freeze claim.
- **Package/archive anchors:** cited `4e54f320…jsonl` (distinct-version A/B/A) matches on disk; `smoke-package-test.sh` -> `package-smoke-regressions-ok`; `check-clean-source-test.sh` -> `package-clean-source-regressions-ok`.
- **Lifecycle/cleanup:** retained `7cebe6b` lifecycle anchor present; `14be1c8` remains an immutable orchestration negative; the `9fd2411` repeated-failover typed negative matches `4744a3b4…`.
- **Session/Carrier and resource controls:** `crates/` unchanged since `e5fefc1`; `cargo test -p neko-crypto --all-targets` -> `37 passed; 0 failed`; `check-preauth-responder-inventory.py` -> `9 surfaces, 7 admission sites`; the nine responder charge-before-work/amplification/permit/cleanup controls remain as independently audited.
- **HY2 distinction:** `13da094` remains `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair, median/P95, performance, or superiority result; same-class retries frozen. Blocked vs complete vs performance remain separated by schema and by the specialized Python validator.
- **Governance checks green:** `check-era4-closure.py` (0 open-ready, 16 already-sufficient, 0 dependency-blocked), `check-plan-sync.sh`, `check-status-evidence.sh`, `check-release-boundaries.sh`, `check-markdown-links.sh`, `check-governance-status.sh`, `check-decision-index.sh` (65 decisions), `check-status-coverage.sh`, `check-evidence-manifests.py` (29 files) all pass. No missing links; all cited 40-hex refs reachable; six cited 64-hex artifact hashes match on-disk files.

## Remaining item-4 / release gates (classification only)

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.
- Release **item 3 unchecked**; **natural-loss evidence unchecked**.
- **D019** remains `SOURCE_RETENTION_POLICY_BLOCKED` (terminal source-state retention vs no-reset accounting is a maintainer policy choice; no TTL/LRU/history invented).
- **RSEC-001** open: non-policy engineering controls reviewed, but adversarial-load suitability and promotion remain absent.
- **HY2** blocked (current-line), **periodic** and **repeated warm failover** blocked (orchestration current line); **IPv6** `BLOCKED_ENVIRONMENT`; live **PMTUD** `BLOCKED_IMPLEMENTATION`.
- No signing/key-custody/SBOM publication workflow; native aarch64 execution unverified (candidate target); no sustained/public reachability or production/service-manager hardening.
- **Item 4 incomplete**: item-4 evidence index now includes three bounded independent reviews (release/security evidence index, non-policy resource controls, HY2 comparison methodology) plus this checkpoint; none is a full security audit or release approval.

## Boundary of this checkpoint

Performed: producer/schema conformance reproduction, retained-artifact and anchor verification, focused regression runs, and the repository's own fast policy checks at the exact head. **Not** performed: independent cryptanalysis, adversarial/load testing, WAN/VPS execution, penetration testing, D019 policy selection, or secrets access. This checkpoint closes nothing and authorizes nothing.
