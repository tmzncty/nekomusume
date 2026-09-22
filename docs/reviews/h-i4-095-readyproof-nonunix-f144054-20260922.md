# H-I4-095 — non-Unix `ReadyProof` cfg break

**Severity:** HIGH (item-4 build/portability correctness; not a runtime transport defect)

**Independent review anchor:** `f1440544aaa7c17d28c4cbdf55ba5c6d94431207`

## Scope reviewed

This bounded pass re-read the exact-current release/governance surface and reviewed the developer-owned H-I4-093/H-I4-094 changes at the current reachable `main`, with focused inspection of:

- `crates/neko-cli/tests/probe.rs`
- `crates/neko-cli/Cargo.toml`
- `docs/CHATGPT_HANDOFF.md`
- `docs/status.md`
- `IMPLEMENTATION_PLAN.md`
- `SECURITY.md`
- `docs/standing-vps-lab-authorization.md`
- `docs/vps-rental-window-priority.md`
- `docs/decisions.md`
- `docs/carrier-architecture.md`
- `docs/specs/nekomusume-session-v0.md`
- `docs/release-security-review-packet.md`

New developer source/test commits reviewed in this interval:

- `b103c9784805b48f8b488d0af705c940c1f82993` — non-Linux Unix `BarrierProof` sink; acceptable for its stated source-shape closure.
- `ca772093f4fc46bda3be9aade00d4fc884b48797` — H-I4-094 readiness-derived pre-baseline proof; Linux causal intent is correct, but this commit introduces the portability defect below.

Documentation/provenance-only commits were separated from source/test evidence; developer-local exact-tree provenance remains developer-reported local evidence, not reviewer-local or hosted execution.

## Finding

`ReadyServer` is compiled unconditionally in `crates/neko-cli/tests/probe.rs` and now contains an unconditional field:

```rust
ready_proof: ReadyProof,
```

but `ReadyProof` itself is declared only under:

```rust
#[cfg(unix)]
struct ReadyProof { ... }
```

The same unconditional type is also constructed by general helpers such as `start_server_for(...)` and `start_periodic_server(...)`.

Therefore, on a non-Unix target (for example a Windows Rust target), the integration test translation unit still sees `ReadyServer` and the unconditional initializers but the `ReadyProof` type does not exist. This is a deterministic cfg/name-resolution build failure, independent of whether the Linux-only `/proc` resource assertion is enabled.

The crate manifest does not target-gate this integration test. The project currently treats x86_64 Linux as the first-RC target, so this finding does **not** claim Windows RC support or a production runtime failure. It is a cross-platform test/build-surface defect introduced by the H-I4-094 proof marker and belongs to the explicit item-4 portability lane.

## Closure contract

Use the smallest test-only/cfg repair. One acceptable shape is simply to make `ReadyProof` itself target-neutral (remove the `#[cfg(unix)]` from the zero-sized marker), because the marker contains no Unix API and `ReadyServer` already carries it unconditionally. An equivalent consistently-cfg'd field/initializer shape is acceptable if it does not weaken the Linux causal proof.

The repair must preserve all accepted H-I4-090..094 invariants:

1. Linux baseline measurement still consumes a readiness-derived proof inside the same `gated_resource_snapshot(...)` call that invokes `process_resource_snapshot(pid)`.
2. The pre-churn `!churn_started` check remains coupled to the baseline measurement.
3. `BarrierProof` remains obtainable only from successful bounded malformed classification and is consumed by the Linux post snapshot.
4. Non-Linux Unix remains warning-clean; bounded timeout/disconnect/EOF cleanup, post-`preauth.release(...)` classification ordering, FD/RSS margins and `ATTEMPTS` stay unchanged.
5. No runtime Session/Carrier/ACK/crypto/wire semantics, D019, release flags, signing/SBOM policy or capacity/security numbers change.
6. No fuzz is required unless decoder/parser/crypto framing owners change.

Validation on the final pushed source/test SHA:

- focused `neko-cli` process/integration tests relevant to the helper change;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
- `git diff --check`;
- clean worktree;
- persist exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust version.

A cross-target Windows compile is useful only if the target/toolchain is already available; do not invent or claim Windows execution otherwise. The source-level cfg defect is sufficient to require repair.

After closure/provenance, immediately resume the exact-current I4-PORT-RES causal re-challenge, then continue the existing deep rolling queue.

## Evidence boundary

This pass is exact-current source/diff review. It does not claim reviewer-local Rust/full-gate execution, Windows/macOS/BSD execution, fuzz, WAN, benchmark or performance evidence. `READY_LIVE` remains `none`; release items 3 and 4 remain incomplete and all release/governance flags remain unchanged.
