# Independent bounded review — CLI process/socket/thread ownership at `fcd7a080`

## Classification

**NO FINDING in the reviewed test-harness owners.** This is bounded item-4 review support, not a claim that every production socket lifetime is globally wall-clock bounded.

Reachable review anchor: `fcd7a080b4bd834eccec45c9d9109687800a51cc` (`main` at review start). The relevant CLI test source is unchanged from developer source/test `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d`.

## Inspected owners

- `crates/neko-cli/tests/probe.rs`
- `crates/neko-cli/tests/multistream.rs`
- `crates/neko-cli/src/multistream.rs`
- `docs/CHATGPT_HANDOFF.md`
- `SECURITY.md`
- `docs/status.md`
- `docs/specs/nekomusume-session-v0.md`
- `docs/carrier-architecture.md`

The review challenged the remaining ownership invariant after H-I4-097..115: a test-owned network child, peer socket, reader, or peer thread must not be able to strand the repository gate merely because the behavior under test stops making progress.

## Challenge and result

### Process owners

Exact-current real network client invocations inspected in `probe.rs` are routed through `bounded_client_output` / `bounded_wait_with_output`, including matrix probe, authenticated TCP/UDP clients, benchmark/failover clients, periodic clients and endpoint-rebind clients. The helper concurrently drains stdout/stderr, bounds direct-child exit, treats kill/`try_wait` failure as cleanup failure rather than exit proof, and—after H-I4-115—bounds reader completion independently so an inherited writer held by a descendant cannot cause an unbounded caller wait.

Remaining direct `Command::output()` uses encountered in this sweep are local/fail-fast owners such as key generation, help/capabilities/invalid configuration, socket-free fixtures, or bounded local lab/simulation commands. No exact-current source evidence showed one of those calls acquiring an external network/process completion dependency, so they were not mechanically converted.

`multistream.rs` test server/client process owners likewise use the bounded output helper for real network lifecycle cases. Identity/configuration rejects remain local/fail-fast exclusions.

### Socket and peer-thread owners

The reviewed `probe.rs` peer paths install finite ownership before blocking operations: TCP peer acceptance uses `bounded_accept` or an explicit nonblocking accept loop with deadline; `frame_read_test` calls reached in network peers have a socket read timeout installed before the first `read_exact`; UDP `recv`/`recv_from` paths inspected install read deadlines before the first receive. The corresponding peer joins therefore have a causal finite socket owner rather than relying on the client to behave correctly.

In `multistream.rs` tests, synthetic TCP peers use a bounded/nonblocking accept loop and set a read timeout before negotiation/Noise frame reads. The real product server/client children are outer-harness bounded.

No concrete remaining test-owned unbounded process/socket/thread owner was demonstrated in this bounded sweep.

## Production/lab fixture policy boundary — not a repair finding

`crates/neko-cli/src/multistream.rs` still contains blocking `TcpListener::accept`, `TcpStream::connect`, and authenticated application-level `frame_read`/`read_exact` operations after admission. Exact-current `docs/specs/nekomusume-session-v0.md` explicitly does not specify socket/runtime integration or live failover timing, and the architecture/status documents do not currently select an application idle timeout for this fixture.

Therefore this review does **not** invent an idle timeout, duration, capacity, or security number. The correct current classification is an operational/policy evidence boundary: bounded workload/resource dimensions are implemented, while recursive wall-clock ownership of an authenticated peer lifetime is not established by the outer process tests. Any release/item-4 packet statement must preserve that distinction. A future numeric liveness policy requires an existing committed semantic decision or maintainer/security policy choice; it is not inferred from this review.

## Exclusions / evidence boundary

- No reviewer-local Rust/full repository gate was executed in this review.
- No macOS/BSD/Windows runtime execution was performed; Unix/Linux source reasoning is not Windows evidence.
- No decoder/parser/crypto-framing implementation changed; fuzz was not run.
- No VPS/WAN, benchmark, soak, or performance work was run.
- No D019, retained-history capacity, TTL/LRU/security value, signing/SBOM/key-custody/publication, core Session/Carrier/ACK/crypto/wire, RC/freeze/release/production decision was made.

H-I4-115 remains closed at `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d` with its reachable developer-local exact-tree provenance. `READY_LIVE` remains `none`; release items 3 and 4 remain incomplete.

## Next dependency-ready lane

Proceed to cross-platform CLI/process factual reconciliation: separate Linux execution evidence, portable source reasoning for Unix variants, and Windows claims; reconcile I4-CLI-PROC-096 and H-I4-097..115 without treating Unix inherited-pipe behavior as Windows proof. After that continue algorithmic/resource boundedness reconciliation and the preserved deep item-4 queue.
