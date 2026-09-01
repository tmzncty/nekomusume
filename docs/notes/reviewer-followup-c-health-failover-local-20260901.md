# Reviewer Follow-up C — local real-socket carrier-health failover

- Parent: `b191dd8181e3f6023eb4c1c43c43e5fd1ff0518c`
- Scope: deterministic loopback/process evidence only; not natural-WAN or production evidence.
- Fault source: explicit off-by-default `--cease-udp-replies-after 1` server test seam; TCP remains available. No firewall, route, or qdisc mutation.
- Health contract: existing `HealthLimits` (`degrade_after=2`, `fail_after=4`, `recover_after=2`) and `CarrierHealthEvidence`; only `HealthState::Failed` can drive the `FailoverController` adapter.
- Path: UDP negotiation + Noise authentication; authenticated UDP DeliveryAck for record 1; record 2 becomes uncertain; server ceases UDP replies; four bounded timeout samples produce `Unknown -> Degraded -> Failed`; exact failed threshold selects TCP; fresh TCP negotiation + Noise resume and `ResumeGuard`; uncertain records are resent; authenticated TCP DeliveryAcks complete all three records exactly once in the receiver Session runtime.
- Adversarial/bounds: malformed post-cessation datagram is ignored and cannot switch; below-threshold and recovery/hysteresis unit tests do not switch; cessation point rejects `0` and `>= count`; all receive timeouts and sample counts are finite.
- Controlled-stop remains a separate default mode and retains `controlled_udp_stop=true`; automatic health evidence reports `failover_mode=automatic_health_failure controlled_udp_stop=false` and never emits a controlled-stop event.

## Bounded self-owned VPS row

PASS on real cross-host IPv4 UDP and TCP sockets in experiment `followup-c-20260901-r2` using release-binary SHA-256 `8d4ac0826bd67ce31b23b4716a8e22004914f29199dff6068f7314429dd49b53`.

- parameters: count 3, payload 37 B/record, 111 application bytes, UDP 40098, TCP 40099, concurrency 1, maximum 10 s;
- observed: one authenticated UDP DeliveryAck, explicit server reply cessation, four health timeout samples, exact `fail_after=4` transition for `authenticated_delivery_ack_timeout`, fresh negotiated/Noise-authenticated TCP resume with ResumeGuard, two authenticated TCP DeliveryAcks, receiver final state exactly 3 records / 111 bytes;
- classification: controlled application-level degradation on self-owned endpoints driving the real health threshold; not natural-WAN loss behavior, performance, capacity, production readiness, or public reachability;
- mutations: no firewall, route, qdisc, namespace, capture, or production-service changes;
- cleanup: disposable remote binary, identities, logs and temp directory removed; no listener remained on 40098/40099.

## Re-port rerun on `ecb8729` (superseding runtime observation)

The prior PASS above remains historical evidence for `96d72e5`; it is not evidence for the re-port. After re-porting onto exact authoritative parent `ecb8729a01761cb62ee889fa17e6c50790006d4f`, the smallest self-owned cross-host automatic-health row did **not** reach the degradation seam.

- experiment: `automatic-health-ecb8729-20260901-r5`, `2026-09-01T05:12:41Z`–`05:12:53Z`;
- release binary SHA-256: `aed4e92c84525953b55818f156077acea391116c45c91cfd404d6ebd606de51b`, matched on both hosts;
- parameters: one session, count 3, payload 37 B/record, declared 111 application bytes, UDP 40098, TCP 40099, maximum 10 s, existing private IPv4 tunnel between self-owned hosts;
- result: negotiation and Noise authentication completed; the server observed a duplicate Noise response retry, received record 1 and sent its authenticated DeliveryAck, while the client consumed a non-application datagram at the DeliveryAck boundary and failed closed with `unauthenticated UDP delivery acknowledgement`; therefore no health transition, TCP resume, or delivery claim is made for this row;
- resources (direct client child): exit 2, 0.000000 user / 0.004464 system CPU seconds, 10,004 KiB max RSS, peak 4 FDs, 8 samples at 10 ms, sampler cleanup complete; socket count was unavailable as a useful client-port measure because the supplied ports were remote service ports;
- mutations: no firewall, route, qdisc, namespace, capture, production service, or global flag change;
- cleanup: temporary remote binary/identity/log directory removed; no experiment listener or process remained on 40098/40099; disposable local identities and logs removed after this bounded summary was recorded.

A public-address diagnostic failed earlier at UDP handshake and was not promoted as evidence. A separate tunnel diagnostic without the malformed-datagram seam reproduced the same stale post-handshake datagram boundary, so the blocker is not classified as natural-WAN degradation. Local deterministic automatic-health, malformed/unadmitted-signal, hysteresis/recovery, controlled-stop separation, overall UDP deadline, ResumeGuard, authenticated DeliveryAck and exactly-once tests remain green. This rerun is a truthful negative runtime observation, not natural-WAN blackhole or production failover evidence.
