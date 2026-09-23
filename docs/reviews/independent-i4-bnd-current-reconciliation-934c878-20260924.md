# Independent bounded review — current algorithmic/resource boundedness reconciliation at `934c878`

## Classification

**NO NEW FINDING in the dependency-ready non-policy boundedness surface.** The known `SessionRuntime.events` retained-history cap remains a policy/value gate and is not repaired by inventing a number.

Reachable review anchor: `934c878164c3d417d5045672aeabef07d4235db2` (`main` at review start).

## Reuse / owner-diff basis

The last dedicated boundedness reconciliation is `docs/reviews/independent-i4-bnd-reconciliation-5b7b22b-20260922.md`, exact source/test anchor `5b7b22b34e3e52d4bdde7bcc5a16bcb81f9342e2`, with developer-local exact-tree provenance in `docs/notes/i4-bnd-provenance-5b7b22b-20260922.md`.

A repository compare from `5b7b22b` to this review anchor shows that the core runtime owners challenged there did **not** change in the interval:

- no changes to `crates/neko-session/src/lib.rs`;
- no changes to `crates/neko-carrier/src/lib.rs`;
- no changes to `crates/neko-reliable/src/lib.rs`.

The substantive code/test changes in that interval are confined to `crates/neko-cli/tests/probe.rs` and `crates/neko-cli/tests/multistream.rs`, plus review/provenance documentation. Those CLI test changes are H-I4-097..115 process/socket/thread ownership hardening and have their own exact-current independent ownership sweep in `docs/reviews/independent-cli-process-ownership-fcd7a08-20260924.md`.

Therefore it would be misleading and wasteful to manufacture a new capacity benchmark or re-select numeric policy merely because the repository HEAD advanced.

## Current challenge

### Runtime state caps

Exact-current `SessionRuntime` continues to enforce the established hard ceilings for runtime streams, aggregate queued records, queue bytes, total bytes, and record bytes. The underlying queue/counter owners are unchanged from the reviewed source anchor. Current `neko-carrier` and `neko-reliable` algorithmic/resource owners are likewise unchanged since the dedicated boundedness reconciliation.

No new dependency-ready defect can be inferred from those unchanged owners without contradicting the prior exact-tree review/provenance.

### CLI test-harness growth and waiting

The post-`5b7b22b` CLI changes reduce, rather than widen, the relevant unboundedness surface:

- real network children are migrated from synchronous/unbounded `.output()`/wait patterns to bounded owner helpers;
- peer accept/read/recv operations are causally deadline-bounded;
- terminate/reap errors are not treated as exit proof;
- stdout/stderr drains are concurrent;
- after H-I4-115, descendant-held inherited pipe writers cannot strand the caller indefinitely because reader completion itself has a finite bound.

The current process-ownership sweep found no remaining concrete test-owned process/socket/thread hang owner. No new queue/list/history growth structure was introduced by these harness repairs that requires a repository capacity value.

### Known policy gate preserved

`SessionRuntime.events: Vec<RuntimeEvent>` intentionally retains lifetime event history. Its maximum retained-history size is not selected by existing committed semantics. Prior item-4 review classified this as `POLICY_BLOCKED_RESOURCE_BOUND` requiring a maintainer/security history/capacity decision.

This review does not choose TTL/LRU/history-size/capacity/security numbers and does not reinterpret D019 or any release policy. The existence of that policy gate does not block independent local review of other surfaces.

## Evidence boundary

- This note is source/owner-diff reconciliation only; it is **not** a new developer-local or reviewer-local execution record.
- No `scripts/check.sh`, Rust test, Clippy, capacity-pressure benchmark, fuzz, VPS/WAN or performance run was executed by this reviewer for this note.
- Historical exact-tree command results remain attached to their own SHAs; they are not promoted to `934c878` execution evidence.
- No decoder/parser/crypto framing owner changed in this lane; no fuzz was required.
- No new capacity/security values, D019 decision, release/freeze/production authority or core architecture decision was made.

## Item-4 consequence / next lane

Algorithmic/resource boundedness is reconciled for the current tree as: **no new dependency-ready defect; one existing policy-blocked retained-history cap remains explicitly open.**

Proceed to release packet / item-4 factual reconciliation after the completed H-I4-097..115 process-ownership group and the two 2026-09-24 independent no-finding reconciliations. Update evidence indexes only from reachable anchors; do not convert source review into local execution evidence and do not change release/governance flags.
