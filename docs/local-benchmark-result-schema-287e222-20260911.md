# Local benchmark result schema closure — `287e222`

Developer-local clean exact-tree validation of reachable comparison schema/producer closure commit `287e2223d40b7b43dfa2a14ba1992e3bf845a0f2`.

## Change and boundary

The common `nekomusume.benchmark-result.v1` structural schema now matches the generic and owned-lab complete comparison producers: array summaries, truthful nullable failed-sample values, optional bounds when no producer-enforced whole-run deadline exists, bounded owned-lab cleanup evidence, and repository-root commit provenance. The separately versioned `nekomusume.benchmark-blocked-harness.v1` schema includes the bounded `client_diagnostic` retained by exact `13da094`. Focused Draft 2020-12 regressions exercise real producer/fixture shapes and fail-closed structural mutations; the stronger owned-lab Python validator remains the semantic authority.

The documentation scope is limited to controlled comparison paths that emit `nekomusume.benchmark-result.v1`. Existing deterministic-recovery `nekomusume.bench.v0` and netns `nekomusume.netns-bench.v0` producers/artifacts remain separate and unchanged. No historical artifact was rewritten and no live benchmark occurred.

## Verification

On a clean detached checkout of exact `287e222`:

- `bash scripts/bench/compare-hy2-test.sh`: exit `0`;
- `bash scripts/bench/compare-hy2-owned-lab-test.sh`: exit `0`;
- `python3 scripts/bench/validate-hy2-owned-lab-test.py`: exit `0`;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`;
- `git diff --check`: exit `0`;
- UTC `2026-09-11T00:14:15Z -> 00:16:13Z`;
- Linux x86_64; Rust `1.98.0`;
- initial/final source tree clean.

A prior exact-`1222270` full-suite attempt had all focused benchmark/schema tests green but one unrelated SIGTERM lifecycle test failed before its helper observed READY; the same exact test then passed in isolation. This note anchors closure only to the later clean replacement tree `287e222`, whose full gate passed.

This is developer-local evidence, not reviewer-executed or GitHub-hosted CI, a performance result, security approval, release, or production evidence.
