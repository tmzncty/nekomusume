# Independent bounded comparison-methodology review (HY2 owned-lab) — exact `e5fefc1`

**Reviewed revision:** `e5fefc1c90b38a8c4a1d2ebb46c4f6262c27a3a9` (`origin/main`).
**Review type:** independent static/deterministic review of the HY2 owned-lab runner/validator and its claim-suppression boundary. **Date:** 2026-09-11 (UTC `2026-09-10T21:18:52Z`). Review host and checkout path intentionally sanitized.
**Artifacts reviewed:** `scripts/bench/compare-hy2-owned-lab.sh` (sha256 `5569f57d…`), `scripts/bench/validate-hy2-owned-lab.py` (sha256 `f93bfb1a…`), `scripts/bench/validate-hy2-owned-lab-test.py` (sha256 `ca334636…`), `scripts/bench/compare-hy2.sh`, `docs/bench/result-schema-v1.md`, `docs/bench/hy2-comparison-workload.md`, the three retained `artifacts/hy2-owned-lab/*/result.json`, and the packet/item-4/status HY2 claims.
**Not** a security audit, cryptanalysis, adversarial-load assessment, live/WAN run, penetration test, or production approval. No VPS/WAN execution, no secrets touched.

## Verdict

**BLOCKER: none. HIGH: none** in the HY2 comparison core controls. **One bounded, reproducible MEDIUM evidence-integrity defect** was found in the BLOCKED-artifact validator branch (details below); it is **not reachable from the harness runner** and does not affect any retained artifact. Reported, **not edited** (repair boundary and verification given below).

Core question answered: **incomplete pairs, failed HY2 samples, cleanup-negative artifacts, and frozen current-line results cannot emit or imply median/P95/superiority/comparative success through the harness.** The one gap is confined to externally hand-authored BLOCKED artifacts.

## Reproduction (all commands run at `e5fefc1`, tree clean)

- `python3 scripts/bench/validate-hy2-owned-lab-test.py` -> `R-HY2-15-mutation-matrix-ok 15` + `validate-hy2-owned-lab-test-ok`.
- `bash scripts/bench/compare-hy2-owned-lab-test.sh` -> `compare-hy2-owned-lab-test-ok`.
- `bash scripts/bench/compare-hy2-test.sh` -> `compare HY2 tests: PASS`.
- `python3 scripts/bench/validate-hy2-owned-lab.py validate-result <each retained artifact>` -> `validated` for `13da094`, `3d54585`, `bc38d06`.
- Each retained `BLOCKED_HARNESS` artifact has exactly the 9 canonical top-level keys and `summary` absent.

## Claim-suppression controls verified (no defect)

1. **Incomplete sample set** — `validate_samples(..., require_complete=True)` rejects a non-full set (`validate-hy2-owned-lab.py:341-342`), so a complete result must contain the exact interleaved `nekomusume`/`hy2` set.
2. **Failed HY2 sample** — a complete result with any `failures != 0` is rejected (`:343-344`), and the R-HY2-15 matrix explicitly asserts a complete artifact cannot relabel a failed sample by recomputing its summary around the remaining successes (`validate-hy2-owned-lab-test.py:55-63`).
3. **Forged comparative summary** — `doc["summary"]` must equal `expected_summary(samples)` (`:405-406`); I confirmed a forged favorable summary is rejected (`summary does not match complete retained sample set`).
4. **BLOCKED artifact** — the literal `summary` key is rejected (`:383-384`), and the runner's blocked path constructs a fixed-shape doc with no summary (`compare-hy2-owned-lab.sh:133`; validator `blocked` subcommand `:481-488`).
5. **Cleanup-negative artifacts** — a failed/unknown cleanup stays `BLOCKED_HARNESS` with truthful `cleanup_status` (`:395-397`); it cannot become a complete result because the complete branch demands `cleanup_status == "verified"` and exact zero-residue evidence (`:448-449`).
6. **Frozen current-line** — the frozen state is a ledger/status classification (`docs/status.md:262`, ledger `notes.hy2`), not emitted by the harness; there is no code path that converts a frozen line into a summary.
7. **Runner fail-closed** — after emitting a candidate complete result, the runner gates on the validator and diverts to `blocked` on rejection: `python3 "$validator" validate-result "$out.tmp" >/dev/null || blocked validation` (`compare-hy2-owned-lab.sh:229`). Median/P95 are computed only from retained successes (`:358`), and `wire_bytes` stays `null`.

## Finding: BLOCKED_HARNESS validation is not key-exclusive

**[MEDIUM — evidence-integrity hardening]** — `scripts/bench/validate-hy2-owned-lab.py:376-377`.

The blocked branch checks `required.issubset(doc)` and rejects only the literal key `summary` (`:383-384`). It does **not** enforce an exact key set, so a BLOCKED/incomplete artifact carrying an arbitrarily named comparative field still validates. Documented guarantee contradicted: `docs/bench/result-schema-v1.md` ("It contains no partial summary or superiority claim") and `docs/reviews/release-item4-subgates-20260909.md:25` ("suppresses comparative summaries for incomplete pairs").

**Exact reproducer** (validator loaded from the reviewed file; blocked doc with one extra key):

```
python3 - <<'PY'
import importlib.util,pathlib,tempfile,copy
p=pathlib.Path('scripts/bench/validate-hy2-owned-lab.py'); s=importlib.util.spec_from_file_location('v',p)
v=importlib.util.module_from_spec(s); s.loader.exec_module(v)
tmp=pathlib.Path(tempfile.mkdtemp()); h='0'*64
def row(i,r,f=0): return {'name':f'{i}-{r}','implementation':i,'run':r,'failures':f,'elapsed_seconds':1.0 if not f else None,'cpu_user_seconds':.1 if not f else None,'cpu_system_seconds':.1 if not f else None,'rss_kib':1 if not f else None,'fd_count':1 if not f else None,'application_bytes':1200 if not f else 0,'payload_sha256':h if not f else None,'wire_bytes':None,'exit_code':0 if not f else 1,'failure_stage':None if not f else 'client_exit'}
cl={'local_processes_reaped':True,'local_listeners_remaining':0,'remote_process_groups_reaped':True,'remote_listeners_remaining':0,'remote_temp_path_removed':True}
base={'schema':'nekomusume.benchmark-blocked-harness.v1','experiment_id':'x','git_commit':'0'*40,'status':'BLOCKED_HARNESS','failure_stage':'hy2-1-failed','contract':{'runs_per_implementation':5,'payload_bytes':1200,'payload_prepared':True,'payload_sha256':h},'samples':[row('nekomusume',1),row('hy2',1,1)],'cleanup_status':'verified','cleanup_evidence':cl}
for label,extra in [('summary',{'summary':[{'implementation':'hy2','median_exchange_latency_ms':0.1}]}),
                    ('median_exchange_latency_ms',{'median_exchange_latency_ms':0.29}),
                    ('superiority',{'superiority':'hy2 5x faster'}),
                    ('comparative_summary',{'comparative_summary':{'hy2_vs_nekomusume_p95_ratio':0.2}}),
                    ('p95_latency_seconds',{'p95_latency_seconds':0.29})]:
    d=copy.deepcopy(base); d.update(extra); f=tmp/'z.json'; v.atomic_write(f,d)
    try: v.validate_result(f); print('ACCEPTED',label)
    except ValueError as e: print('REJECTED',label,'->',e)
PY
```

Observed at `e5fefc1`: `summary` REJECTED; `median_exchange_latency_ms`, `superiority`, `comparative_summary`, `p95_latency_seconds` all **ACCEPTED**.

**Reachability:** the harness runner cannot produce such a doc (fixed 9-key emission at `compare-hy2-owned-lab.sh:133` / validator `:486`), and all three retained artifacts have exactly the 9 canonical keys. The exposure is limited to externally/hand-authored BLOCKED artifacts that are then validated (`--validate`, or committed and checked). No retained artifact exercises it.

**Minimal repair (verified in a `/tmp` copy; repository not edited pending maintainer decision):** in the blocked branch, make the required set exact — add `"contract"` to `required` (`:376`) and replace `required.issubset(doc)` with `set(doc) != required` (`:377`). `"contract"` must be added because the canonical blocked doc (runner `:486`, test fixtures) legitimately carries it.

```
- required = {"schema","experiment_id","git_commit","status","failure_stage","samples","cleanup_status","cleanup_evidence"}
+ required = {"schema","experiment_id","git_commit","status","failure_stage","contract","samples","cleanup_status","cleanup_evidence"}
- if not required.issubset(doc) or doc["status"] != "BLOCKED_HARNESS" or not doc["failure_stage"]:
+ if set(doc) != required or doc["status"] != "BLOCKED_HARNESS" or not doc["failure_stage"]:
```

Repair validation (patched copy only): `validate-hy2-owned-lab-test.py` passes unchanged (`R-HY2-15-mutation-matrix-ok 15`, `validate-hy2-owned-lab-test-ok`); all three retained artifacts remain `VALID`; and the four comparative-key probes above flip to REJECTED while the canonical doc stays ACCEPTED. A one-line mutation regression asserting a comparative-key blocked doc is rejected should accompany the fix.

**[nit] Complete-artifact branch is also not key-exclusive** (`:401-402`): it uses `required.issubset(doc)`, which is intentional because the runner legitimately emits `mode`/`transport`/`scope`/`captured_at`. Consequence: an extra non-canonical key (e.g. a second `superiority` field) is accepted on a complete artifact. The authoritative comparative field is `summary`, which is strictly validated against the recomputed sample set, so no comparative statistic can be forged; this is cosmetic only.

## Explicit exclusions

Performed: static reading of the runner/validator, executable reproduction of the validator and runner test suites, retained-artifact validation, adversarial key/summary mutation probes, and a verified repair prototype in `/tmp`. **Not** performed: live or WAN/VPS execution of the comparison, HY2 re-installation or re-run, independent cryptanalysis, adversarial-load/performance measurement, secret handling, or any change to repository code. This review does not establish HY2 reliability, comparative performance, or suitability.

## Remaining release gates (unchanged)

HY2 remains `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2` at typed `unknown / client_started` for exact `13da094`; there is no complete pair, median/P95, performance or superiority result. Same-class retries remain frozen. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `RSEC-001` open; `D019` `SOURCE_RETENTION_POLICY_BLOCKED`. This bounded review is partial item-4 evidence only and closes nothing.
