# Independent bounded disabled-gate enforcement review — exact `757e0b6`

Bounded independent enforcement verification of the disabled/Experimental-Track gates — 0-RTT early application data and concurrent heterogeneous (UDP+TCP) multipath striping — against the executable/config/CLI surfaces in `crates/neko-cli/src/main.rs`, `crates/neko-carrier/src/lib.rs`, `crates/neko-crypto/src/lib.rs`, at reachable exact `757e0b6c056f0bac39c1fb00e24d54cab55f3f72`. This is enforcement verification only — it does not propose implementing the disabled features. Not a WAN claim, not an independent security/release approval.

## Challenged invariants and result

- **No 0-RTT / early application data path is reachable — holds.** Data protection exists only in `SecureSession::seal`/`open`, which requires a completed Noise IK handshake (`into_stateless_transport_mode`); there is no early-data or 0-RTT record type, codec kind, or CLI flag. Handshake `receive_first`/`finish` precede any application `seal`.
- **No config/CLI path silently enables concurrent UDP+TCP application striping — holds.** `CarrierManager.active` is a single `Option<PathId>`; `choose`/`migrate_back_to_udp`/`promote_*` serialize through it. There is no aggregation/striping command, flag, or state that lets two carriers carry application data simultaneously.
- **Warm/standby readiness/control traffic is not misclassified as application striping — holds.** `prepare_warm_candidate` installs a standby that carries only D064 readiness/control traffic and "cannot carry application data"; promotion requires the full path/generation/session/epoch tuple plus `ready_observations == 3`. Standby control traffic is not application striping.
- **Disabled-gate docs/status match capability output — holds.** `capabilities` marks `health-observe`/`failover`/`multistream`/`endpoint-rebind-*` as `"maturity":"experimental"` and lists them as opt-in runner commands — none auto-enables a disabled gate. The maturity labels match `status.md`'s frozen/blocked declarations.

## Evidence

- Static review on exact `757e0b6`; no executable 0-RTT or concurrent-striping path exists in the current source.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged. The disabled features remain disabled; this review proposes none.
