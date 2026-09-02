# Reviewer 25e0daa Primary A — exact-head D064 warm controlled-fault evidence

## Scope and identity

- exact authoritative GitHub `main` parent, verified by fetched `origin/main` and independent `git ls-remote`: `25e0daa4e74b3239568067afa967412ec4c0ebc7`
- tree: `3374b6e7017f28a7f580c98784e5aef2c798aedb`
- fresh detached worktree: `reviewer-25e0daa-primary-a`
- release binary SHA-256, verified identically on build host and both self-owned endpoints: `8ede1564015561498559586a83c6aeea2a75171bd2ae45b93e547e1793082852`
- binary: 1,141,224-byte Linux x86-64 ELF; rustc 1.98.0; Cargo 1.98.0; target `x86_64-unknown-linux-gnu`
- parameters: established self-owned client↔VPS path, concurrency 1, fresh ports 40085/udp and 40086/tcp, 3 records × 16 bytes, client maximum 12 s, server maximum 15 s, application-level UDP reply cessation after the first reply
- classification: bounded controlled cross-host application experiment; not natural-WAN/public reachability, production, security, capacity, performance, or superiority evidence

The execution read `AGENTS.md`, standing authorization, current handoff Primary A/F/completion text, the current D064 runtime/tests/spec, retained negative evidence, and immutable prior evidence commit `3ec793050a3b4e2b519b005684b53ad46868ff69`.

## Warm result

The single diagnostic warm run passed end to end. Raw client/server chronology and process-resource rows are retained in `artifacts/reviewer-25e0daa-primary-a/`.

1. Canonical UDP negotiation and Noise authentication completed at generation 0; record 1 received an authenticated logical DeliveryAck.
2. Record 2 became uncertain after controlled application-level UDP reply cessation (`reason=bounded_test_seam`).
3. TCP negotiation, Noise authentication and resume validation completed at generation 1 while UDP remained sole active.
4. Three sequential authenticated readiness responses were admitted. Client request/response timestamps (microseconds from process origin) were 1,419,318/1,901,513; 1,901,541/2,431,182; and 2,431,204/2,962,996. Latencies were 482,195 / 529,641 / 531,792 us; the 1,543,678 us sequence was below 1 s per probe and 3 s overall.
5. `tcp_warm` explicitly reported `application_data=0`, proving no TCP application data before atomic promotion.
6. Three one-second authenticated-delivery-ACK failure observations advanced UDP health unknown → degraded → failed. The manager reason was `udp_path_degraded`, threshold 3, generation 1.
7. Atomic promotion used `warm_authenticated_resume`: failure decision 6,083,065 us, new-active 6,083,108 us, first resumed data accepted 6,517,353 us, recovery 434,287 us.
8. Both uncertain records were replayed over TCP and received authenticated DeliveryAck; 3/3 records and exact 48 application bytes completed in order. Observable accounting is confirmed=3 records/48 bytes, uncertain=2/32, replayed=2/32, duplicate=0, lost=0. Delivery epoch 1 and path generation 1 were recorded.

Client/server exited 0. Client direct-child resources: user/system CPU 0/0.006959 s, max RSS 10,032 KiB, peak FD 5. Server: user/system CPU 0.00138/0.003105 s, max RSS 10,096 KiB, peak FD 6, peak owned sockets 2. The sampler does not claim client socket count because no owned local port was supplied.

## Batch decision, cleanup and claim boundary

No new 5 warm + 5 cold batch was run. The immutable `3ec7930` evidence already contains a complete valid interleaved 5+5 set from the identical release-binary hash and repaired D064 implementation lineage; repeating it would add little evidence and consume the bounded lab window. This commit retains only the requested exact-`25e0daa` warm row.

After capture, both endpoints reported zero experiment listeners and zero experiment `neko-cli`/resource-sampler processes; temporary identities, binaries, samplers and runtime files were removed. No firewall, route, qdisc, NAT, tunnel, provider, service, production configuration, handoff, governance flag, frozen corpus or persistent secret changed. No push was performed.
