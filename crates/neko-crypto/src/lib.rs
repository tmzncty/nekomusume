//! Bounded cryptographic/session primitives for local research.
//!
//! This crate deliberately exposes no listener, runtime, key loading, or
//! production configuration.  It provides the small fail-closed state pieces
//! that a future Noise session must compose.

use snow::params::NoiseParams;

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
pub const RECORD_CONTEXT_LEN: usize = 26;
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

pub struct InitiatorHandshake {
    state: snow::HandshakeState,
    scope: Vec<u8>,
}
pub struct ResponderHandshake {
    state: snow::HandshakeState,
    policy: TrustPolicy,
}

impl InitiatorHandshake {
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
    fn ctx(direction: u8) -> RecordContext {
        RecordContext {
            delivery_epoch: 7,
            key_phase: 0,
            path_generation: 3,
            stream_id: 11,
            direction,
        }
    }
    fn pair() -> (SecureSession, SecureSession) {
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
}

#[cfg(test)]
mod preauth_tests {
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
}
