# Reviewer 3978f3f Primary A — exact-tree A0 and changed-path A1 blocker

## Scope and identity

- reviewed implementation: `df61091d379aa10ad001e24f04e2143e13c0cb08`
- reviewed tree: `0b40a1aadc2b530ee11d3344312dacbac28632ae`
- authoritative GitHub `main` verified before work: `3978f3fdd3fb34510468a2e1708c0b2c5c5f6aec`, a direct reviewer/handoff descendant of the implementation
- fresh detached worktree: `reviewer-3978f3f-df61091`
- exact release binary SHA-256 on build host and both self-owned Linux endpoints: `3460fb74856f6eb53db47c4e04f6bb887c77157b85d8d12b6bbe1c1c2c980a52`
- binary size: 1,126,912 bytes; Linux x86-64 ELF; rustc 1.98.0; Cargo 1.98.0; target `x86_64-unknown-linux-gnu`
- classification boundary: bounded self-owned cross-host IPv4 application experiment; not natural-WAN/public reachability, production, capacity, security, or performance evidence

The execution read `AGENTS.md`, standing authorization, the full reviewer handoff Primary A/completion/fallback text, D064, the exact `0df6f48` blocker, and historical valid self-owned cross-host evidence. It did **not** retry the unchanged public-address/no-ingress path preserved by `0df6f48`.

## A0

PASS:

- `cargo test --workspace --all-targets --locked --no-fail-fast`: exit 0
- `git diff --check`: exit 0
- one release build completed from the exact reviewed tree; the same hash was verified after deployment to both endpoints

## A1 path validation and retained blocker

A materially changed path was used: the previously valid second self-owned Linux client host to the self-owned VPS, rather than the client/public-address route in `0df6f48`. A fresh UDP endpoint on port 40099 completed canonical negotiation, Noise authentication, and a 17-byte application echo with client/server exit 0. This established-path preflight was viable.

One warm D064 run was then attempted with fresh ports 40097/40098, count 3, 16 bytes/record, concurrency 1, client max 5 seconds and server max 8 seconds. It failed closed and is retained without an unchanged retry:

1. UDP negotiation/authentication succeeded.
2. The first 16-byte record received an authenticated encrypted `DeliveryAck`.
3. The second range became uncertain after controlled application-level UDP reply cessation.
4. Warm TCP negotiation, Noise authentication and resume validation succeeded.
5. Readiness challenge 1 and 2 returned authenticated `admitted=true` responses.
6. Challenge 3 did not complete: the client reported `readiness response timeout or malformed frame`; the server reported `bad readiness frame`.
7. No `tcp_resource_admitted`, `tcp_warm`, `tcp_resumed`, TCP application data, promotion, replay completion, final ordered bytes, or recovery timing was emitted.

Therefore the requested 5 warm + 5 cold interleaved batch did not run. This is one exact changed-path D064 blocker, not evidence of warm recovery. Raw redacted structured records are preserved in:

- `artifacts/reviewer-3978f3f-primary-a/changed-path-client.jsonl`
- `artifacts/reviewer-3978f3f-primary-a/changed-path-server.jsonl`

## Cleanup and claim boundary

After capture, both endpoints reported zero matching temporary directories, zero `neko-cli` processes, and zero listeners/sockets on ports 40097–40099. No firewall, route, qdisc, tunnel, service, global flags, handoff, frozen corpus, production configuration, or persistent secret was modified. Disposable identity files were removed. No push was performed.
