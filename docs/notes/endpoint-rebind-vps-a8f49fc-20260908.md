# Bounded authenticated UDP source-port rebinding observation — exact `a8f49fc`

## Scope

One fresh bounded self-owned IPv4 observation after the EPREB-006/007 authenticated promotion-barrier repair. This proves one same-Session source-port change under a fixed authenticated crypto record context. It is not evidence of general NAT traversal, public-IP migration, roaming reliability, public reachability, natural path recovery, performance superiority, or production readiness.

## Exact identity and bounds

- repository commit: `a8f49fcbffa7c0e13063db8772470d7b41f3e55d`
- GitHub Actions: Rust CI run `34199496127`, completed successfully
- release binary SHA-256: `3fef6064a47726a509dce6c3760820df45cfb9d50042be6c310beca5fb2b848b`
- target: administrator-owned IPv4 VPS; raw endpoint intentionally omitted
- one Session, two 16-byte application records, one temporary UDP listener, maximum duration 10 seconds
- endpoint relation: `source_endpoint_changed=true`; raw A/B endpoints are not retained
- carrier/source-binding generation: 0 -> 1; crypto `RecordContext.path_generation` remained the existing fixed context and is not claimed to have migrated
- start: `2026-09-08T07:50:44Z`
- end: `2026-09-08T07:50:46Z`
- no route, firewall, DNS, proxy, tunnel, qdisc, namespace, capture, or production-service change

## Sanitized ordered result

Client and server both exited 0. The retained sequence was:

1. endpoint A authenticated and confirmed the generation-0 application record;
2. endpoint B sent the ordinary authenticated candidate/readiness request;
3. server observed exact B and sent an independent fresh server-owned challenge;
4. B returned the authenticated exact-tuple challenge response;
5. server validated and atomically promoted B;
6. server sent the authenticated post-promotion synchronization response;
7. client authenticated `endpoint_promotion_sync_received` before emitting B application Data;
8. B received the post-promotion Session `DeliveryAck`;
9. both summaries reported 2 records and 32 application bytes.

Relevant secret-safe events:

- `endpoint_a_confirmed`
- `endpoint_candidate_seen`
- `endpoint_challenge_sent`
- `endpoint_validated`
- `endpoint_promoted`
- `endpoint_promotion_sync_sent`
- `endpoint_promotion_sync_received`
- `endpoint_post_delivery_ack_sent` / `endpoint_post_delivery_ack_validated`

Bounded informational resources:

- client elapsed `0:00.13`, maximum RSS `2304 kbytes`;
- VPS server elapsed `0:01.05`, maximum RSS `2560 kbytes`;
- both exit status 0.

## Cleanup

The temporary VPS binary, identity, logs, launcher state, and experiment directory were removed. An independent post-run check returned `cleanup_verified=true`, with no listener on the experiment UDP port and no experiment process. No private key, credential, raw address, payload, or raw diagnostic entered the repository.

## Historical relation

The earlier exact-`c81fa73` observation remains superseded candidate provenance only because it lacked the authenticated post-promotion synchronization barrier. This fresh exact-`a8f49fc` run does not reuse or upgrade that earlier conclusion.