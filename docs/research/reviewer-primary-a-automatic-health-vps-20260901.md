# Primary A — truthful automatic UDP degradation and cold TCP recovery

- window (UTC): `2026-09-01T06:53:25Z`–`2026-09-01T06:55:45Z`
- exact authoritative parent: `877e50f7a37144bc070377092056d79324aebb9b`
- tested working tree: this commit's A1–A5 implementation
- release `neko-cli` SHA-256: `7799be9594d5e10f988eae19a982bfb92b11fa2502ac76afb11b32111b8f42d2`
- endpoints/path: two self-owned Linux hosts; addresses, disposable identities and private topology omitted
- bounds: concurrency 1; ports 40080/TCP and 40081/UDP; 3 records; 32 B/record; 96 application bytes; client maximum 6 s; server maximum 8 s

## Deterministic implementation evidence

The first UDP DeliveryAck is admitted by an absolute-deadline loop restricted to the expected peer. It identifies only the exact cached negotiation and Noise responses as harmless old protocol artifacts, admits only an authenticated exact Session/stream/offset/length DeliveryAck, and fails closed after three malformed same-peer datagrams. Deterministic tests cover a delayed duplicate Noise response after handshake completion, wrong-peer traffic and malformed-bound exhaustion. The delayed duplicate does not restart negotiation/Noise, replace ResumeGuard or Session state, advance path generation, or alter delivery state.

Carrier health now has bounded explicit progress/failure observations separate from measured `HealthSample` values. The automatic path records `authenticated_delivery_ack_timeout` events without serializing invented RTT, loss or PTO values. Its explicit D064 profile uses `k_failure=3`: one and two failures do not switch, the third reaches Failed, and authenticated progress resets the failure counter.

`CarrierManager` exclusively creates the UDP-to-TCP decision, increments generation, selects the fallback path and supplies stable reason `udp_path_degraded`. `FailoverController` only applies that decision to its uncertain/replay/dedup bookkeeping. Deterministic tests keep stale/wrong manager inputs atomic and preserve authenticated DeliveryAck, ResumeGuard, exactly-once and controlled-stop separation.

The runner uses real `Instant` observations for health failure, decision, TCP active and first resumed-data send. Recovery latency is emitted only when both decision and first resumed-data timestamps exist. Because TCP is connected after the decision, the evidence class is explicitly `cold`, not warm.

## Self-owned VPS rows

### Preserved negative row — `primary-a-877e50f-vps-r1`

The first bounded attempt used the public-address path. It failed before negotiation: the client emitted 20 bounded UDP hellos and ended `UDP handshake timeout`; the server reached its bounded `failover timeout`. Client exit was 2 and no automatic failover claim was made. Cleanup passed. This was not retried unchanged.

### Accepted repaired row — `primary-a-877e50f-vps-r2`

The next and only repaired row changed the path to the existing self-owned private host path. Both endpoints verified the exact binary SHA-256 above. The server authenticated UDP record 1, then used the explicit bounded reply-cessation seam. Client observations were:

```text
failure 1 at 104956 us: state=unknown
failure 2 at 208986 us: state=degraded
failure 3 at 312970 us: state=failed
manager decision: generation=1 threshold=3 reason=udp_path_degraded
diagnostic cause: authenticated_delivery_ack_timeout
fallback class: cold
decision -> TCP active: 1119155 us
TCP active -> first resumed-data send: 1569765 us
recovery latency: 2688921 us
```

The resumed TCP path passed authenticated ResumeGuard and two exact DeliveryAcks. Server and client exited 0; the receiver reported exactly 3 records / 96 application bytes in order. `controlled_udp_stop=false` remained distinct from the automatic path. No RTT/loss/PTO metric was claimed.

Cleanup passed on both hosts: no listener remained on 40080/40081, no experiment process remained, and temporary binaries, identities and runtime directories were removed. Raw logs, addresses, identities, keys and topology were not committed.

## Verification and boundary

Focused carrier-health/admission/process tests, `cargo fmt --all -- --check`, workspace all-target check/test, workspace all-target Clippy with warnings denied, `scripts/check.sh`, and `git diff --check` passed. No parser or wire behavior changed, so fuzz was not run.

This proves one bounded explicit application-level UDP degradation followed by manager-owned cold TCP recovery on one self-owned path. It does not prove natural WAN loss, warm standby, public/general reachability, performance/capacity, release readiness, production readiness or protocol freeze. The prior exact-code negative evidence remains valid historical evidence and is not overwritten.