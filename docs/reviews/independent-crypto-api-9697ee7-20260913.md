# Independent bounded crypto integration/API review — exact `9697ee7`

Bounded independent implementation/API review of `crates/neko-crypto/src/lib.rs` — `noise_ik_params`, `NonceManager`, `ReplayWindow`, `LocalIdentity`, `TrustPolicy`/`TrustRecord`, `RecordContext`, `prologue`/`bound_prologue`, `InitiatorHandshake`/`ResponderHandshake`, `SecureSession` (`seal`/`open`/`update_key_phase`), at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. This is an API/integration-boundary challenge, **not** cipher/Noise cryptanalysis and not dependency-suitability approval; D019/persistent-replay policy and primitive selection are out of scope.

## Challenged invariants and result

- **Noise pattern/prologue/transcript binding — holds.** `noise_ik_params` admits only `Noise_IK_25519_ChaChaPoly_SHA256` (`:109`); `prologue`/`bound_prologue` inject `PROLOGUE_PREFIX` + application domain + optional negotiation/transcript binding into the Noise handshake (`:263-277`, `:302-316`, `:431-467`). `authenticated_binding_matches_and_one_bit_mismatch_fails_before_session` proves a one-bit binding mismatch fails the handshake.
- **Trust/authz before protected data — holds.** `TrustPolicy::authorize` requires `version == 1 && Active && exact public_key && exact scope` (`:229-240`); it runs inside `ResponderHandshake`/`receive_first` before any transport session is produced.
- **Direction/context/epoch/key-phase separation — holds.** `RecordContext{delivery_epoch,key_phase,path_generation,stream_id,direction}` is `encode()`d *inside* the AEAD ciphertext (`:607`), and `open` compares the decrypted context to `self.context.encode()` before releasing plaintext (`:628`) — context is authenticated, not external metadata.
- **Nonce uniqueness/exhaustion — holds.** `NonceManager::next_nonce` is direction-local monotonic and returns `NonceExhausted` forever at `u64::MAX` — never wraps, never reuses (`:44-55`; `nonce_is_direction_local_and_fails_closed_at_wrap`).
- **Replay/old-phase rejection — holds.** `ReplayWindow` is a bounded sliding window rejecting duplicates (`Replay`) and out-of-window values (`TooOld`); `open` advances replay state **only after** AEAD authentication and context validation (`:631-632`). On `update_key_phase` both nonce and replay windows reset under the new phase (`:591-592`), so old-phase records fail the context check.
- **Key-update state transition — holds.** `update_key_phase` is bounded by `MAX_KEY_PHASE`, rekeys both directions, increments `key_phase` with `checked_add`, and resets per-phase nonce+replay state (`:580-593`).
- **Secret-safe error/debug surfaces — holds.** `SessionRejected` collapses external failures to one non-sensitive class; `LocalIdentity` deliberately does not implement `Debug`, and private key material never reaches an error/debug path.
- **Size bounds before auth/allocation — holds.** `seal`/`open`/`open_unreliable`/`seal_unreliable` all check length bounds before `write_message`/`read_message` or buffer allocation (`:563`, `:571`, `:602`, `:619`).

## Evidence

- `cargo test -p neko-crypto` on exact `9697ee7`: handshake binding, nonce exhaustion, replay window, key-phase, and context-mismatch tests all pass.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged. D019 persistent-replay policy remains maintainer-gated.
