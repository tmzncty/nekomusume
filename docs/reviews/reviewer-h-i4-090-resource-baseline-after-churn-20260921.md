# H-I4-090 — malformed-UDP resource baseline sampled after the churn

**Severity:** HIGH — release/item-4 evidence-truth defect.

**Reviewed head before this note:** `9c5a9c2f2482549e1382ee4361b8418e315b09cb`.

**Changed owner:** `crates/neko-cli/tests/probe.rs`, source/test repair at
`26a2c40be8a524024f58c7b771aec8b924ae6524`.

## Claim challenged

The malformed-UDP churn fixture is intended to challenge resource growth across
a bounded hostile-input window. On Linux its `/proc/<pid>/fd` and `VmRSS`
observations must be affirmative and the test must compare a pre-churn baseline
to a post-churn observation. A green test must not be able to erase the attack
window by taking both samples after the workload.

## Concrete counterexample

Before H-I4-089, exact `42be53917ee58359ad86532a6aab0136f75047b9`
performed:

1. server READY;
2. `before = process_resource_snapshot(pid)`;
3. eight malformed UDP sends from distinct source ports;
4. bounded settle;
5. `after = process_resource_snapshot(pid)`;
6. FD/RSS growth assertions.

Exact `26a2c40` correctly made the `/proc` helper Linux-only and made missing
Linux observations fail closed, but while moving the code it also moved the
`before` sample to *after* the malformed sends and the settle sleep. Current
code now performs `before` and `after` back-to-back inside the post-churn Linux
block.

Therefore the fixture can pass even if the malformed-input window itself caused
persistent FD/RSS growth: both samples see the already-grown post-churn state.
The assertions only prove approximately "two adjacent post-churn observations
are similar", not "the bounded churn did not grow resources beyond the allowed
margin".

This is a new evidence defect introduced by the H-I4-089 repair. It does not
reopen the narrower macOS `/proc` portability finding: Linux-only scoping and
`None` fail-closed behavior are directionally correct.

## Why this blocks the current PORT/resource closure

`SECURITY.md` requires per-connection/global resource limits, and release item 4
requires verification of resource/abuse limits. The fixture is one of the
current executable challenge points for malformed UDP/resource growth. A clean
exact-tree gate at `26a2c40` proves the test suite passed as written; it does not
make a mutation-insensitive oracle into valid resource evidence.

The developer provenance at
`docs/notes/h-i4-089-provenance-26a2c40-20260921.md` remains truthful as
**developer-local execution provenance**, but it cannot support the resource-
growth subclaim until this oracle is repaired.

## Minimal closure contract

1. Restore the Linux affirmative **pre-churn** snapshot after server READY and
   before the first malformed datagram is sent.
2. Keep the Linux-only `/proc` boundary from H-I4-089. Do not re-generalize the
   measurement to macOS/Unix.
3. Take the post-churn snapshot only after the malformed sends and existing
   bounded settle point.
4. Both Linux snapshots must be affirmative; missing/failed observation must
   fail the resource claim rather than skip it.
5. Compare the true pre/post pair with the existing committed FD/RSS margins.
   Do not choose new resource/capacity policy values in this repair.
6. Add a mutation-sensitive source/test guard so moving the baseline after the
   attack window cannot silently preserve a green resource-growth test. A small
   helper/harness seam is acceptable if it remains local to this test contract;
   do not build a new checker framework merely for this finding.
7. Preserve the non-Linux Unix portable socket/lifecycle portion without
   fabricating `/proc` evidence.
8. Run focused affected tests, then on the final pushed source/test SHA in a
   safe clean checkout/worktree run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree
   and persist exact reachable provenance with UTC start/end, exit codes,
   OS/arch and Rust stable version.
9. No fuzz is required unless decoder/parser/crypto framing changes; this repair
   should not touch those owners.

## Review of the other new implementation owners since `c8bc0f5`

- H-I4-087 Session aggregate send+recv record-cap repair (`14e2f52` plus focused
  coverage at `d819d38`): no new counterexample found in this bounded pass.
- H-I4-088 MemoryCarrier empty-message queue bound (`26aa4e8`): no new
  counterexample found in this bounded pass; it reuses the already committed
  queue-byte bound rather than inventing a new numeric policy value.
- I4-PORT-01 Unix process-fixture scoping (`42be539`): the later H-I4-089
  correctly identified that `/proc` itself was Linux-only; H-I4-090 is the
  remaining current-owner oracle-ordering defect.

## Evidence boundary

This finding is from exact-current source/diff reasoning. **No reviewer-local
Rust test, `scripts/check.sh`, fuzz run, hosted CI, WAN run or performance run is
claimed here.** GitHub combined status/workflow lookup for exact `26a2c40`
returned no hosted run; absence is neither success nor failure.

`READY_LIVE: none` remains unchanged. No D019 choice, release authority,
Session/Carrier/ACK/crypto/wire architecture, signing/SBOM/key-custody policy, or
capacity/security numeric value is changed by this finding.
