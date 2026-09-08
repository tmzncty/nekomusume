# Bounded authenticated UDP source-port rebinding observation — exact `c81fa73`

## Scope

One bounded self-owned IPv4 observation of the opt-in `endpoint-rebind-server` / `endpoint-rebind-client` runtime. This is source-port rebinding on the same logical Session, not a claim about arbitrary NAT traversal, roaming reliability, public reachability, natural loss/recovery, or production readiness.

## Exact identity and bounds

- repository commit: `c81fa736bf6d551aab8405a5c32c104772496bf8`
- binary: `neko-cli` release build, SHA-256 `b4471008b6a6a9d67980fffaf6c8ff92ec4b5bc9a95a16c57048dfed49db36c5`
- target: administrator-owned IPv4 VPS; raw address intentionally omitted from this artifact
- one Session, two records, 16-byte payloads, one UDP listener, maximum duration 10 seconds
- endpoint class: `source_port_changed=true`; old/new raw endpoints are not retained
- no route, firewall, DNS, proxy, tunnel, qdisc, namespace, or production-service change
- start: `2026-09-08T06:59:45Z`
- end: `2026-09-08T06:59:47Z`

## Sanitized result

Client and server both exited with status 0. The event order was:

1. endpoint A authenticated and confirmed generation 0 application record;
2. endpoint B sent an authenticated candidate with generation 1;
3. server observed the candidate and sent a fresh server-generated challenge to exact B;
4. B returned the authenticated challenge response;
5. server validated and atomically promoted B to generation 1;
6. post-promotion application DeliveryAck succeeded on B;
7. client and server summaries reported 2 records / 32 application bytes.

The retained secret-safe event markers were:

- `endpoint_a_confirmed`
- `endpoint_candidate_seen`
- `endpoint_challenge_sent`
- `endpoint_validated`
- `endpoint_promoted`
- `endpoint_post_delivery_ack_validated` / `endpoint_post_delivery_ack_sent`
- `source_endpoint_changed=true`

Resource observations from `/usr/bin/time -v` were bounded and informational only:

- client elapsed `0:00.19`, maximum RSS `1920 kbytes`;
- VPS server elapsed `0:00.74`, maximum RSS `2688 kbytes`;
- both reported exit status 0.

## Cleanup

The temporary VPS binary, identities, logs, launcher state, and experiment directory were removed. An independent post-run check confirmed `cleanup_verified=true`, no listener remained on the experiment UDP port, and no experiment process remained. No private key, raw endpoint, credential, payload, or raw diagnostic was copied into the repository.

## Claim boundary

This is one bounded authenticated same-Session source-port change observation. It does not establish NAT traversal generally, public-IP migration, roaming reliability, a rate, performance superiority, natural network recovery, or production/release readiness. Release, production, freeze, and released flags remain unchanged.