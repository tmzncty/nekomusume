# Local provenance-HIGH closure gate — `289fbad`

Developer-local clean exact-tree validation of reachable commit `289fbaddf04fb8bc5281f207830250c43ff7cb4b`, which completes the current item-4 provenance correction.

## Closure

Authoritative release/item-4 navigation now uses reachable `4034f86` for the corrected TCP test and consolidated bounded developer review, plus independent wire/parser review `5d85e09`. Pre-rebase labels that GitHub cannot resolve (`1db7a97`, `7275062`, `c5d0b15`, `58b5d13`, `7711162`, `1be290d`, `6f50d70`, `eb5f0e5`, `f4f7f93`, and `8d7c147`) are explicitly quarantined and are not counted as accepted exact-tree evidence. Historical reports remain without rewriting their original execution claims.

## Verification

On a clean detached checkout of exact `289fbad`:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`;
- `git diff --check`: exit `0`;
- UTC `2026-09-12T11:17:42Z -> 11:19:38Z`;
- Linux x86_64; Rust `1.98.0`;
- initial/final source tree clean.

GitHub Actions run `34690579549` also passed on exact `289fbad`; hosted CI is additional cross-evidence, not a substitute for the local exact-tree gate.

This closes the current evidence-anchoring defect only. It does not complete release item 4, supply independent maintainer/security approval, select D019 policy, establish adversarial-load suitability, complete item 3, or authorize RC/release/production.
