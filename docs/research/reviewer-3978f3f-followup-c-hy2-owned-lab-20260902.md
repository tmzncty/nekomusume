# Reviewer 3978f3f Follow-up C — owned-lab HY2 paired attempt

## Identity and dependency state

- Reviewed implementation parent: `df61091d379aa10ad001e24f04e2143e13c0cb08`.
- Authoritative coordination descendant read-only at execution start: `3978f3fdd3fb34510468a2e1708c0b2c5c5f6aec`.
- Primary A `0255551` and Follow-up B `e311ce1` were reviewed as truthful negative evidence with cleanup complete. Their blockers do not make this independent Follow-up C unready.
- Source and second endpoint identities were mechanically verified through the established non-interactive `tmzn-hk` owned path. Addresses and credentials are deliberately omitted.
- Pinned Hysteria artifact was reused read-only from `/usr/local/bin/hysteria`: v2.9.3, commit `2d973f9513ef661d1922d6d14acb37945caef47d`, SHA-256 `66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1`.

The production Hysteria process remained running. `/etc/hysteria` was not read, copied, stopped, reconfigured or reused. No route, firewall, network policy or system service was changed.

## Implemented owned-lab seam

`scripts/bench/compare-hy2-owned-lab.sh` is separate from the intentionally loopback-only `compare-hy2.sh`. It fails closed unless the caller supplies an exact SSH alias, endpoint ID, SHA-256 of the resolved endpoint, matching remote address, executable identities and explicit `NEKO_OWNED_LAB=yes` consent. It bounds runs to 3–10, payloads to 1–1200 bytes, timeout to 30 seconds and four distinct repository-approved high ports.

The adapter generates disposable certificate, authentication secret, identities, payload and configs beneath unique `/tmp` paths. It uses a remote loopback TCP echo target, Hysteria `tcpForwarding`, and the Nekomusume authenticated Noise TCP echo with one identical external payload. Samples are interleaved, retain failures, record CPU/user/system/RSS/FD/application bytes, and keep `wire_bytes=null`. Traps target only the unique experiment path and verify owned ports before result emission. The benchmark schema now admits bounded `contract` and `resources` evidence fields.

## Exact execution blocker

No paired sample was produced. After fixing two setup defects before any sample (public-address bind on a NAT endpoint, then SSH background jobs retaining the channel), the changed third attempt proved:

1. the disposable HY2 v2.9.3 server reached `server up and running` on temporary UDP port 40098;
2. the same pinned client failed its initial QUIC connection after five seconds with `connect error: timeout: no recent network activity`;
3. no forwarding listener became ready, therefore neither a HY2 nor a Nekomusume timed sample was admitted;
4. cleanup removed the unique local/remote paths and temporary processes/listeners.

This is a fair-semantics environment blocker: the established source-to-owned-endpoint path does not carry the new temporary HY2 UDP listener. Standing authorization forbids no retry, but the task explicitly forbids production/network-policy changes and unchanged failed experiments. Changing firewall/provider policy would cross this execution boundary, so Follow-up C stops with the exact blocker preserved. There are no raw comparative samples, medians, P95 values, failure-rate comparison or superiority claim.

`wire_bytes` remains null because no trustworthy bounded capture was taken. Release-candidate, production-readiness, release and global-freeze flags are unchanged.

## Verification

The full local gate passed after the implementation: format, locked workspace check, all-target tests, clippy with warnings denied, `scripts/check.sh`, bounded fuzz smoke and `git diff --check`. `compare-hy2-owned-lab-test.sh` verifies the pinned artifact/endpoint/port safety guards and forbids production Hysteria config/service operations.
