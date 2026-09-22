# H-I4-107 — multistream bounded-owner repair remains incomplete

**Severity:** HIGH — release/item-4 test-harness correctness

**Reviewed anchor:** `0dfed87fc96f1427e333f8d42dde34f96a31ae2a`

**Developer changes reviewed first:**

- `a5e80bb85948419e20a01430ee8381bf475fa3bd` — converts the three H-I4-106 **server-side** `wait_with_output()` owners to a local `bounded_wait_with_output` helper and records exact-tree developer-local provenance in `docs/notes/h-i4-106-provenance-a5e80bb-20260922.md`.
- `f3093d5a6c5ec063d29f08b0eada0a4afee6a538` — adds a deterministic no-client timeout regression for that helper.

The server-side direct `wait_with_output()` calls are repaired on the ordinary `try_wait -> Ok(None) -> deadline -> kill succeeds -> reap succeeds` path. However, the exact H-I4-106 closure contract is **not fully satisfied**: the two real networked client processes are still run with synchronous unbounded `.output()`, and the new helper itself has unresolved ownership on error branches.

No exact-`a5e80bb` hosted workflow/status was visible in this reviewer pass. The persisted gate is developer-local provenance, not reviewer-local execution or hosted CI.

## Challenged invariant

A built-binary/process integration test must have an independent harness-local completion bound for each networked child whose termination is part of the oracle. A bounded process-owner helper must also remain **ownership-truthful on every result branch**. In particular:

1. product behavior under test cannot be the only termination proof for a synchronous `.output()` call;
2. `try_wait()` error does not prove child exit;
3. `kill()` error does not prove child exit;
4. a later `try_wait()` error does not prove reap;
5. stdout/stderr reader threads may be joined only after child exit / pipe closure is proven, or their own completion is otherwise independently bounded.

A failure branch may report that bounded cleanup itself failed, but it must not silently classify unresolved child/pipe ownership as converged.

## Concrete counterexamples

### 1. H-I4-106 client owners remain unbounded

Exact-current `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` still starts the real multistream client with:

```rust
Command::new(bin)
    .args([... "--mode", "client", ...])
    .output()
    .unwrap();
```

The test reaches `bounded_wait_with_output(&mut server, ...)` only **after** that client returns. If a protocol/lifecycle regression leaves the client blocked on connect/handshake/framed I/O, the test never reaches the new bounded server helper.

`unauthorized_client_is_rejected_by_allowlist` retains the same synchronous networked client `.output()` shape. Its expected authorization failure is product behavior under test, not an independent harness-level proof that the process must terminate.

This was explicitly inside H-I4-106's closure contract: the handoff required a captured/bounded owner for both sides of the two real server/client tests. Therefore H-I4-106 must not be treated as repository-closed until these two client owners also have independent bounds.

### 2. Initial `try_wait()` error in `bounded_wait_with_output`

The helper starts stdout/stderr `read_to_end` threads before observing child exit and then has:

```rust
Err(e) => panic!("try_wait failed: {e}"),
```

This panics immediately without attempting bounded terminate/reap. At that point:

- child exit is unproven;
- the child handle is abandoned by the unwinding test path;
- both reader threads may still be blocked in `read_to_end()` on live stdout/stderr pipes;
- their join handles are dropped before any exit/pipe-closure proof.

This is the same ownership class that H-I4-101/H-I4-102 required other process helpers to fail closed around; the new multistream helper reintroduces it locally.

### 3. Deadline cleanup treats `try_wait()` error as if reap were complete

On deadline the helper executes:

```rust
let _ = child.kill();
...
match child.try_wait() {
    Ok(Some(_)) | Err(_) => break,
    ...
}
panic!("child did not exit within bound");
```

Two facts are lost:

- `kill()` failure is ignored;
- post-kill `try_wait()` error is grouped with `Ok(Some(_))` and exits the reap loop even though no exit proof exists.

The helper then panics before joining the drain threads. If termination/reap did not actually converge, those readers may remain blocked on live pipes. The helper is wall-clock bounded in the common timeout case, but its ownership claim is not true on these error branches.

The `f3093d5` negative regression exercises a live no-client server reaching the ordinary deadline with normal kill/reap. It is useful evidence for the original server-side hang, but it does not bound the two client `.output()` owners and does not challenge either helper `try_wait Err` branch or ignored `kill Err`.

## Impact and boundary

A multistream lifecycle/protocol regression can still strand item-4 CI indefinitely in either real client process, while process API/lifecycle errors inside the new helper can fail the test while abandoning unresolved child and pipe-reader ownership. Therefore the current repository statement that H-I4-106 is fully closed is not factually safe.

This is a **test-harness/release-evidence HIGH**. It is not a production Session/Carrier/ACK/crypto/wire semantic finding and creates no new WAN question; `READY_LIVE: none` remains authoritative.

## Closure contract

1. Finish the original H-I4-106 ownership repair: give the networked client processes in `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` and `unauthorized_client_is_rejected_by_allowlist` their own narrow test-local bounded owner. Do not move `.output()` into an unkillable worker thread.
2. Keep the three converted server call sites and normal-path deadline behavior, but make every `bounded_wait_with_output` failure branch ownership-truthful.
3. Initial `try_wait Err` must enter bounded best-effort termination/reap before failing; do not immediately panic with unresolved child ownership.
4. Deadline cleanup must not ignore `kill()` result and must not treat post-kill `try_wait Err` as `Ok(Some)`. Preserve truthful classes: exit proven; bounded cleanup succeeded; bounded cleanup itself failed / exit unproven.
5. Do not block on stdout/stderr drain joins while exit/pipe closure is unproven. If cleanup itself fails and a live child may still own the write ends, fail closed without falsely claiming drain convergence. Prefer the smallest test-local repair; do not build a generic process framework.
6. Preserve stdout/stderr evidence and existing negotiation/crypto/JSON/human-output assertions. If a shared test-local child owner can serve both client and server without widening policy, prefer that over duplicated lifecycle code.
7. Keep the deterministic no-client regression. Add focused coverage for the bounded client owner. For OS-error-only branches, if forcing real `try_wait`/`kill` errors would require unsafe/kernel fault injection, source-level ownership proof is acceptable; do not manufacture brittle platform tricks.
8. No decoder/parser/crypto-framing implementation change is implicated; fuzz is not mechanically required.
9. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
10. After closure continue immediately into the remaining process/socket/thread ownership sweep and cross-platform reconciliation. Do not infer repository-wide queue exhaustion from this repair.

## Reviewer execution / exclusions

This finding comes from exact-current GitHub source/control-flow review. The reviewer did **not** claim reviewer-local Rust/full-gate execution, cross-platform execution, fuzz, WAN/live evidence, or performance evidence in this pass.
