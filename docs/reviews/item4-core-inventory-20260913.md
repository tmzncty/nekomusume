# Item-4 core-surface inventory — reachable exact `4e21520`

Repository-wide inventory of every implemented core crate and candidate against the independent-review index, after the full 2026-09-13 deep item-4 sweep including the candidate and measurement lanes. **Supersedes the earlier premature `e10b6f` inventory**, which omitted the executable `Plpmtud`/`FecBlock` candidates and missed the bench P95 inconsistency. This inventory reports coverage status only; the final queue-exhaustion decision and any independent security/release approval remain with the reviewer/maintainer.

## Coverage classification

Every implemented core surface now has a dedicated bounded independent challenge or is explicitly subsumed by a named broader review. Remaining items are genuinely policy/environment/external-authority gates.

### Dedicated independent bounded challenge — complete

- reliable UDP recovery / future+never-sent ACK (`531c82d`/`02b6eaa`)
- CarrierState generation/validation/hysteresis (`0831443`)
- CarrierManager margin/hold/migration-back (`a14cf47`/`91f8cd8`, `5f1cb8d`)
- FairScheduler flow accounting (`496258d`)
- carrier adapters close/error/resource (`31c1769`)
- DeliveryLedger overlap/context/watermark (`5de08e8`)
- SessionRuntime lifecycle/resource/terminal-cleanup (`9697ee7`, superseded note + provenance)
- ProcessMessage/Resume/readiness codec (`321c68f`)
- DatagramRuntime counters/closed-state (`1c13462`)
- crypto Noise-IK integration/API, not cryptanalysis (`cd67a2e`)
- observability projection/ring/drop/schema closure O1–O6 (`8f93b93`/`8a4e465`/`ece67b8`/`8e11de0`, re-close `18f7c58`)
- CLI portability/exit/JSON (`3ef4e85`) + command truth boundaries (`16be0e0`)
- bench evidence semantics + P95 convention (`01b24a0`, `757e0b6`, re-close `e1a4c21`)
- package/reproducibility/release scripts (`1884bab`)
- dependency/build surface (`c0c875c`)
- static boundedness + result validators (`85c7bd1`)
- PLPMTUD candidate state model (`76b14da`)
- disabled XOR FEC candidate (`167f1db`)
- disabled-gate enforcement: 0-RTT / concurrent striping (`6e04684`)
- resource/abuse responder surfaces (`f184cf2`, reviewer)
- wire parser/decoder (`independent-wire-review-4034f86`, reviewer)

### Legitimately subsumed

- HY2 methodology/schema (`independent-hy2-methodology-review-e5fefc1`, reviewer) — covered; not re-opened absent source change.
- Pre-auth responder inventory and TCP EOF/RST semantics — closed in earlier gates.

### Disabled / policy / environment / external-authority only (no executable review question)

- `SessionRuntime.events` retained bound — `POLICY_BLOCKED_RESOURCE_BOUND`.
- D019 persistent source-retention — maintainer policy.
- RSEC-001 adversarial-load/capacity suitability — unestablished, not converted to local pressure test.
- Item-3 natural-loss/long-lived/HY2/IPv6/environment evidence — frozen/blocked.
- Signing/key custody/SBOM/publication trust; previous-frozen-release interop — separate authority gates.
- Independent final security/release judgment; RC/freeze/release/production authority.
- All `READY_LIVE` live lanes — none; Experimental Track candidates (PLPMTUD/FEC/0-RTT/striping) have no executable live path.

## Status

After this corrected inventory, no substantial implemented or candidate core surface lacks a dedicated bounded independent challenge, no concrete repairable defect is open, no READY review-support or READY live question exists, and all remaining items are policy/environment/external-authority gates. Coverage is reported for reviewer judgment; item-4 completion is not self-declared.
