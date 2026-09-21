# Reviewer repository-wide item-4 independent-review inventory — exact `7197abf`

**Repository anchor at inventory start:** reachable `7197abf037988adf3a4ac097429dc047b5f52f33`.

This is a repository-wide coverage/refill inventory, not a security approval, release decision, reviewer-local CI run, hosted-CI claim, WAN result, or performance conclusion. I re-read the current governance/spec/status/handoff/release-packet owners and the exact-current source/test changes since the prior reviewer handoff. Release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.

## Current 13-surface inventory

| # | Surface | Current bounded independent support | Inventory verdict |
|---|---|---|---|
| 1 | `neko-reliable` ACK/loss/retransmit/RTT/PTO/Reno/fault semantics | deep reliable-UDP review plus current Recovery boundedness completion through reachable `031f7b8`; Candidate A remains closed absent owner change | **covered/current enough** |
| 2 | `CarrierState` generation/validation/hysteresis/single-active/drain/fail/activate | `independent-carrier-state-8f93b93-20260913.md`; current carrier changes are in `MemoryEndpoint`, not this owner | **covered/current** |
| 3 | Concurrent Carrier Manager / health / migration-back | `independent-carrier-manager-a14cf47-20260913.md`; later carrier edits do not touch the reviewed manager owner | **covered/current** |
| 4 | FairScheduler / multi-stream / Session+stream flow-control | earlier scheduler independent challenge remains owner-current; Session flow-control has current independent post-H-I4-086/087 re-challenge at `independent-i4-fs2-session-flow-control-11cdb08-20260921.md` | **covered/current** |
| 5 | Memory/UDP/TCP adapter close/error/resource semantics | current Memory independent re-challenge at `independent-i4-ad1-memorycarrier-rechallenge-d0f4c55-20260921.md`; UDP/TCP current bounded reviews sit on unchanged adapter owners relative to the earlier independent adapter review | **covered/current** |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting | current FS2 independent review covers repaired ACK-before-drain + aggregate queue ownership; prior terminal/lifecycle reviews remain separate support; retained `events` capacity remains a policy gate, not an invented implementation value | **covered/current within non-policy scope** |
| 7 | `neko-observe` projection/event/counter/high-water correctness | prior independent observability re-close remains owner-current; no material post-review owner change; Candidate B remains closed | **covered/current** |
| 8 | package/reproducibility/operator scripts | prior dedicated independent package/release challenge remains owner-current; no material package/operator owner change in this refill group | **covered/current** |
| 9 | Cargo manifests/lock/features/build/native hooks/unsafe inheritance | owner manifests/lock are unchanged; H-I4-085 corrected false evidence wording about `snow`/`ring` without changing dependency/default-feature/crypto selection; prior independent dependency-surface review remains source-current | **covered/current; prose truth recently repaired** |
| 10 | cross-platform CLI/process-test semantics | independent follow-up at `98978ee` found an explicit Unix/POSIX scope gap; repair `42be539` then cfg-gated the affected helper/tests, but there is no dedicated independent post-repair bounded closure note | **READY_LOCAL gap: post-`42be539` re-challenge** |
| 11 | CLI exit-code / JSON / human-output contract | machine-result contract has dedicated independent review at `independent-i4-cli-machine-1b1b8c8-20260921.md`, which explicitly left human wording to I4-CLI-H; the subsequent human-output note is developer-authored. Current `matrix_probe` uses literal `artifact.contains("\"reachable\":true")`, not typed parsing; reconciliation wording was corrected at `7197abf` | **READY_LOCAL gap: independent human-output/stderr re-challenge** |
| 12 | algorithmic resource boundedness under existing limits | current developer BND review plus independent current Recovery boundedness, current independent Session FS2 and Memory AD1 reviews jointly cover the owners changed by H-I4-087/088; no capacity-pressure/adversarial-load policy is selected | **covered piecemeal; READY_LOCAL synthesis challenge is still useful** |
| 13 | release packet factual consistency / evidence boundary | `dd166b3` + `588ca` reconciled the closure group; exact-current reviewer repair `7197abf` corrected the CLI-H parsed-result overstatement. The post-reconciliation packet/status/plan boundary has not yet received a dedicated independent spot-check | **READY_LOCAL gap: post-reconciliation evidence-boundary check** |

## Refill consequence

The repository is **not queue-exhausted**. The narrow PORT/CLI/BND group is coherent enough to advance, but three dedicated current independent gaps remain and one cross-surface synthesis remains valuable before any final item-4 claim:

1. independently re-challenge the exact post-`42be539` cross-platform/process-test scope and confirm Unix-only lifecycle fixtures no longer masquerade as non-Unix execution evidence;
2. independently challenge I4-CLI-H human-output/stderr/exit truth, including the exact literal-substring reachability gate and its one-case-schema boundary;
3. independently synthesize current algorithmic boundedness across the newly repaired Session/Memory owners plus Recovery under **existing** limits only, without inventing policy values or running capacity pressure;
4. independently spot-check `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current reconciliation/provenance notes and release flags after the closure-group reconciliation;
5. then perform the final repository-wide item-4 sweep/refill rather than declaring completion from these lanes alone.

No new code/instrumentation/hypothesis/path condition created an unresolved real-network question, so the authoritative live classification remains `READY_LIVE: none`. D019/source-retention and `SessionRuntime.events` history capacity remain maintainer/security policy gates and are not allowed to block the independent local lanes above.

## Evidence boundary

This inventory is source/spec/review-note inspection. **No reviewer-local commands/tests were executed in this environment.** Developer clean exact-tree provenance stays separately labelled at its own pushed SHAs. No absent GitHub-hosted run is interpreted as success or failure. No decoder/parser/crypto-framing code changed in this reviewer-only slice, so no fuzz claim is made.
