# Local item-4 anchor-repair gate — `f4f7f93`

Historical developer report for pre-rebase local label `f4f7f93`. That object is not repository-reachable, so this file is **quarantined / not accepted as exact-tree evidence**. Reachable commit `563532f` introduced this report; current accepted navigation is rebuilt from reachable `4034f86` and later commits.

## Reconciled provenance

- `dc52a5e` is the GitHub-reachable consolidated source/test anchor for the five-domain item-4 challenge review.
- Historical label `eb5f0e5` was claimed as the consolidated evidence-note commit but is not repository-reachable; that exact-tree gate claim is quarantined / not accepted. Reachable `c0b2ee0` introduced the consolidated report.
- Historical label `f4f7f93` described the index repair but is not repository-reachable. Reachable `563532f` introduced this quarantined report; current authoritative navigation is the later reachable repair chain.
- Stale `ConnectionReset/BrokenPipe` wording is removed from current support; the accepted contract accepts clean EOF or platform reset and still rejects any received bytes.
- The rolling navigation remains `0 open-ready`, but item 4 is not marked complete or independently reviewed.

## Verification

Focused checks:

- `python3 scripts/check-era4-closure.py`: passed
- `bash scripts/check-plan-sync.sh`: passed
- `bash scripts/check-status-evidence.sh`: passed
- `bash scripts/check-release-boundaries.sh`: passed
- `bash scripts/check-markdown-links.sh`: passed
- `git diff --check`: passed

Exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-12T11:07:39Z` -> `2026-09-12T11:09:34Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is evidence navigation/provenance only. It is not independent review, security approval, D019 closure, RC, release, public-listener or production authorization.
