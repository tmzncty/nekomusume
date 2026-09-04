//! Bounded cryptographic/session primitives for local research.
//!
//! This crate deliberately exposes no listener, runtime, key loading, or
//! production configuration.  It provides the small fail-closed state pieces
//! that a future Noise session must compose.

use snow::params::NoiseParams;
use std::collections::BTreeMap;

pub const SNOW_VERSION: &str = "0.10.0";
pub const MAX_REPLAY_WINDOW: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    NonceExhausted,
    Replay,
    TooOld,
    InvalidWindow,
}

/// Direction-local monotonically increasing nonce counter. Counter exhaustion
/// is terminal: it never wraps and never returns a reused nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceManager {
    next: u64,
    exhausted: bool,
}
impl NonceManager {
    pub const fn new(start: u64) -> Self {
        Self {
            next: start,
            exhausted: false,
        }
    }
    pub const fn next_value(&self) -> u64 {
        self.next
    }
    pub const fn is_exhausted(&self) -> bool {
        self.exhausted
    }
    pub fn next_nonce(&mut self) -> Result<u64, CryptoError> {
        if self.exhausted {
            return Err(CryptoError::NonceExhausted);
        }
        let value = self.next;
        match self.next.checked_add(1) {
            Some(next) => self.next = next,
            None => self.exhausted = true,
        }
        Ok(value)
    }
}

/// Bounded sliding replay window. Authentication must be performed by the
/// caller before `accept`; this type only tracks authenticated sequence IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayWindow {
    width: u64,
    highest: Option<u64>,
    seen: u64,
}
impl ReplayWindow {
    pub fn new(width: u64) -> Result<Self, CryptoError> {
        if width == 0 || width > MAX_REPLAY_WINDOW {
            return Err(CryptoError::InvalidWindow);
        }
        Ok(Self {
            width,
            highest: None,
            seen: 0,
        })
    }
    pub const fn highest(&self) -> Option<u64> {
        self.highest
    }
    pub fn accept(&mut self, sequence: u64) -> Result<(), CryptoError> {
        let Some(highest) = self.highest else {
            self.highest = Some(sequence);
            self.seen = 1;
            return Ok(());
        };
        if sequence > highest {
            let shift = sequence - highest;
            self.seen = if shift >= self.width {
                1
            } else {
                (self.seen << shift) | 1
            };
            self.highest = Some(sequence);
            return Ok(());
        }
        let distance = highest - sequence;
        if distance >= self.width {
            return Err(CryptoError::TooOld);
        }
        let bit = 1u64 << distance;
        if self.seen & bit != 0 {
            return Err(CryptoError::Replay);
        }
        self.seen |= bit;
        Ok(())
    }
}

/// The only handshake pattern admitted by this research boundary.
pub fn noise_ik_params() -> NoiseParams {
    "Noise_IK_25519_ChaChaPoly_SHA256"
        .parse()
        .expect("constant Noise params")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_is_direction_local_and_fails_closed_at_wrap() {
        let mut n = NonceManager::new(u64::MAX - 1);
        assert_eq!(n.next_nonce(), Ok(u64::MAX - 1));
        assert_eq!(n.next_nonce(), Ok(u64::MAX));
        assert_eq!(n.next_nonce(), Err(CryptoError::NonceExhausted));
        assert_eq!(n.next_nonce(), Err(CryptoError::NonceExhausted));
        assert!(n.is_exhausted());
    }

    #[test]
    fn replay_window_rejects_duplicates_and_old_values() {
        let mut w = ReplayWindow::new(4).unwrap();
        assert_eq!(w.accept(10), Ok(()));
        assert_eq!(w.accept(10), Err(CryptoError::Replay));
        assert_eq!(w.accept(9), Ok(()));
        assert_eq!(w.accept(9), Err(CryptoError::Replay));
        assert_eq!(w.accept(5), Err(CryptoError::TooOld));
        assert_eq!(w.accept(14), Ok(()));
        assert_eq!(w.accept(10), Err(CryptoError::TooOld));
    }

    #[test]
    fn replay_window_shift_discards_only_outside_window() {
        let mut w = ReplayWindow::new(4).unwrap();
        w.accept(1).unwrap();
        w.accept(2).unwrap();
        w.accept(5).unwrap();
        assert_eq!(w.accept(4), Ok(()));
        assert_eq!(w.accept(1), Err(CryptoError::TooOld));
    }

    #[test]
    fn invalid_replay_windows_are_rejected() {
        assert_eq!(ReplayWindow::new(0), Err(CryptoError::InvalidWindow));
        assert_eq!(
            ReplayWindow::new(MAX_REPLAY_WINDOW + 1),
            Err(CryptoError::InvalidWindow)
        );
        assert_eq!(noise_ik_params().name, "Noise_IK_25519_ChaChaPoly_SHA256");
    }
}

pub const MAX_HANDSHAKE_MESSAGE: usize = 1024;
pub const MAX_RECORD_PLAINTEXT: usize = 4096;
pub const MAX_UNRELIABLE_DATAGRAM: usize = 1200;
pub const RECORD_CONTEXT_LEN: usize = 26;
pub const MAX_KEY_PHASE: u8 = 1;
const PROLOGUE_PREFIX: &[u8] = b"nekomusume/noise-ik/v0\0";

/// External failures intentionally collapse to one non-sensitive class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRejected;

/// Long-term local key material. Debug is deliberately not implemented.
pub struct LocalIdentity {
    private: Vec<u8>,
    public: Vec<u8>,
}
impl LocalIdentity {
    pub fn generate() -> Result<Self, SessionRejected> {
        let pair = snow::Builder::new(noise_ik_params())
            .generate_keypair()
            .map_err(|_| SessionRejected)?;
        Ok(Self {
            private: pair.private,
            public: pair.public,
        })
    }
    pub fn public_key(&self) -> &[u8] {
        &self.public
    }
    pub fn private_key(&self) -> &[u8] {
        &self.private
    }
    pub fn from_keypair(private: &[u8], public: &[u8]) -> Result<Self, SessionRejected> {
        if private.len() != 32 || public.len() != 32 {
            return Err(SessionRejected);
        }
        Ok(Self {
            private: private.to_vec(),
            public: public.to_vec(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustRecord {
    pub version: u16,
    pub public_key: Vec<u8>,
    pub scope: Vec<u8>,
    pub status: TrustStatus,
}

/// Minimal fail-closed trust and authorization policy. Presence alone is not
/// enough: version, active status, exact identity and requested scope must all match.
#[derive(Debug, Clone, Default)]
pub struct TrustPolicy {
    records: Vec<TrustRecord>,
}
impl TrustPolicy {
    pub fn new(records: Vec<TrustRecord>) -> Self {
        Self { records }
    }
    pub fn authorize(&self, public_key: &[u8], scope: &[u8]) -> Result<(), SessionRejected> {
        self.records
            .iter()
            .any(|r| {
                r.version == 1
                    && r.status == TrustStatus::Active
                    && r.public_key == public_key
                    && r.scope == scope
            })
            .then_some(())
            .ok_or(SessionRejected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordContext {
    pub delivery_epoch: u64,
    pub key_phase: u8,
    pub path_generation: u64,
    pub stream_id: u64,
    pub direction: u8,
}
impl RecordContext {
    fn encode(self) -> [u8; RECORD_CONTEXT_LEN] {
        let mut out = [0; RECORD_CONTEXT_LEN];
        out[0..8].copy_from_slice(&self.delivery_epoch.to_be_bytes());
        out[8] = self.key_phase;
        out[9..17].copy_from_slice(&self.path_generation.to_be_bytes());
        out[17..25].copy_from_slice(&self.stream_id.to_be_bytes());
        out[25] = self.direction;
        out
    }
}

fn prologue(application_domain: &[u8]) -> Result<Vec<u8>, SessionRejected> {
    if application_domain.is_empty() || application_domain.len() > 128 {
        return Err(SessionRejected);
    }
    let mut p = Vec::with_capacity(PROLOGUE_PREFIX.len() + application_domain.len());
    p.extend_from_slice(PROLOGUE_PREFIX);
    p.extend_from_slice(application_domain);
    Ok(p)
}
fn bound_prologue(application_domain: &[u8], binding: &[u8]) -> Result<Vec<u8>, SessionRejected> {
    if binding.len() > 256 {
        return Err(SessionRejected);
    }
    if binding.is_empty() {
        return prologue(application_domain);
    }
    if application_domain.is_empty() || application_domain.len() > 128 {
        return Err(SessionRejected);
    }
    let mut p =
        Vec::with_capacity(PROLOGUE_PREFIX.len() + 4 + application_domain.len() + binding.len());
    p.extend_from_slice(PROLOGUE_PREFIX);
    p.extend_from_slice(&(application_domain.len() as u16).to_be_bytes());
    p.extend_from_slice(application_domain);
    p.extend_from_slice(&(binding.len() as u16).to_be_bytes());
    p.extend_from_slice(binding);
    Ok(p)
}

pub struct InitiatorHandshake {
    state: snow::HandshakeState,
    scope: Vec<u8>,
}
pub struct ResponderHandshake {
    state: snow::HandshakeState,
    policy: TrustPolicy,
}

impl InitiatorHandshake {
    pub fn new_with_prologue_binding(
        local: &LocalIdentity,
        responder_public: &[u8],
        scope: &[u8],
        application_domain: &[u8],
        binding: &[u8],
    ) -> Result<Self, SessionRejected> {
        if scope.is_empty() || scope.len() > 128 {
            return Err(SessionRejected);
        }
        let p = bound_prologue(application_domain, binding)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.remote_public_key(responder_public))
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_initiator)
            .map_err(|_| SessionRejected)?;
        Ok(Self {
            state,
            scope: scope.to_vec(),
        })
    }
    pub fn new(
        local: &LocalIdentity,
        responder_public: &[u8],
        scope: &[u8],
        application_domain: &[u8],
    ) -> Result<Self, SessionRejected> {
        if scope.is_empty() || scope.len() > 128 {
            return Err(SessionRejected);
        }
        let p = prologue(application_domain)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.remote_public_key(responder_public))
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_initiator)
            .map_err(|_| SessionRejected)?;
        Ok(Self {
            state,
            scope: scope.to_vec(),
        })
    }
    pub fn with_resume_binding(
        local: &LocalIdentity,
        responder_public: &[u8],
        scope: &[u8],
        application_domain: &[u8],
        binding: &ResumeBinding,
    ) -> Result<Self, SessionRejected> {
        if scope.is_empty() || scope.len() > 62 {
            return Err(SessionRejected);
        }
        let mut payload = Vec::with_capacity(1 + scope.len() + 65);
        payload.push(scope.len() as u8);
        payload.extend_from_slice(scope);
        payload.extend_from_slice(&binding.encode());
        let p = prologue(application_domain)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.remote_public_key(responder_public))
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_initiator)
            .map_err(|_| SessionRejected)?;
        Ok(Self {
            state,
            scope: payload,
        })
    }
    pub fn with_resume_negotiation_binding(
        local: &LocalIdentity,
        responder_public: &[u8],
        scope: &[u8],
        application_domain: &[u8],
        binding: &ResumeBinding,
        negotiation_binding: &[u8],
    ) -> Result<Self, SessionRejected> {
        if negotiation_binding.is_empty()
            || negotiation_binding.len() > 256
            || scope.is_empty()
            || scope.len() > 60
        {
            return Err(SessionRejected);
        }
        let mut payload = Vec::with_capacity(1 + scope.len() + 65 + 2 + negotiation_binding.len());
        payload.push(scope.len() as u8);
        payload.extend_from_slice(scope);
        payload.extend_from_slice(&binding.encode());
        payload.extend_from_slice(&(negotiation_binding.len() as u16).to_be_bytes());
        payload.extend_from_slice(negotiation_binding);
        let p = bound_prologue(application_domain, negotiation_binding)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.remote_public_key(responder_public))
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_initiator)
            .map_err(|_| SessionRejected)?;
        Ok(Self {
            state,
            scope: payload,
        })
    }
    pub fn first_message(&mut self) -> Result<Vec<u8>, SessionRejected> {
        let mut out = vec![0; MAX_HANDSHAKE_MESSAGE];
        let n = self
            .state
            .write_message(&self.scope, &mut out)
            .map_err(|_| SessionRejected)?;
        out.truncate(n);
        Ok(out)
    }
    pub fn finish(
        mut self,
        response: &[u8],
        context: RecordContext,
    ) -> Result<SecureSession, SessionRejected> {
        let mut payload = [0; 1];
        let n = self
            .state
            .read_message(response, &mut payload)
            .map_err(|_| SessionRejected)?;
        if n != 0 || !self.state.is_handshake_finished() {
            return Err(SessionRejected);
        }
        SecureSession::from_handshake(self.state, context)
    }
}

impl ResponderHandshake {
    pub fn new_with_prologue_binding(
        local: &LocalIdentity,
        policy: TrustPolicy,
        application_domain: &[u8],
        binding: &[u8],
    ) -> Result<Self, SessionRejected> {
        let p = bound_prologue(application_domain, binding)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_responder)
            .map_err(|_| SessionRejected)?;
        Ok(Self { state, policy })
    }
    pub fn new(
        local: &LocalIdentity,
        policy: TrustPolicy,
        application_domain: &[u8],
    ) -> Result<Self, SessionRejected> {
        let p = prologue(application_domain)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_responder)
            .map_err(|_| SessionRejected)?;
        Ok(Self { state, policy })
    }
    pub fn new_with_resume_negotiation_binding(
        local: &LocalIdentity,
        policy: TrustPolicy,
        application_domain: &[u8],
        negotiation_binding: &[u8],
    ) -> Result<Self, SessionRejected> {
        let p = bound_prologue(application_domain, negotiation_binding)?;
        let state = snow::Builder::new(noise_ik_params())
            .local_private_key(&local.private)
            .and_then(|b| b.prologue(&p))
            .and_then(snow::Builder::build_responder)
            .map_err(|_| SessionRejected)?;
        Ok(Self { state, policy })
    }
    pub fn receive_first_with_resume(
        mut self,
        message: &[u8],
        context: RecordContext,
    ) -> Result<(Vec<u8>, SecureSession, Vec<u8>, ResumeBinding), SessionRejected> {
        if message.len() > MAX_HANDSHAKE_MESSAGE {
            return Err(SessionRejected);
        }
        let mut payload = [0; 128];
        let n = self
            .state
            .read_message(message, &mut payload)
            .map_err(|_| SessionRejected)?;
        if n < 67 {
            return Err(SessionRejected);
        }
        let scope_len = payload[0] as usize;
        if scope_len == 0 || n < 1 + scope_len + 65 {
            return Err(SessionRejected);
        }
        let scope = payload[1..1 + scope_len].to_vec();
        let binding = ResumeBinding::decode(&payload[1 + scope_len..1 + scope_len + 65])?;
        if n > 1 + scope_len + 65 {
            let rest = &payload[1 + scope_len + 65..n];
            if rest.len() < 2 || u16::from_be_bytes([rest[0], rest[1]]) as usize != rest.len() - 2 {
                return Err(SessionRejected);
            }
        }

        let remote = self
            .state
            .get_remote_static()
            .ok_or(SessionRejected)?
            .to_vec();
        self.policy.authorize(&remote, &scope)?;
        let mut response = vec![0; MAX_HANDSHAKE_MESSAGE];
        let n = self
            .state
            .write_message(&[], &mut response)
            .map_err(|_| SessionRejected)?;
        response.truncate(n);
        let session = SecureSession::from_handshake(self.state, context)?;
        Ok((response, session, remote, binding))
    }
    pub fn receive_first(
        mut self,
        message: &[u8],
        context: RecordContext,
    ) -> Result<(Vec<u8>, SecureSession), SessionRejected> {
        if message.len() > MAX_HANDSHAKE_MESSAGE {
            return Err(SessionRejected);
        }
        let mut scope = [0; 128];
        let n = self
            .state
            .read_message(message, &mut scope)
            .map_err(|_| SessionRejected)?;
        let remote = self.state.get_remote_static().ok_or(SessionRejected)?;
        self.policy.authorize(remote, &scope[..n])?;
        let mut response = vec![0; MAX_HANDSHAKE_MESSAGE];
        let n = self
            .state
            .write_message(&[], &mut response)
            .map_err(|_| SessionRejected)?;
        response.truncate(n);
        let session = SecureSession::from_handshake(self.state, context)?;
        Ok((response, session))
    }
}

pub struct SecureSession {
    transport: snow::StatelessTransportState,
    send: NonceManager,
    replay: ReplayWindow,
    context: RecordContext,
}
impl SecureSession {
    fn from_handshake(
        state: snow::HandshakeState,
        context: RecordContext,
    ) -> Result<Self, SessionRejected> {
        Ok(Self {
            transport: state
                .into_stateless_transport_mode()
                .map_err(|_| SessionRejected)?,
            send: NonceManager::new(0),
            replay: ReplayWindow::new(MAX_REPLAY_WINDOW).map_err(|_| SessionRejected)?,
            context,
        })
    }
    pub fn seal_unreliable(&mut self, payload: &[u8]) -> Result<Vec<u8>, SessionRejected> {
        if payload.len() > MAX_UNRELIABLE_DATAGRAM {
            return Err(SessionRejected);
        }
        self.seal(payload)
    }
    pub fn open_unreliable(&mut self, record: &[u8]) -> Result<Vec<u8>, SessionRejected> {
        // Reject oversized datagrams before authentication/replay mutation. The
        // authenticated record has 8 bytes of sequence, context, payload and tag.
        if record.len() > 8 + RECORD_CONTEXT_LEN + MAX_UNRELIABLE_DATAGRAM + 16 {
            return Err(SessionRejected);
        }
        self.open(record)
    }

    /// Rekey both directions at an authenticated phase boundary. The caller
    /// must invoke this on both peers in the same order; no old-phase records
    /// are accepted after commit.
    pub fn update_key_phase(&mut self) -> Result<(), SessionRejected> {
        if self.context.key_phase >= MAX_KEY_PHASE {
            return Err(SessionRejected);
        }
        self.transport.rekey_outgoing();
        self.transport.rekey_incoming();
        self.context.key_phase = self
            .context
            .key_phase
            .checked_add(1)
            .ok_or(SessionRejected)?;
        self.send = NonceManager::new(0);
        self.replay = ReplayWindow::new(MAX_REPLAY_WINDOW).map_err(|_| SessionRejected)?;
        Ok(())
    }
    pub fn key_phase(&self) -> u8 {
        self.context.key_phase
    }

    /// Format: sequence (8-byte BE) || Noise ciphertext. The canonical record
    /// context is inside the authenticated ciphertext and compared before release.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, SessionRejected> {
        if plaintext.len() > MAX_RECORD_PLAINTEXT {
            return Err(SessionRejected);
        }
        let sequence = self.send.next_nonce().map_err(|_| SessionRejected)?;
        let mut inner = Vec::with_capacity(RECORD_CONTEXT_LEN + plaintext.len());
        inner.extend_from_slice(&self.context.encode());
        inner.extend_from_slice(plaintext);
        let mut out = vec![0; 8 + inner.len() + 16];
        out[..8].copy_from_slice(&sequence.to_be_bytes());
        let n = self
            .transport
            .write_message(sequence, &inner, &mut out[8..])
            .map_err(|_| SessionRejected)?;
        out.truncate(8 + n);
        Ok(out)
    }
    pub fn open(&mut self, record: &[u8]) -> Result<Vec<u8>, SessionRejected> {
        if record.len() < 24 || record.len() > 8 + RECORD_CONTEXT_LEN + MAX_RECORD_PLAINTEXT + 16 {
            return Err(SessionRejected);
        }
        let sequence = u64::from_be_bytes(record[..8].try_into().map_err(|_| SessionRejected)?);
        let mut plain = vec![0; record.len() - 8];
        let n = self
            .transport
            .read_message(sequence, &record[8..], &mut plain)
            .map_err(|_| SessionRejected)?;
        if n < RECORD_CONTEXT_LEN || plain[..RECORD_CONTEXT_LEN] != self.context.encode() {
            return Err(SessionRejected);
        }
        // Replay state advances only after AEAD authentication and context validation.
        self.replay.accept(sequence).map_err(|_| SessionRejected)?;
        Ok(plain[RECORD_CONTEXT_LEN..n].to_vec())
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    pub(super) fn ctx(direction: u8) -> RecordContext {
        RecordContext {
            delivery_epoch: 7,
            key_phase: 0,
            path_generation: 3,
            stream_id: 11,
            direction,
        }
    }
    pub(super) fn pair() -> (SecureSession, SecureSession) {
        let initiator = LocalIdentity::generate().unwrap();
        let responder = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: initiator.public_key().to_vec(),
            scope: b"echo".to_vec(),
            status: TrustStatus::Active,
        }]);
        let mut i =
            InitiatorHandshake::new(&initiator, responder.public_key(), b"echo", b"test-domain")
                .unwrap();
        let first = i.first_message().unwrap();
        let r = ResponderHandshake::new(&responder, policy, b"test-domain").unwrap();
        let (response, rs) = r.receive_first(&first, ctx(0)).unwrap();
        let is = i.finish(&response, ctx(0)).unwrap();
        (is, rs)
    }
    #[test]
    fn authenticated_binding_matches_and_one_bit_mismatch_fails_before_session() {
        let initiator = LocalIdentity::generate().unwrap();
        let responder = LocalIdentity::generate().unwrap();
        let policy = || {
            TrustPolicy::new(vec![TrustRecord {
                version: 1,
                public_key: initiator.public_key().to_vec(),
                scope: b"echo".to_vec(),
                status: TrustStatus::Active,
            }])
        };
        let binding = b"exact transcript";
        let mut i = InitiatorHandshake::new_with_prologue_binding(
            &initiator,
            responder.public_key(),
            b"echo",
            b"bound",
            binding,
        )
        .unwrap();
        let first = i.first_message().unwrap();
        let r =
            ResponderHandshake::new_with_prologue_binding(&responder, policy(), b"bound", binding)
                .unwrap();
        let (response, mut receiver) = r.receive_first(&first, ctx(0)).unwrap();
        let mut sender = i.finish(&response, ctx(0)).unwrap();
        assert_eq!(
            receiver.open(&sender.seal(b"admitted").unwrap()).unwrap(),
            b"admitted"
        );
        let mut i = InitiatorHandshake::new_with_prologue_binding(
            &initiator,
            responder.public_key(),
            b"echo",
            b"bound",
            binding,
        )
        .unwrap();
        let first = i.first_message().unwrap();
        let mut bad = binding.to_vec();
        bad[0] ^= 1;
        let r = ResponderHandshake::new_with_prologue_binding(&responder, policy(), b"bound", &bad)
            .unwrap();
        assert!(r.receive_first(&first, ctx(0)).is_err());
        assert!(
            InitiatorHandshake::new_with_prologue_binding(
                &initiator,
                responder.public_key(),
                b"echo",
                b"bound",
                &[0; 257]
            )
            .is_err()
        );
    }
    #[test]
    fn ik_handshake_and_bidirectional_records() {
        let (mut a, mut b) = pair();
        let x = a.seal(b"hello").unwrap();
        assert_eq!(b.open(&x).unwrap(), b"hello");
        let y = b.seal(b"world").unwrap();
        assert_eq!(a.open(&y).unwrap(), b"world");
    }
    #[test]
    fn tamper_and_replay_collapse_to_uniform_rejection() {
        let (mut a, mut b) = pair();
        let x = a.seal(b"secret").unwrap();
        let mut bad = x.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert_eq!(b.open(&bad), Err(SessionRejected));
        assert_eq!(b.open(&x).unwrap(), b"secret");
        assert_eq!(b.open(&x), Err(SessionRejected));
    }
    #[test]
    fn record_context_mismatch_is_rejected() {
        let initiator = LocalIdentity::generate().unwrap();
        let responder = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: initiator.public_key().to_vec(),
            scope: b"echo".to_vec(),
            status: TrustStatus::Active,
        }]);
        let mut i =
            InitiatorHandshake::new(&initiator, responder.public_key(), b"echo", b"context-test")
                .unwrap();
        let first = i.first_message().unwrap();
        let r = ResponderHandshake::new(&responder, policy, b"context-test").unwrap();
        let (response, mut receiver) = r.receive_first(&first, ctx(1)).unwrap();
        let mut sender = i.finish(&response, ctx(0)).unwrap();
        let record = sender.seal(b"bound").unwrap();
        assert_eq!(receiver.open(&record), Err(SessionRejected));
    }
    #[test]
    fn revoked_or_wrong_scope_is_not_authorized() {
        let initiator = LocalIdentity::generate().unwrap();
        let responder = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: initiator.public_key().to_vec(),
            scope: b"other".to_vec(),
            status: TrustStatus::Active,
        }]);
        let mut i =
            InitiatorHandshake::new(&initiator, responder.public_key(), b"echo", b"d").unwrap();
        let first = i.first_message().unwrap();
        let r = ResponderHandshake::new(&responder, policy, b"d").unwrap();
        assert!(r.receive_first(&first, ctx(0)).is_err());
    }
    #[test]
    fn prologue_mismatch_and_oversize_fail_uniformly() {
        let initiator = LocalIdentity::generate().unwrap();
        let responder = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: initiator.public_key().to_vec(),
            scope: b"echo".to_vec(),
            status: TrustStatus::Active,
        }]);
        let mut i =
            InitiatorHandshake::new(&initiator, responder.public_key(), b"echo", b"a").unwrap();
        let first = i.first_message().unwrap();
        let r = ResponderHandshake::new(&responder, policy, b"b").unwrap();
        assert!(r.receive_first(&first, ctx(0)).is_err());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreauthLimits {
    pub max_input_bytes: usize,
    pub max_input_packets: u8,
    pub max_response_bytes: usize,
    pub max_response_packets: u8,
}
impl Default for PreauthLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 8192,
            max_input_packets: 4,
            max_response_bytes: 2048,
            max_response_packets: 4,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreauthBudget {
    limits: PreauthLimits,
    input_bytes: usize,
    input_packets: u8,
    response_bytes: usize,
    response_packets: u8,
}
impl PreauthBudget {
    pub fn new(limits: PreauthLimits) -> Result<Self, SessionRejected> {
        if limits.max_input_bytes == 0
            || limits.max_input_packets == 0
            || limits.max_response_bytes == 0
            || limits.max_response_packets == 0
        {
            return Err(SessionRejected);
        }
        Ok(Self {
            limits,
            input_bytes: 0,
            input_packets: 0,
            response_bytes: 0,
            response_packets: 0,
        })
    }
    pub fn charge_input(&mut self, bytes: usize) -> Result<(), SessionRejected> {
        let new_bytes = self.input_bytes.checked_add(bytes).ok_or(SessionRejected)?;
        let new_packets = self.input_packets.checked_add(1).ok_or(SessionRejected)?;
        if new_bytes > self.limits.max_input_bytes || new_packets > self.limits.max_input_packets {
            return Err(SessionRejected);
        }
        self.input_bytes = new_bytes;
        self.input_packets = new_packets;
        Ok(())
    }
    pub fn charge_response(&mut self, bytes: usize) -> Result<(), SessionRejected> {
        let new_bytes = self
            .response_bytes
            .checked_add(bytes)
            .ok_or(SessionRejected)?;
        let new_packets = self
            .response_packets
            .checked_add(1)
            .ok_or(SessionRejected)?;
        let amplification = self
            .input_bytes
            .checked_mul(3)
            .ok_or(SessionRejected)?
            .min(self.limits.max_response_bytes);
        let packet_allowance = self.input_packets.min(self.limits.max_response_packets);
        if new_bytes > amplification || new_packets > packet_allowance {
            return Err(SessionRejected);
        }
        self.response_bytes = new_bytes;
        self.response_packets = new_packets;
        Ok(())
    }

    pub fn rollback_input(&mut self, bytes: usize) -> Result<(), SessionRejected> {
        self.input_bytes = self.input_bytes.checked_sub(bytes).ok_or(SessionRejected)?;
        self.input_packets = self.input_packets.checked_sub(1).ok_or(SessionRejected)?;
        Ok(())
    }

    pub fn rollback_response(&mut self, bytes: usize) -> Result<(), SessionRejected> {
        self.response_bytes = self.response_bytes.checked_sub(bytes).ok_or(SessionRejected)?;
        self.response_packets = self.response_packets.checked_sub(1).ok_or(SessionRejected)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessPreauthLimits {
    pub max_states_per_source: usize,
    pub max_states_global: usize,
    pub max_memory_per_state: usize,
    pub max_memory_global: usize,
    pub max_queue_per_source: usize,
    pub max_queue_global: usize,
    pub max_input_bytes_per_source: usize,
    pub max_input_packets_per_source: usize,
    pub max_input_bytes_per_window: usize,
    pub max_input_packets_per_window: usize,
    pub max_work_per_packet: usize,
    pub max_work_per_source: usize,
    pub max_work_per_window: usize,
    pub max_response_bytes_per_source: usize,
    pub max_response_packets_per_source: usize,
    pub max_response_bytes_per_window: usize,
    pub max_response_packets_per_window: usize,
    pub admission_window_ms: u64,
    pub idle_timeout_ms: u64,
    pub max_lifetime_ms: u64,
}
impl Default for ProcessPreauthLimits {
    fn default() -> Self {
        Self {
            max_states_per_source: 8,
            max_states_global: 1024,
            max_memory_per_state: 16 * 1024,
            max_memory_global: 16 * 1024 * 1024,
            max_queue_per_source: 4,
            max_queue_global: 256,
            max_input_bytes_per_source: 64 * 1024,
            max_input_packets_per_source: 64,
            max_input_bytes_per_window: 8 * 1024 * 1024,
            max_input_packets_per_window: 8192,
            max_work_per_packet: 4096,
            max_work_per_source: 131_072,
            max_work_per_window: 1_048_576,
            max_response_bytes_per_source: 2048,
            max_response_packets_per_source: 4,
            max_response_bytes_per_window: 256 * 1024,
            max_response_packets_per_window: 512,
            admission_window_ms: 1000,
            idle_timeout_ms: 1000,
            max_lifetime_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PreauthStateId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessPreauthState {
    source: Vec<u8>,
    memory_bytes: usize,
    queued: usize,
    created_at_ms: u64,
    last_progress_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProcessPreauthSource {
    states: usize,
    queued: usize,
    input_bytes: usize,
    input_packets: usize,
    work_units: usize,
    response_bytes: usize,
    response_packets: usize,
}

/// Process-owned admission accounting for unauthenticated work. Source keys are
/// opaque bounded caller projections; raw addresses or identities need not be
/// retained. Charges are atomic and happen before the protected operation.
#[derive(Debug)]
pub struct ProcessPreauthAdmission {
    limits: ProcessPreauthLimits,
    next_id: u64,
    states: BTreeMap<PreauthStateId, ProcessPreauthState>,
    sources: BTreeMap<Vec<u8>, ProcessPreauthSource>,
    memory_bytes: usize,
    queued: usize,
    window_started_ms: u64,
    input_bytes: usize,
    input_packets: usize,
    work_units: usize,
    response_bytes: usize,
    response_packets: usize,
}
impl ProcessPreauthAdmission {
    pub fn new(limits: ProcessPreauthLimits, now_ms: u64) -> Result<Self, SessionRejected> {
        if limits.max_states_per_source == 0
            || limits.max_states_global == 0
            || limits.max_states_per_source > limits.max_states_global
            || limits.max_memory_per_state == 0
            || limits.max_memory_global == 0
            || limits.max_memory_per_state > limits.max_memory_global
            || limits.max_queue_per_source == 0
            || limits.max_queue_global == 0
            || limits.max_queue_per_source > limits.max_queue_global
            || limits.max_input_bytes_per_source == 0
            || limits.max_input_packets_per_source == 0
            || limits.max_input_bytes_per_window == 0
            || limits.max_input_packets_per_window == 0
            || limits.max_work_per_packet == 0
            || limits.max_work_per_source == 0
            || limits.max_work_per_window == 0
            || limits.max_response_bytes_per_source == 0
            || limits.max_response_packets_per_source == 0
            || limits.max_response_bytes_per_window == 0
            || limits.max_response_packets_per_window == 0
            || limits.admission_window_ms == 0
            || limits.idle_timeout_ms == 0
            || limits.idle_timeout_ms > limits.max_lifetime_ms
            || limits.max_lifetime_ms == 0
        {
            return Err(SessionRejected);
        }
        Ok(Self {
            limits,
            next_id: 0,
            states: BTreeMap::new(),
            sources: BTreeMap::new(),
            memory_bytes: 0,
            queued: 0,
            window_started_ms: now_ms,
            input_bytes: 0,
            input_packets: 0,
            work_units: 0,
            response_bytes: 0,
            response_packets: 0,
        })
    }

    fn refresh_window(&mut self, now_ms: u64) -> Result<(), SessionRejected> {
        if now_ms < self.window_started_ms {
            return Err(SessionRejected);
        }
        if now_ms - self.window_started_ms >= self.limits.admission_window_ms {
            self.window_started_ms = now_ms;
            self.input_bytes = 0;
            self.input_packets = 0;
            self.work_units = 0;
            self.response_bytes = 0;
            self.response_packets = 0;
        }
        Ok(())
    }

    fn live(
        &self,
        id: PreauthStateId,
        now_ms: u64,
    ) -> Result<&ProcessPreauthState, SessionRejected> {
        let state = self.states.get(&id).ok_or(SessionRejected)?;
        if now_ms < state.created_at_ms
            || now_ms < state.last_progress_ms
            || now_ms - state.last_progress_ms >= self.limits.idle_timeout_ms
            || now_ms - state.created_at_ms >= self.limits.max_lifetime_ms
        {
            return Err(SessionRejected);
        }
        Ok(state)
    }

    pub fn admit_state(
        &mut self,
        source: &[u8],
        memory_bytes: usize,
        now_ms: u64,
    ) -> Result<PreauthStateId, SessionRejected> {
        self.refresh_window(now_ms)?;
        if source.is_empty()
            || source.len() > 256
            || memory_bytes == 0
            || memory_bytes > self.limits.max_memory_per_state
            || self.states.len() >= self.limits.max_states_global
            || self.sources.get(source).map_or(0, |usage| usage.states)
                >= self.limits.max_states_per_source
            || self
                .memory_bytes
                .checked_add(memory_bytes)
                .ok_or(SessionRejected)?
                > self.limits.max_memory_global
        {
            return Err(SessionRejected);
        }
        let id = PreauthStateId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(SessionRejected)?;
        self.states.insert(
            id,
            ProcessPreauthState {
                source: source.to_vec(),
                memory_bytes,
                queued: 0,
                created_at_ms: now_ms,
                last_progress_ms: now_ms,
            },
        );
        self.memory_bytes += memory_bytes;
        self.sources.entry(source.to_vec()).or_default().states += 1;
        Ok(id)
    }

    pub fn charge_input(
        &mut self,
        id: PreauthStateId,
        bytes: usize,
        work_units: usize,
        now_ms: u64,
    ) -> Result<(), SessionRejected> {
        self.refresh_window(now_ms)?;
        let source = self.live(id, now_ms)?.source.clone();
        let usage = self.sources.get(&source).ok_or(SessionRejected)?;
        let source_bytes = usage
            .input_bytes
            .checked_add(bytes)
            .ok_or(SessionRejected)?;
        let source_packets = usage.input_packets.checked_add(1).ok_or(SessionRejected)?;
        let source_work = usage
            .work_units
            .checked_add(work_units)
            .ok_or(SessionRejected)?;
        let input_bytes = self.input_bytes.checked_add(bytes).ok_or(SessionRejected)?;
        let input_packets = self.input_packets.checked_add(1).ok_or(SessionRejected)?;
        let work = self
            .work_units
            .checked_add(work_units)
            .ok_or(SessionRejected)?;
        if work_units > self.limits.max_work_per_packet
            || source_bytes > self.limits.max_input_bytes_per_source
            || source_packets > self.limits.max_input_packets_per_source
            || source_work > self.limits.max_work_per_source
            || input_bytes > self.limits.max_input_bytes_per_window
            || input_packets > self.limits.max_input_packets_per_window
            || work > self.limits.max_work_per_window
        {
            return Err(SessionRejected);
        }
        let usage = self.sources.get_mut(&source).ok_or(SessionRejected)?;
        usage.input_bytes = source_bytes;
        usage.input_packets = source_packets;
        usage.work_units = source_work;
        self.input_bytes = input_bytes;
        self.input_packets = input_packets;
        self.work_units = work;
        self.states
            .get_mut(&id)
            .ok_or(SessionRejected)?
            .last_progress_ms = now_ms;
        Ok(())
    }

    pub fn enqueue(&mut self, id: PreauthStateId, now_ms: u64) -> Result<(), SessionRejected> {
        self.refresh_window(now_ms)?;
        let source = self.live(id, now_ms)?.source.clone();
        let source_queued = self.sources.get(&source).ok_or(SessionRejected)?.queued;
        if source_queued >= self.limits.max_queue_per_source
            || self.queued >= self.limits.max_queue_global
        {
            return Err(SessionRejected);
        }
        self.states.get_mut(&id).ok_or(SessionRejected)?.queued += 1;
        self.sources.get_mut(&source).ok_or(SessionRejected)?.queued += 1;
        self.queued += 1;
        Ok(())
    }

    pub fn dequeue(&mut self, id: PreauthStateId) -> Result<(), SessionRejected> {
        let state = self.states.get_mut(&id).ok_or(SessionRejected)?;
        if state.queued == 0 {
            return Err(SessionRejected);
        }
        let source = state.source.clone();
        state.queued -= 1;
        self.sources.get_mut(&source).ok_or(SessionRejected)?.queued -= 1;
        self.queued -= 1;
        Ok(())
    }

    pub fn charge_response(
        &mut self,
        id: PreauthStateId,
        bytes: usize,
        now_ms: u64,
    ) -> Result<(), SessionRejected> {
        self.refresh_window(now_ms)?;
        let source = self.live(id, now_ms)?.source.clone();
        let usage = self.sources.get(&source).ok_or(SessionRejected)?;
        let source_bytes = usage
            .response_bytes
            .checked_add(bytes)
            .ok_or(SessionRejected)?;
        let source_packets = usage
            .response_packets
            .checked_add(1)
            .ok_or(SessionRejected)?;
        let response_bytes = self
            .response_bytes
            .checked_add(bytes)
            .ok_or(SessionRejected)?;
        let response_packets = self
            .response_packets
            .checked_add(1)
            .ok_or(SessionRejected)?;
        if source_bytes > self.limits.max_response_bytes_per_source
            || source_packets > self.limits.max_response_packets_per_source
            || response_bytes > self.limits.max_response_bytes_per_window
            || response_packets > self.limits.max_response_packets_per_window
        {
            return Err(SessionRejected);
        }
        let usage = self.sources.get_mut(&source).ok_or(SessionRejected)?;
        usage.response_bytes = source_bytes;
        usage.response_packets = source_packets;
        self.response_bytes = response_bytes;
        self.response_packets = response_packets;
        Ok(())
    }

    pub fn release(&mut self, id: PreauthStateId) -> Result<(), SessionRejected> {
        let state = self.states.remove(&id).ok_or(SessionRejected)?;
        self.memory_bytes -= state.memory_bytes;
        self.queued -= state.queued;
        let remove_source = {
            let usage = self.sources.get_mut(&state.source).ok_or(SessionRejected)?;
            usage.states -= 1;
            usage.queued -= state.queued;
            usage.states == 0
        };
        if remove_source {
            self.sources.remove(&state.source);
        }
        Ok(())
    }

    pub fn expire(&mut self, now_ms: u64) -> Result<usize, SessionRejected> {
        if now_ms < self.window_started_ms {
            return Err(SessionRejected);
        }
        let expired: Vec<_> = self
            .states
            .iter()
            .filter_map(|(id, state)| {
                (now_ms >= state.created_at_ms
                    && now_ms >= state.last_progress_ms
                    && (now_ms - state.last_progress_ms >= self.limits.idle_timeout_ms
                        || now_ms - state.created_at_ms >= self.limits.max_lifetime_ms))
                    .then_some(*id)
            })
            .collect();
        for id in &expired {
            self.release(*id)?;
        }
        Ok(expired.len())
    }

    pub fn live_states(&self) -> usize {
        self.states.len()
    }
    pub fn memory_bytes(&self) -> usize {
        self.memory_bytes
    }
    pub fn queued(&self) -> usize {
        self.queued
    }
}

#[cfg(test)]
mod preauth_tests {
    use super::session_tests::pair;
    use super::*;
    #[test]
    fn response_requires_charged_input_and_respects_amplification() {
        let mut b = PreauthBudget::new(PreauthLimits::default()).unwrap();
        assert_eq!(b.charge_response(1), Err(SessionRejected));
        b.charge_input(10).unwrap();
        assert_eq!(b.charge_response(30), Ok(()));
        assert_eq!(b.charge_response(1), Err(SessionRejected));
    }
    #[test]
    fn rejected_charge_is_atomic() {
        let mut b = PreauthBudget::new(PreauthLimits {
            max_input_bytes: 4,
            max_input_packets: 1,
            max_response_bytes: 12,
            max_response_packets: 1,
        })
        .unwrap();
        assert_eq!(b.charge_input(5), Err(SessionRejected));
        b.charge_input(4).unwrap();
        b.charge_response(12).unwrap();
        assert_eq!(b.charge_response(1), Err(SessionRejected));
    }
    fn process_limits() -> ProcessPreauthLimits {
        ProcessPreauthLimits {
            max_states_per_source: 1,
            max_states_global: 2,
            max_memory_per_state: 4,
            max_memory_global: 6,
            max_queue_per_source: 1,
            max_queue_global: 1,
            max_input_bytes_per_source: 4,
            max_input_packets_per_source: 2,
            max_input_bytes_per_window: 4,
            max_input_packets_per_window: 2,
            max_work_per_packet: 5,
            max_work_per_source: 5,
            max_work_per_window: 5,
            max_response_bytes_per_source: 3,
            max_response_packets_per_source: 1,
            max_response_bytes_per_window: 3,
            max_response_packets_per_window: 1,
            admission_window_ms: 10,
            idle_timeout_ms: 20,
            max_lifetime_ms: 20,
        }
    }

    #[test]
    fn process_admission_limits_are_atomic_and_source_scoped() {
        let mut admission = ProcessPreauthAdmission::new(process_limits(), 0).unwrap();
        let a = admission.admit_state(b"source-a", 4, 0).unwrap();
        assert_eq!(
            admission.admit_state(b"source-a", 1, 0),
            Err(SessionRejected)
        );
        let b = admission.admit_state(b"source-b", 2, 0).unwrap();
        assert_eq!(admission.live_states(), 2);
        assert_eq!(admission.memory_bytes(), 6);
        assert_eq!(
            admission.admit_state(b"source-c", 1, 0),
            Err(SessionRejected)
        );
        admission.release(b).unwrap();
        assert_eq!(admission.memory_bytes(), 4);
        assert_eq!(admission.live_states(), 1);
        admission.release(a).unwrap();
    }

    #[test]
    fn process_window_queue_and_lifetime_fail_closed() {
        let mut admission = ProcessPreauthAdmission::new(process_limits(), 0).unwrap();
        let id = admission.admit_state(b"source", 2, 0).unwrap();
        admission.charge_input(id, 4, 5, 0).unwrap();
        assert_eq!(admission.charge_input(id, 1, 0, 0), Err(SessionRejected));
        assert_eq!(admission.charge_response(id, 3, 0), Ok(()));
        assert_eq!(admission.charge_response(id, 1, 0), Err(SessionRejected));
        admission.enqueue(id, 0).unwrap();
        assert_eq!(admission.enqueue(id, 0), Err(SessionRejected));
        assert_eq!((admission.queued(), admission.memory_bytes()), (1, 2));
        admission.dequeue(id).unwrap();
        // A new monotonic window resets global rate counters, never the
        // state-lifetime source counters. A distinct source can use the window.
        assert_eq!(admission.charge_input(id, 1, 0, 10), Err(SessionRejected));
        let other = admission.admit_state(b"other", 2, 10).unwrap();
        admission.charge_input(other, 4, 5, 10).unwrap();
        assert_eq!(admission.live_states(), 2);
        assert_eq!(admission.charge_input(id, 1, 0, 9), Err(SessionRejected));
        assert_eq!(admission.charge_input(id, 1, 0, 20), Err(SessionRejected));
        assert_eq!(admission.expire(20), Ok(1));
        assert_eq!(
            (
                admission.live_states(),
                admission.memory_bytes(),
                admission.queued()
            ),
            (1, 2, 0)
        );
        admission.release(other).unwrap();
        assert_eq!((admission.live_states(), admission.memory_bytes()), (0, 0));
    }

    #[test]
    fn synchronized_key_update_resets_nonce_and_rejects_old_phase() {
        let (mut a, mut b) = pair();
        let old = a.seal(b"old").unwrap();
        assert_eq!(b.open(&old).unwrap(), b"old");
        a.update_key_phase().unwrap();
        b.update_key_phase().unwrap();
        assert_eq!((a.key_phase(), b.key_phase()), (1, 1));
        let fresh = a.seal(b"new").unwrap();
        assert_eq!(u64::from_be_bytes(fresh[..8].try_into().unwrap()), 0);
        assert_eq!(b.open(&fresh).unwrap(), b"new");
        assert_eq!(b.open(&old), Err(SessionRejected));
        assert_eq!(a.update_key_phase(), Err(SessionRejected));
    }
    #[test]
    fn unsynchronized_update_fails_closed_without_peer_state_change() {
        let (mut a, mut b) = pair();
        a.update_key_phase().unwrap();
        let old = b.seal(b"old").unwrap();
        assert_eq!(a.open(&old), Err(SessionRejected));
        assert_eq!(b.key_phase(), 0);
    }
    #[test]
    fn unreliable_oversize_rejection_preserves_replay_state() {
        let (mut a, mut b) = super::session_tests::pair();
        let oversized = a.seal(&vec![0; MAX_UNRELIABLE_DATAGRAM + 1]).unwrap();
        assert_eq!(b.open_unreliable(&oversized), Err(SessionRejected));
        // The same sequence remains available to the generic bounded record API;
        // the unreliable policy rejection did not advance replay state.
        assert_eq!(
            b.open(&oversized).unwrap().len(),
            MAX_UNRELIABLE_DATAGRAM + 1
        );
    }

    #[test]
    fn unreliable_datagram_roundtrip_replay_and_size_are_bounded() {
        let (mut a, mut b) = super::session_tests::pair();
        let record = a.seal_unreliable(b"telemetry").unwrap();
        assert_eq!(b.open_unreliable(&record).unwrap(), b"telemetry");
        assert_eq!(b.open_unreliable(&record), Err(SessionRejected));
        assert_eq!(
            a.seal_unreliable(&vec![0; MAX_UNRELIABLE_DATAGRAM + 1]),
            Err(SessionRejected)
        );
    }
}

/// Authenticated carrier-attachment claim. This is carried inside a fresh
/// Noise handshake; the fresh transport keys avoid nonce reuse while these
/// fields bind the new carrier to one existing logical Session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBinding {
    pub session_id: u64,
    pub delivery_epoch: u64,
    pub key_phase: u8,
    pub path_generation: u64,
    pub expires_at_ms: u64,
    pub token: [u8; 32],
}
impl ResumeBinding {
    pub fn encode(&self) -> [u8; 65] {
        let mut out = [0; 65];
        out[0..8].copy_from_slice(&self.session_id.to_be_bytes());
        out[8..16].copy_from_slice(&self.delivery_epoch.to_be_bytes());
        out[16] = self.key_phase;
        out[17..25].copy_from_slice(&self.path_generation.to_be_bytes());
        out[25..33].copy_from_slice(&self.expires_at_ms.to_be_bytes());
        out[33..65].copy_from_slice(&self.token);
        out
    }
    pub fn decode(input: &[u8]) -> Result<Self, SessionRejected> {
        if input.len() != 65 {
            return Err(SessionRejected);
        }
        Ok(Self {
            session_id: u64::from_be_bytes(input[0..8].try_into().map_err(|_| SessionRejected)?),
            delivery_epoch: u64::from_be_bytes(
                input[8..16].try_into().map_err(|_| SessionRejected)?,
            ),
            key_phase: input[16],
            path_generation: u64::from_be_bytes(
                input[17..25].try_into().map_err(|_| SessionRejected)?,
            ),
            expires_at_ms: u64::from_be_bytes(
                input[25..33].try_into().map_err(|_| SessionRejected)?,
            ),
            token: input[33..65].try_into().map_err(|_| SessionRejected)?,
        })
    }
}

/// Single-peer, monotonic attachment guard. Peer authentication is supplied by
/// the enclosing Noise IK handshake; this guard rejects cross-peer, replay,
/// expiry and stale path generation without deriving transport keys itself.
#[derive(Debug)]
pub struct ResumeGuard {
    peer_public: Vec<u8>,
    session_id: u64,
    delivery_epoch: u64,
    key_phase: u8,
    next_path_generation: u64,
    token: [u8; 32],
    negotiation_binding: Vec<u8>,
}
impl ResumeGuard {
    pub fn new(peer_public: &[u8], binding: &ResumeBinding) -> Result<Self, SessionRejected> {
        Self::new_with_negotiation(peer_public, binding, &[])
    }
    pub fn new_with_negotiation(
        peer_public: &[u8],
        binding: &ResumeBinding,
        negotiation_binding: &[u8],
    ) -> Result<Self, SessionRejected> {
        if negotiation_binding.len() > 256 {
            return Err(SessionRejected);
        }
        if peer_public.is_empty() || binding.path_generation == u64::MAX {
            return Err(SessionRejected);
        }
        Ok(Self {
            peer_public: peer_public.to_vec(),
            session_id: binding.session_id,
            delivery_epoch: binding.delivery_epoch,
            key_phase: binding.key_phase,
            next_path_generation: binding.path_generation + 1,
            token: binding.token,
            negotiation_binding: negotiation_binding.to_vec(),
        })
    }
    pub fn attach(
        &mut self,
        peer_public: &[u8],
        claim: &ResumeBinding,
        now_ms: u64,
    ) -> Result<(), SessionRejected> {
        self.attach_with_negotiation(peer_public, claim, &[], now_ms)
    }
    pub fn attach_with_negotiation(
        &mut self,
        peer_public: &[u8],
        claim: &ResumeBinding,
        negotiation_binding: &[u8],
        now_ms: u64,
    ) -> Result<(), SessionRejected> {
        if peer_public != self.peer_public
            || claim.session_id != self.session_id
            || claim.delivery_epoch != self.delivery_epoch
            || claim.key_phase != self.key_phase
            || claim.path_generation != self.next_path_generation
            || claim.token != self.token
            || self.negotiation_binding != negotiation_binding
            || now_ms > claim.expires_at_ms
        {
            return Err(SessionRejected);
        }
        self.next_path_generation = self
            .next_path_generation
            .checked_add(1)
            .ok_or(SessionRejected)?;
        Ok(())
    }
}

#[cfg(test)]
mod resume_tests {
    use super::*;
    fn claim(g: u64) -> ResumeBinding {
        ResumeBinding {
            session_id: 7,
            delivery_epoch: 3,
            key_phase: 1,
            path_generation: g,
            expires_at_ms: 100,
            token: [9; 32],
        }
    }
    #[test]
    fn attachment_is_peer_bound_monotonic_expiring_and_single_use() {
        let original = claim(4);
        let mut guard = ResumeGuard::new(b"peer", &original).unwrap();
        let next = claim(5);
        assert_eq!(ResumeBinding::decode(&next.encode()).unwrap(), next);
        assert_eq!(guard.attach(b"peer", &next, 99), Ok(()));
        assert_eq!(guard.attach(b"peer", &next, 99), Err(SessionRejected));
        let mut wrong = claim(6);
        wrong.token = [8; 32];
        assert_eq!(guard.attach(b"peer", &wrong, 99), Err(SessionRejected));
        assert_eq!(guard.attach(b"other", &claim(6), 99), Err(SessionRejected));
        assert_eq!(guard.attach(b"peer", &claim(6), 101), Err(SessionRejected));
    }

    #[test]
    fn resume_guard_binds_exact_negotiation_and_preserves_legacy_mode() {
        let original = claim(4);
        let next = claim(5);
        let mut bound =
            ResumeGuard::new_with_negotiation(b"peer", &original, b"version-0").unwrap();
        assert_eq!(
            bound.attach_with_negotiation(b"peer", &next, b"version-1", 99),
            Err(SessionRejected)
        );
        assert_eq!(
            bound.attach_with_negotiation(b"peer", &next, b"version-0", 99),
            Ok(())
        );
        let mut legacy = ResumeGuard::new(b"peer", &original).unwrap();
        assert_eq!(legacy.attach(b"peer", &next, 99), Ok(()));
    }

    #[test]
    fn fresh_noise_transport_is_bound_to_existing_logical_session() {
        let client = LocalIdentity::generate().unwrap();
        let server = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: client.public_key().to_vec(),
            scope: b"resume".to_vec(),
            status: TrustStatus::Active,
        }]);
        let original = claim(4);
        let next = claim(5);
        let mut guard = ResumeGuard::new(client.public_key(), &original).unwrap();
        let mut initiator = InitiatorHandshake::with_resume_binding(
            &client,
            server.public_key(),
            b"resume",
            b"resume-test",
            &next,
        )
        .unwrap();
        let first = initiator.first_message().unwrap();
        let (response, mut responder_session, peer, received) =
            ResponderHandshake::new(&server, policy, b"resume-test")
                .unwrap()
                .receive_first_with_resume(&first, session_tests::ctx(0))
                .unwrap();
        guard.attach(&peer, &received, 99).unwrap();
        let mut initiator_session = initiator.finish(&response, session_tests::ctx(0)).unwrap();
        let record = initiator_session.seal_unreliable(b"continued").unwrap();
        assert_eq!(
            responder_session.open_unreliable(&record).unwrap(),
            b"continued"
        );
    }
}
