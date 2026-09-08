# Bounded scripted migration-back observation — exact `5d6582c`

This note retains one bounded self-owned IPv4 observation of the experimental
`--migration-back` path. It is evidence for one scripted application-level
fallback/recovery/return sequence only. It does **not** establish natural path
recovery, a reliability rate, public reachability, comparative performance,
release candidacy, or production readiness.

## Exact identity and bounds

- implementation commit: `5d6582c0a674306cf47002d4cd26d6e0c6065c22`;
- release binary SHA-256 on both controlled endpoints:
  `2ca390b4cd5b004e914102e1d38d005b9fc8b83df02ad6e4415cf7f31b7867df`;
- binary size: 1,226,872 bytes;
- experiment id: `migration-back-final` (client/server role suffixes);
- window: 2026-09-08 12:38:53–12:38:58 UTC+8;
- one client Session, IPv4, temporary unprivileged TCP/UDP ports 40080/40081;
- three 16-byte application records, 48 application bytes total, 10-second
  command bound;
- fault/recovery shape: off-by-default scripted UDP application-reply cessation,
  authenticated TCP warm fallback, then one-shot authenticated UDP recovery;
- the server used a bounded 10 ms test-only TCP DeliveryAck delay so the existing
  score-margin gate could be crossed by measured observations. This is part of
  the scripted experiment and is not a network-performance comparison.

No private endpoint, credential, identity material, payload content, or raw
private diagnostic is retained here.

## Observed sequence

The client and server completed the following ordered boundaries:

1. UDP path 1 / generation 0 authenticated and confirmed the first record.
2. Scripted application replies ceased; one assigned middle record became
   uncertain and three bounded authenticated-DeliveryAck timeout observations
   drove the existing failure threshold.
3. TCP path 2 / generation 1 completed authenticated warm readiness, became the
   sole active owner, replayed the middle record, and validated its DeliveryAck.
4. The active TCP authenticated application RTT sample was 51,756 microseconds
   (`loss_per_mille=0`, `pto=0`).
5. One exact-peer, exact-tuple authenticated UDP generation-1 recovery challenge
   completed with a measured RTT of 20,127 microseconds
   (`loss_per_mille=0`, `pto=0`). Validation and health remained separate
   inputs.
6. The first migration attempt retained TCP ownership at the hold gate. The next
   eligible manager decision atomically moved ownership from TCP path 2 to UDP
   path 1 at generation 1.
7. Only after that decision, the previously unassigned final record was sent on
   UDP and its authenticated DeliveryAck was validated.

The final client accounting was:

```text
confirmed:   3 records / 48 bytes
uncertain:   1 record  / 16 bytes
replayed:    1 record  / 16 bytes
duplicate:   0 records / 0 bytes
lost:        0 records / 0 bytes
conflicting: 0 records / 0 bytes
```

The client exited 0. The server reached its required recovery and post-return
terminal milestones and reported exactly 3 records / 48 application bytes.

## Local and negative gates

Before this observation, exact `5d6582c` passed the full repository gate,
strict clippy, `git diff --check`, all 44 `neko-carrier` tests, a positive real
loopback fallback/recovery/return proof, and a real server/client tampered
recovery negative. The negative reached authenticated TCP resumed delivery,
attempted the recovery challenge, retained TCP ownership, emitted no migration
or post-return DeliveryAck success, and allowed no server success summary.

## Cleanup and governance boundary

The disposable remote binary, identity and log were deleted. Post-run checks
found no experimental listener or process. No firewall, route, DNS, proxy,
tunnel, qdisc, namespace, production service, or release flag was changed.

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and
`RELEASED=false` remain unchanged. C2 source-retention policy remains a separate
release/security limitation.