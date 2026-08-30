# N7-11 minimal executable remediation: authenticated version admission

**Audit parent:** `12bb0985212a4dba5134e9836a96129caf4ff637`
**Disposition:** actionable design only; N7-11 is **not** resolved by this document.
**Identity boundary:** no identity file, key format, key loading, trust record, or authorization rule changes.

## Finding

`neko-wire::VersionNegotiator` is bounded and fail-closed, but it is used only by unit/compatibility/fuzz tests. Every executable path in `neko-cli` starts Noise IK immediately. Consequently the live TCP/UDP probe, failover, and TCP multistream paths neither negotiate a version nor cryptographically bind a negotiation to the resulting `SecureSession`. `admit_data()` is therefore not on a live data-admission boundary.

The smallest complete integration point is the **TCP multistream fixture** in `crates/neko-cli/src/multistream.rs`: it already composes a real `TcpStream`, Noise IK authentication, `SecureSession`, `ProcessMessage`, and `SessionRuntime` behind two local handshake helpers. It avoids duplicating the first patch across the larger probe/failover state machine.

## Decision

Run N1 negotiation before Noise, then bind the exact client hello, exact server response, and selected version into the Noise prologue. Do not use transport metadata, TCP delivery, or a post-handshake unauthenticated flag as proof of negotiation.

```text
client                         TCP                         server
  |-- frame(N1 exact hello) ------------------------------->|
  |<---------------- frame(N1 exact response) --------------|
  |  client verifies selection       server has selection   |
  |  binding = canonical(hello,response,selected)            |
  |-- frame(Noise IK msg1; prologue includes binding) ------>|
  |<--------- frame(Noise IK msg2; same prologue) -----------|
  |  VersionNegotiator::admit_data() must succeed on both    |
  |-- AEAD(ProcessMessage) --------------------------------->|
  |                 SessionRuntime admission                 |
```

Tampering, stripping, replaying a response for a different hello, or choosing different versions makes the Noise transcript/prologue differ and the IK handshake fail before any `ProcessMessage` is opened or passed to `SessionRuntime`.

## Exact patch plan

### 1. `crates/neko-wire/src/lib.rs`: retain and export a canonical binding

Add:

```rust
pub const NEGOTIATION_BINDING_DOMAIN: &[u8] =
    b"nekomusume/version-negotiation/v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationBinding(Vec<u8>);
impl NegotiationBinding {
    pub fn as_bytes(&self) -> &[u8];
}

impl VersionNegotiator {
    pub fn authenticated_binding(&self) -> Result<NegotiationBinding, NegotiationError>;
}
```

`VersionNegotiator` retains the exact hello and response bytes it sent/accepted. `authenticated_binding()` succeeds only in `Established(selected)` and encodes:

```text
domain || hello_len:u16be || exact_hello ||
          response_len:u16be || exact_response || selected:u16be
```

Lengths are bounded by the existing N1 limits. Use exact bytes, not reconstructed semantic values, so every accepted representation is committed. Keep existing `client_hello`, `server_accept_hello`, and `client_accept_response` signatures for compatibility. Invalid input remains terminal and neither `admit_data()` nor `authenticated_binding()` succeeds.

Do **not** introduce `neko-wire -> neko-crypto`; dependency remains toward the caller/composition root.

### 2. `crates/neko-crypto/src/lib.rs`: generic prologue-binding API

Add non-breaking constructors:

```rust
impl InitiatorHandshake {
    pub fn new_with_prologue_binding(
        local: &LocalIdentity,
        responder_public: &[u8],
        scope: &[u8],
        application_domain: &[u8],
        binding: &[u8],
    ) -> Result<Self, SessionRejected>;
}
impl ResponderHandshake {
    pub fn new_with_prologue_binding(
        local: &LocalIdentity,
        policy: TrustPolicy,
        application_domain: &[u8],
        binding: &[u8],
    ) -> Result<Self, SessionRejected>;
}
```

Refactor private `prologue` to encode unambiguously:

```text
PROLOGUE_PREFIX || domain_len:u16be || application_domain ||
                   binding_len:u16be || binding
```

Cap `binding` at 256 bytes and checked-add all capacities. The old `new` constructors delegate with an empty binding so non-N7-11 callers remain source-compatible. Apply the same internal helper to `with_resume_binding`; a follow-up may expose a bound resume constructor before failover adopts negotiation.

The API is intentionally byte-oriented: `neko-crypto` must not depend on wire policy. Noise already hashes the prologue into the handshake hash; no separate hash dependency or AEAD AAD fork is required.

**Compatibility warning:** changing the old empty-binding prologue encoding would break every current peer/vector. Preserve its existing byte encoding when `binding.is_empty()`. Only the new constructors use the extended, domain-separated encoding. This keeps the patch opt-in and makes multistream the sole changed wire path.

### 3. `crates/neko-cli/Cargo.toml`

Add direct dependency:

```toml
neko-wire = { path = "../neko-wire" }
```

The CLI is the composition root; this dependency direction is correct.

### 4. `crates/neko-cli/src/multistream.rs`: exact live composition

Add `SUPPORTED_VERSIONS: &[u16] = &[neko_wire::NEGOTIATION_VERSION]` and two helpers:

```rust
fn client_negotiate(socket: &mut TcpStream)
    -> (VersionNegotiator, NegotiationBinding);
fn server_negotiate(socket: &mut TcpStream)
    -> (VersionNegotiator, NegotiationBinding);
```

Order in `client_handshake` / `server_handshake`:

1. Exchange framed N1 hello/response using existing bounded `frame_read`/`frame_write`.
2. Complete local negotiator and obtain `authenticated_binding()`.
3. Build Noise via `*_with_prologue_binding(..., binding.as_bytes())`.
4. Complete Noise.
5. Call `negotiator.admit_data()` immediately before returning `SecureSession`.
6. Only then may `secure_read` decode `ProcessMessage` or call `SessionRuntime`.

Use the same external failure class already used by this fixture (for example `"handshake rejected"`); do not expose whether negotiation, authorization, or transcript authentication failed.

Do not initially modify `main.rs` probe/failover or UDP. Their datagram retry/demultiplexing requires an explicit anti-replay/amplification state decision and is not the smallest safe slice.

## Tests and executable acceptance

### `crates/neko-wire/src/lib.rs` unit tests

- established client/server produce byte-identical bindings;
- binding contains exact hello and exact response and is unavailable before establishment/after rejection;
- different offer sets that select the same version produce different bindings;
- late/invalid messages cannot replace a retained transcript.

### `crates/neko-crypto/src/lib.rs` unit tests

- equal non-empty bindings complete IK and pass an encrypted record;
- one-bit binding mismatch fails IK before `SecureSession` exists;
- empty-binding legacy constructors still interoperate and preserve the existing prologue path;
- oversized binding fails uniformly with `SessionRejected`.

### `crates/neko-cli/tests/multistream.rs` process tests

Keep the existing successful multistream test, which now proves negotiated/authenticated admission. Add a malicious raw TCP peer test that:

1. accepts/sends a valid N1 exchange;
2. changes one response/hello byte when constructing its local Noise binding (or reuses a response from a different hello);
3. attempts Noise;
4. asserts the CLI exits non-zero and no successful record/session diagnostic is emitted.

Add unsupported-only negotiation coverage using a test-only raw peer; assert fail-closed before any Noise/data frame. Do not add a CLI option for arbitrary versions in this minimal patch.

Run:

```sh
cargo fmt --all -- --check
cargo test -p neko-wire
cargo test -p neko-crypto
cargo test -p neko-cli --test multistream
cargo test --workspace
./scripts/check.sh
```

## Alternatives rejected

1. **Put hello/response inside Noise payload.** Strong authentication, but the responder must choose and return the version within a revised handshake payload/state API; this is more invasive and entangles authorization payload/resume formats.
2. **Bind selected version only in `RecordContext`/AEAD plaintext.** Too weak: it does not commit the offer/response transcript and data admission occurs only after constructing a session under a potentially downgraded handshake.
3. **Concatenate negotiation bytes into `DOMAIN` only in CLI.** Executable but abuses the 128-byte application-domain limit and leaves ambiguous composition. A small generic crypto constructor earns its abstraction.

## N7-10 dependency and rollout

The remediation is logically independent of N7-10's compatibility/replay test work: it relies only on the bounded N1 API already present at the exact parent. The exact parent already contains `5e2ac79` (negotiator) and `067bc55` (N3 compatibility harness), so implementation need not wait for another N7-10 code change.

However, do not fan this into probe/failover/UDP until the minimal multistream slice passes the complete gate. Resume/failover needs a bound equivalent of `with_resume_binding`; UDP additionally needs bounded peer state, retransmission identity, and anti-amplification behavior. Until those follow-ups land, advertise negotiation only for the multistream fixture and do not claim global CLI/carrier coverage.

## Definition of done

N7-11 is resolved only when the code and tests above land and executable evidence shows: exact N1 transcript agreement, Noise failure on transcript mismatch, and no `ProcessMessage`/`SessionRuntime` admission before both negotiation and Noise authentication succeed. This design commit alone is not closure evidence.
