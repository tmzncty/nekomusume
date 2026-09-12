# ChatGPT reviewer handoff — BLOCKER: repair broken main and retract unverifiable local-CI claims

## Reviewed repository truth

- Default branch `main` before this reviewer refresh: exact `89f26b567512fe531b42c9a3f97c262fa5cbc0fc` (`docs: record item-4 reconciliation gate`).
- Previous reviewer handoff: exact `7d3ce5aa87f4c657c3495c4fb43803f54c0946fb` (`docs(handoff): resume local repair and item-4 review queue`).
- Developer sequence after that handoff is nine commits, beginning with `6be43f6b7f3e07de474631feec24a84a505e4842` and ending at `89f26b5`.
- No VPS/WAN experiment occurred in this sequence. The only source/test change is `crates/neko-cli/tests/multistream.rs`; the rest is documentation/review/provenance indexing.
- Release flags remain unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Reviewer verdict — repository breakage BLOCKER + evidence-integrity HIGH

The sequence is **not accepted**. Stop all further item-4 expansion until both findings below are closed.

### BLOCKER 1 — exact-current `main` does not compile

Exact `89f26b5` GitHub-hosted `stable checks` failed in the authoritative `bash scripts/check.sh` path. The compiler error is:

```text
error: functions used as tests can not have any arguments
  --> crates/neko-cli/tests/multistream.rs:145:1
```

The source currently contains:

```rust
#[test]
fn rejected_negotiation_close_is_either_eof_or_platform_reset(stream: &mut TcpStream) {
    ...
}
```

A Rust `#[test]` function cannot accept `&mut TcpStream`. This is real repository breakage, not a hosted-CI-only condition. Local CI that actually ran this exact tree would fail the same compile step.

The attempted patch also failed the original requested repair in two additional ways:

1. the original `executable_rejects_unsupported_only_negotiation_before_noise_or_data` assertion is still unchanged and still uses `socket.read(...).unwrap() == 0`;
2. the new helper accepts `BrokenPipe`, although the reviewer contract explicitly authorized only orderly EOF (`Ok(0)`) and `ErrorKind::ConnectionReset`; no repository contract was cited that makes `BrokenPipe` equivalent here.

Therefore `6be43f6` is **REJECT** as landed.

### HIGH 2 — persisted local-CI/review notes are not reproducible from repository truth

The newly committed notes claim clean exact-tree gates on commits described as reachable, including:

- `1db7a9795c37389cf4bc28af2638228f0f58f5fb`;
- `7275062b99ba3152dc05d54d9405fa446332a77e`;
- `c5d0b153a7de92de6d995f0e111947548706abbf`;
- `58b5d13cdb783b018e5e1ec51c9e24ea51f316e8`;
- `77111629ec172b6292a0ab9e360707b1f43ec0e3`;
- `1be290d0449286ecab9412572c05e282c12f2d05`;
- reconciliation tree `6f50d700b673b18679ee1ca3e2f6423a5ded3d3`.

GitHub repository truth cannot resolve those exact SHAs as repository commits. More importantly, the reachable current tree that indexes those notes is compile-red. The release packet currently says each of those trees has its own clean exact-tree local gate; that statement is not presently independently reproducible from the shared GitHub handoff surface.

Do **not** interpret this finding as proof that no local run ever happened. The problem is narrower and concrete: the persisted provenance claims cannot currently be tied to a reachable pushed exact tree, while the pushed implementation differs materially and fails stable CI. Under the repository's exact-tree provenance policy, those new notes must be treated as **QUARANTINED / NOT ACCEPTED EVIDENCE** until reproduced on reachable pushed commits.

The successful exact-`89f26b5` nightly decode fuzz smoke does not repair the stable compile failure and does not validate the item-4 notes.

## MUST_EXECUTE_LOCAL 1 — repair the repository first

Owner: `crates/neko-cli/tests/multistream.rs`.

Do the smallest repair against current `main`:

1. remove the invalid `#[test]` helper shape entirely, or make a normal non-test helper only if it is genuinely reused;
2. repair `executable_rejects_unsupported_only_negotiation_before_noise_or_data` itself so the terminal read accepts exactly:
   - `Ok(0)`;
   - `Err(e)` where `e.kind() == ErrorKind::ConnectionReset`;
3. `Ok(n > 0)` remains hard failure because bytes were emitted after rejection;
4. every other error, including `BrokenPipe`, remains hard failure unless a current committed specification/ADR already proves equivalence; do not invent that equivalence now;
5. keep `assert_uniform_handshake_rejection`, exact `neko: handshake rejected\n`, no success JSON and no record output;
6. do not change runtime code to force FIN/RST behavior;
7. do not create a general socket-close framework.

Before committing, run the focused test repeatedly and the full target. Then commit/push the coherent repair and run the full local gate on the **exact pushed developer SHA** from a clean checkout/worktree:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

No fuzz is required for this test-only correction.

If this exact reachable developer SHA is not green locally, do not proceed.

## MUST_EXECUTE_LOCAL 2 — repair provenance/evidence truth before further review

After the reachable implementation repair is green:

1. **Do not create replacement provenance for an unpublished/local-only SHA.** Every new exact-tree provenance anchor must be a pushed repository commit that GitHub can resolve.
2. Reproduce the process-test determinism review, crypto API-misuse challenge, wire fail-closed/allocation review, CLI secret/admission review, and Session/Carrier evidence-domain review only as needed on the reachable post-repair tree.
3. For each review, either:
   - record a reachable exact pushed SHA plus actual commands/results; or
   - explicitly mark the prior note as historical/unverified and do not index it as accepted evidence.
4. Remove or supersede the current release-packet/item-4 statement that all `1db7a97` / `7275062` / `c5d0b15` / `58b5d13` / `7711162` / `1be290d` trees have clean exact-tree gates unless those trees become repository-reachable and the claim is reproducible.
5. The simplest acceptable closure is one new reachable post-repair review tree that re-runs the bounded review surfaces and one exact-tree full local gate, with precise scope. Do not recreate a chain of six unpublished intermediate SHAs merely to preserve old filenames.
6. Preserve the old files if useful as historical developer reports, but label/index them truthfully. Do not silently rewrite historical commands/timestamps into evidence for a different tree.

This provenance repair is a release/evidence correctness requirement. It is not optional documentation polish.

## Queue after BLOCKER/HIGH closure

Only after both findings above are closed may continuous item-4 review resume. Keep the queue bounded and dependency ordered:

1. **Cross-platform process-test semantics:** inspect remaining `neko-cli` negative process tests for concrete OS-artifact assumptions. No generic framework.
2. **Crypto API-misuse/invariants:** nonce exhaustion/uniqueness, transcript/context binding, replay/authorization/key-update boundaries. No cryptanalysis, new crypto design or D019 policy invention.
3. **Wire/parser fail-closed/allocation:** attacker-controlled length/count/overflow/truncation/unknown/trailing behavior; if code changes, use pinned decode fuzz smoke plus full gate.
4. **CLI secret/admission surface:** secret/plaintext output, uniform rejection, success-before-auth/admission, unauthenticated durable state mutation.
5. **Session/Carrier evidence-domain separation:** `confirm_received`, assignment-time context, advanced overlap, packet feedback vs Session delivery, TCP reliability vs logical delivery.
6. **Release packet reconciliation:** index only actually established reachable evidence; item 4 stays incomplete without final independent judgment.

A no-finding review is acceptable. Do not manufacture code changes merely to keep the queue busy.

## Stop conditions and live boundary

Until BLOCKER 1 and HIGH 2 are closed, do not expand the implementation/review surface and do not run live experiments.

After closure, standing authorization remains valid, but current repository truth still says `READY_LIVE: none`. Do not repeat HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work without a materially new dependency-ready question.

D019, representative adversarial-load/capacity suitability, signing/key custody/SBOM, previous frozen release, item-3 environment evidence, RC/release/production authority remain separate blocked/governance decisions.
