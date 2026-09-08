# Distinct-version package rehearsal — exact `91a735c` → `dc90c5f` → `91a735c`

## Scope

One bounded self-owned VPS package rehearsal using two genuinely distinct packages built by each exact tree's existing repository tooling. Endpoint identity is opaque (`self-owned-vps-A` / `private-bind-A`). This is operator evidence only, not state-format compatibility, service-manager hardening, public reachability, security approval, RC, release, or production authorization.

- Experiment ID: `package-distinct-91a735c-dc90c5f-20260909`
- Old A tree: `91a735c0252e7df5da611da4a70e71a60dbdd44d`
- Current B tree: `dc90c5f265a93f113560ba3c97ab436638bdc307`
- Experimental TCP port: `40082`
- Workload: one authenticated 32-byte exchange at each A/B/A stage
- Fresh temporary client/server identities and an external state marker were used; no repository or production identity/state was used.

## Package identity

| Package | Archive SHA-256 | Binary SHA-256 |
|---|---|---|
| A (`91a735c`) | `9677bf075c008aa164fa585d3a13feb234ad1dddac4a261ce86c473567a07027` | `d6c9fdd12e3d719b58c260dbbe49d4da685ca5b8fbd3bc4455ac06a83a45847e` |
| B (`dc90c5f`) | `57e8807fc50dd659ffc27d674e884af7b3a712f91a64918e81bfe7a52325a18e` | `6283a25a93b2fabd0f8cb06008ec72225cf52e29950811aec2b6a8dbf5da80de` |

Both archives passed their exact tree's `scripts/release/smoke-package.sh`. Installed binary hashes matched their package provenance and were different from one another.

## Result

The immutable release-directory sequence completed. The five-line sanitized phase anchor is retained verbatim at [`package-operator-distinct-version-91a735c-dc90c5f-20260909.result.jsonl`](package-operator-distinct-version-91a735c-dc90c5f-20260909.result.jsonl), SHA-256 `4e54f3205cf295ec82e9186eb6b6271558e6babddae819c5ef34e1595a0ddb09`, with exit status `0` and five records:

```json
{"phase":"installed_distinct_hashes","status":"ok"}
{"phase":"A1-authenticated-clean","status":"ok"}
{"phase":"B-authenticated-clean","status":"ok"}
{"phase":"A2-authenticated-clean","status":"ok"}
{"phase":"final_A_state_retained","status":"ok"}
```

At A1, B, and A2, the selected package completed one authenticated TCP 32-byte exchange and left no experimental listener/process. The final `current` link resolved to A. The external state marker content survived all three stages and retained mode `0600`.

This demonstrates package-directory switch/rollback and reuse of unchanged external identity/state for this bounded probe contract. It does not establish arbitrary application-state schema migration compatibility or a production upgrade policy.

## Cleanup and boundaries

The final experiment process/listener check was empty; the dedicated remote releases/state/identity/log path was removed and verified absent. Local temporary worktree, packages, identities and logs were removed. No production Hysteria, service, route, firewall, DNS, proxy, tunnel, qdisc, production identity, or production data was modified. HY2, repeated-failover and package-lifecycle experiments were not rerun.
