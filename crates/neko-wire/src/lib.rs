//! Candidate, synchronous wire codec for the M0 boundary.
//!
//! This module is deliberately transport-, runtime-, and cryptography-free. The
//! format is a candidate for review, not a frozen v0 protocol specification.

/// Two-byte candidate record marker.
pub const MAGIC: [u8; 2] = *b"NK";
/// Candidate wire version.
pub const VERSION: u8 = 0;
/// Header size: magic, version, type, flags, and big-endian payload length.
pub const HEADER_LEN: usize = 9;
/// Maximum payload accepted by the candidate decoder.
pub const MAX_PAYLOAD_LEN: usize = 4096;
/// Maximum bytes in a canonical unsigned integer.
pub const MAX_VARINT_BYTES: usize = 10;

/// Candidate record kinds. Values are provisional and intentionally small.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    Data = 1,
    Ack = 2,
    PathChallenge = 3,
}

impl RecordType {
    fn from_byte(value: u8) -> Result<Self, DecodeError> {
        match value {
            1 => Ok(Self::Data),
            2 => Ok(Self::Ack),
            3 => Ok(Self::PathChallenge),
            other => Err(DecodeError::UnknownType(other)),
        }
    }
}

/// A candidate record with an opaque payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub record_type: RecordType,
    pub flags: u8,
    pub payload: Vec<u8>,
}

/// Candidate authenticated SessionRecord frame type. The outer NK header remains
/// unchanged; this layer is the authenticated payload grammar.
const FRAME_IGNORABLE: u8 = 0x01;
const FRAME_RESERVED_MASK: u8 = 0xf0;
const FRAME_DATA: u8 = 0x00;
const FRAME_DATAGRAM: u8 = 0x0c;
const FRAME_DELIVERY_ACK: u8 = 0x02;
const FRAME_CLOSE: u8 = 0x04;
const FRAME_PATH_CHALLENGE: u8 = 0x06;
const FRAME_PATH_RESPONSE: u8 = 0x08;
const FRAME_HEADER_LEN: usize = 3;
const MAX_FRAME_PAYLOAD_LEN: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Data(Vec<u8>),
    Datagram(Vec<u8>),
    DeliveryAck(Vec<u8>),
    Close(Vec<u8>),
    PathChallenge([u8; 8]),
    PathResponse([u8; 8]),
    UnknownIgnorable { frame_type: u8, payload: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    Empty,
    Truncated,
    LengthTooLarge(usize),
    InvalidLength { frame_type: u8, length: usize },
    UnknownCritical(u8),
    ReservedType(u8),
    TooManyFrames,
}

fn frame_type(frame: &Frame) -> u8 {
    match frame {
        Frame::Data(_) => FRAME_DATA,
        Frame::Datagram(_) => FRAME_DATAGRAM,
        Frame::DeliveryAck(_) => FRAME_DELIVERY_ACK,
        Frame::Close(_) => FRAME_CLOSE,
        Frame::PathChallenge(_) => FRAME_PATH_CHALLENGE,
        Frame::PathResponse(_) => FRAME_PATH_RESPONSE,
        Frame::UnknownIgnorable { frame_type, .. } => *frame_type,
    }
}
fn frame_payload(frame: &Frame) -> &[u8] {
    match frame {
        Frame::Data(p) | Frame::Datagram(p) | Frame::DeliveryAck(p) | Frame::Close(p) => p,
        Frame::PathChallenge(p) | Frame::PathResponse(p) => p,
        Frame::UnknownIgnorable { payload, .. } => payload,
    }
}
fn validate_frame_type(t: u8, len: usize) -> Result<(), FrameError> {
    if t & FRAME_RESERVED_MASK == FRAME_RESERVED_MASK {
        return Err(FrameError::ReservedType(t));
    }
    match t & !FRAME_IGNORABLE {
        FRAME_DATA | FRAME_DATAGRAM | FRAME_DELIVERY_ACK | FRAME_CLOSE => {
            if len > MAX_FRAME_PAYLOAD_LEN {
                Err(FrameError::LengthTooLarge(len))
            } else {
                Ok(())
            }
        }
        FRAME_PATH_CHALLENGE | FRAME_PATH_RESPONSE => {
            if len == 8 {
                Ok(())
            } else {
                Err(FrameError::InvalidLength {
                    frame_type: t,
                    length: len,
                })
            }
        }
        _ if t & FRAME_IGNORABLE != 0 => Ok(()),
        _ => Err(FrameError::UnknownCritical(t)),
    }
}

/// Encode a bounded list of frames into the authenticated SessionRecord payload.
pub fn encode_frames(frames: &[Frame]) -> Result<Vec<u8>, FrameError> {
    if frames.is_empty() {
        return Err(FrameError::Empty);
    }
    if frames.len() > 64 {
        return Err(FrameError::TooManyFrames);
    }
    let mut out = Vec::new();
    for frame in frames {
        let t = frame_type(frame);
        let payload = frame_payload(frame);
        validate_frame_type(t, payload.len())?;
        let len =
            u16::try_from(payload.len()).map_err(|_| FrameError::LengthTooLarge(payload.len()))?;
        out.push(t);
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(payload);
        if out.len() > MAX_PAYLOAD_LEN {
            return Err(FrameError::LengthTooLarge(out.len()));
        }
    }
    Ok(out)
}

/// Decode all frames from one authenticated SessionRecord payload. Unknown
/// ignorable types are retained; unknown critical and reserved types fail closed.
pub fn decode_frames(mut input: &[u8]) -> Result<Vec<Frame>, FrameError> {
    if input.is_empty() {
        return Err(FrameError::Empty);
    }
    let mut frames = Vec::new();
    while !input.is_empty() {
        if frames.len() >= 64 {
            return Err(FrameError::TooManyFrames);
        }
        if input.len() < FRAME_HEADER_LEN {
            return Err(FrameError::Truncated);
        }
        let t = input[0];
        let len = u16::from_be_bytes([input[1], input[2]]) as usize;
        validate_frame_type(t, len)?;
        input = &input[FRAME_HEADER_LEN..];
        if input.len() < len {
            return Err(FrameError::Truncated);
        }
        let payload = &input[..len];
        input = &input[len..];
        let base = t & !FRAME_IGNORABLE;
        let frame = match base {
            FRAME_DATA => Frame::Data(payload.to_vec()),
            FRAME_DATAGRAM => Frame::Datagram(payload.to_vec()),
            FRAME_DELIVERY_ACK => Frame::DeliveryAck(payload.to_vec()),
            FRAME_CLOSE => Frame::Close(payload.to_vec()),
            FRAME_PATH_CHALLENGE => {
                Frame::PathChallenge(payload.try_into().map_err(|_| FrameError::InvalidLength {
                    frame_type: t,
                    length: len,
                })?)
            }
            FRAME_PATH_RESPONSE => {
                Frame::PathResponse(payload.try_into().map_err(|_| FrameError::InvalidLength {
                    frame_type: t,
                    length: len,
                })?)
            }
            _ => Frame::UnknownIgnorable {
                frame_type: t,
                payload: payload.to_vec(),
            },
        };
        frames.push(frame);
    }
    Ok(frames)
}

/// Errors emitted deterministically by candidate decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    BadMagic,
    UnknownVersion(u8),
    UnknownType(u8),
    InvalidFlags(u8),
    LengthTooLarge(u32),
    LengthMismatch { declared: usize, available: usize },
    TrailingBytes(usize),
    NonCanonicalInteger,
    IntegerOverflow,
}

/// Encode an unsigned integer as minimal, little-endian base-128 bytes.
pub fn encode_varint(mut value: u64, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            return;
        }
    }
}

/// Decode one canonical unsigned little-endian base-128 integer.
pub fn decode_varint(input: &[u8]) -> Result<(u64, usize), DecodeError> {
    let mut value = 0u64;
    for (index, &byte) in input.iter().enumerate().take(MAX_VARINT_BYTES) {
        let shift = index * 7;
        let part = (byte & 0x7f) as u64;
        if shift == 63 && part > 1 {
            return Err(DecodeError::IntegerOverflow);
        }
        value |= part
            .checked_shl(shift as u32)
            .ok_or(DecodeError::IntegerOverflow)?;
        if byte & 0x80 == 0 {
            if index > 0 && part == 0 {
                return Err(DecodeError::NonCanonicalInteger);
            }
            return Ok((value, index + 1));
        }
    }
    if input.len() < MAX_VARINT_BYTES {
        Err(DecodeError::Truncated)
    } else {
        Err(DecodeError::IntegerOverflow)
    }
}

/// Encode one candidate record using big-endian fixed-width header fields.
pub fn encode(record: &Record) -> Result<Vec<u8>, EncodeError> {
    if record.flags != 0 {
        return Err(EncodeError::InvalidFlags(record.flags));
    }
    if record.payload.len() > MAX_PAYLOAD_LEN {
        return Err(EncodeError::PayloadTooLarge(record.payload.len()));
    }
    let length = u32::try_from(record.payload.len()).expect("candidate limit fits u32");
    let mut output = Vec::with_capacity(HEADER_LEN + record.payload.len());
    output.extend_from_slice(&MAGIC);
    output.push(VERSION);
    output.push(record.record_type as u8);
    output.push(record.flags);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(&record.payload);
    Ok(output)
}

/// Decode exactly one candidate record; no network or cryptographic work occurs.
pub fn decode(input: &[u8]) -> Result<Record, DecodeError> {
    if input.len() < HEADER_LEN {
        return Err(DecodeError::Truncated);
    }
    if input[..2] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    if input[2] != VERSION {
        return Err(DecodeError::UnknownVersion(input[2]));
    }
    let record_type = RecordType::from_byte(input[3])?;
    if input[4] != 0 {
        return Err(DecodeError::InvalidFlags(input[4]));
    }
    let declared = u32::from_be_bytes(input[5..9].try_into().expect("header checked"));
    if declared as usize > MAX_PAYLOAD_LEN {
        return Err(DecodeError::LengthTooLarge(declared));
    }
    let available = input.len() - HEADER_LEN;
    if available < declared as usize {
        return Err(DecodeError::LengthMismatch {
            declared: declared as usize,
            available,
        });
    }
    if available > declared as usize {
        return Err(DecodeError::TrailingBytes(available - declared as usize));
    }
    Ok(Record {
        record_type,
        flags: 0,
        payload: input[HEADER_LEN..].to_vec(),
    })
}

/// Errors emitted while encoding a candidate record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    InvalidFlags(u8),
    PayloadTooLarge(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(kind: RecordType, payload: &[u8]) -> Record {
        Record {
            record_type: kind,
            flags: 0,
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn golden_vectors_cover_empty_and_boundary_records() {
        let vectors = [
            (RecordType::Data, vec![]),
            (RecordType::Ack, vec![0]),
            (RecordType::PathChallenge, vec![0xff]),
            (RecordType::Data, vec![1, 2]),
            (RecordType::Ack, vec![0xaa; 3]),
            (RecordType::PathChallenge, vec![0x55; 4]),
            (RecordType::Data, vec![0; 5]),
            (RecordType::Ack, vec![0xff; 6]),
            (RecordType::PathChallenge, vec![7; 7]),
            (RecordType::Data, vec![8; 8]),
            (RecordType::Ack, vec![9; 9]),
            (RecordType::PathChallenge, vec![10; 10]),
            (RecordType::Data, vec![0x10; 16]),
            (RecordType::Ack, vec![0x20; 31]),
            (RecordType::PathChallenge, vec![0x30; 32]),
            (RecordType::Data, vec![0x40; 63]),
            (RecordType::Ack, vec![0x50; 64]),
            (RecordType::PathChallenge, vec![0x60; 127]),
            (RecordType::Data, vec![0x70; 128]),
            (RecordType::Ack, vec![0x80; 255]),
            (RecordType::PathChallenge, vec![0x90; 1024]),
            (RecordType::Data, vec![0xa0; MAX_PAYLOAD_LEN]),
        ];
        assert!(vectors.len() >= 20);
        for (kind, payload) in vectors {
            let encoded = encode(&record(kind, &payload)).unwrap();
            assert_eq!(decode(&encoded).unwrap(), record(kind, &payload));
        }
    }

    #[test]
    fn golden_varints_are_minimal() {
        for (value, expected) in [
            (0, vec![0]),
            (1, vec![1]),
            (127, vec![127]),
            (128, vec![0x80, 1]),
            (16_384, vec![0x80, 0x80, 1]),
            (u64::MAX, vec![0xff; 9].into_iter().chain([1]).collect()),
        ] {
            let mut actual = Vec::new();
            encode_varint(value, &mut actual);
            assert_eq!(actual, expected);
            assert_eq!(decode_varint(&actual), Ok((value, actual.len())));
        }
    }

    #[test]
    fn decode_rejects_malformed_inputs() {
        assert_eq!(decode(&[]), Err(DecodeError::Truncated));
        assert_eq!(decode(b"badbadbad"), Err(DecodeError::BadMagic));
        let mut bytes = encode(&record(RecordType::Data, &[])).unwrap();
        bytes[2] = 1;
        assert_eq!(decode(&bytes), Err(DecodeError::UnknownVersion(1)));
        bytes[2] = VERSION;
        bytes[3] = 99;
        assert_eq!(decode(&bytes), Err(DecodeError::UnknownType(99)));
        bytes[3] = RecordType::Data as u8;
        bytes[4] = 1;
        assert_eq!(decode(&bytes), Err(DecodeError::InvalidFlags(1)));
        bytes[4] = 0;
        bytes[5..9].copy_from_slice(&(MAX_PAYLOAD_LEN as u32 + 1).to_be_bytes());
        assert_eq!(decode(&bytes), Err(DecodeError::LengthTooLarge(4097)));
        let mut short = encode(&record(RecordType::Data, &[1, 2])).unwrap();
        short.pop();
        assert_eq!(
            decode(&short),
            Err(DecodeError::LengthMismatch {
                declared: 2,
                available: 1
            })
        );
        let mut trailing = encode(&record(RecordType::Data, &[])).unwrap();
        trailing.push(0);
        assert_eq!(decode(&trailing), Err(DecodeError::TrailingBytes(1)));
        assert_eq!(decode_varint(&[0x80]), Err(DecodeError::Truncated));
        assert_eq!(
            decode_varint(&[0x80, 0]),
            Err(DecodeError::NonCanonicalInteger)
        );
        assert_eq!(
            decode_varint(&[0xff; 10]),
            Err(DecodeError::IntegerOverflow)
        );
    }

    #[test]
    fn decode_rejects_all_short_prefixes_without_panic_or_evidence() {
        for len in 0..=7 {
            let input = vec![0x4e; len];
            assert!(decode(&input).is_err(), "length {len} unexpectedly decoded");
        }
    }
    #[test]
    fn decode_rejects_overflow_and_noncanonical_varints() {
        let cases = [
            vec![
                b'N', b'K', 1, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0x02,
            ],
            vec![b'N', b'K', 1, 0, 0x80, 0x00],
        ];
        for input in cases {
            assert!(decode(&input).is_err());
        }
    }

    #[test]
    fn encode_rejects_flags_and_oversized_payloads() {
        assert_eq!(
            encode(&Record {
                record_type: RecordType::Data,
                flags: 1,
                payload: vec![]
            }),
            Err(EncodeError::InvalidFlags(1))
        );
        assert_eq!(
            encode(&record(RecordType::Data, &vec![0; MAX_PAYLOAD_LEN + 1])),
            Err(EncodeError::PayloadTooLarge(MAX_PAYLOAD_LEN + 1))
        );
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    #[test]
    fn multi_frame_roundtrip_and_unknown_compatibility() {
        let frames = vec![
            Frame::Data(b"data".to_vec()),
            Frame::DeliveryAck(vec![1, 2]),
            Frame::PathChallenge(*b"12345678"),
            Frame::UnknownIgnorable {
                frame_type: 0x11,
                payload: vec![9],
            },
            Frame::Close(vec![]),
        ];
        assert_eq!(
            decode_frames(&encode_frames(&frames).unwrap()).unwrap(),
            frames
        );
    }
    #[test]
    fn critical_unknown_and_reserved_types_fail_closed() {
        assert_eq!(
            decode_frames(&[0x0a, 0, 0]),
            Err(FrameError::UnknownCritical(0x0a))
        );
        assert_eq!(
            decode_frames(&[0xf1, 0, 0]),
            Err(FrameError::ReservedType(0xf1))
        );
    }
    #[test]
    fn frame_bounds_and_malformed_lengths_are_deterministic() {
        assert_eq!(encode_frames(&[]), Err(FrameError::Empty));
        assert_eq!(decode_frames(&[FRAME_DATA, 0]), Err(FrameError::Truncated));
        assert_eq!(
            decode_frames(&[FRAME_PATH_CHALLENGE, 0, 7, 0, 0, 0, 0, 0, 0, 0]),
            Err(FrameError::InvalidLength {
                frame_type: FRAME_PATH_CHALLENGE,
                length: 7
            })
        );
        assert_eq!(
            decode_frames(&[FRAME_DATA, 0, 4, 1]),
            Err(FrameError::Truncated)
        );
        assert_eq!(
            encode_frames(&[Frame::Data(vec![0; MAX_FRAME_PAYLOAD_LEN + 1])]),
            Err(FrameError::LengthTooLarge(MAX_FRAME_PAYLOAD_LEN + 1))
        );
    }
}

#[cfg(test)]
mod datagram_frame_tests {
    use super::*;
    #[test]
    fn datagram_vector_is_deterministic_and_bounded() {
        let encoded = encode_frames(&[Frame::Datagram(b"ping".to_vec())]).unwrap();
        assert_eq!(encoded, vec![0x0c, 0, 4, b'p', b'i', b'n', b'g']);
        assert_eq!(
            decode_frames(&encoded),
            Ok(vec![Frame::Datagram(b"ping".to_vec())])
        );
        assert!(encode_frames(&[Frame::Datagram(vec![0; 1025])]).is_err());
    }
    #[test]
    fn datagram_truncation_is_rejected() {
        assert_eq!(decode_frames(&[0x0c, 0, 4, 1]), Err(FrameError::Truncated));
    }
}

/// Maximum number of versions in a negotiation message. This is deliberately
/// small: peer-controlled counts must never drive unbounded allocation.
pub const MAX_NEGOTIATION_VERSIONS: usize = 16;
/// The first explicitly negotiated protocol version.
pub const NEGOTIATION_VERSION: u16 = 0;
/// Domain separator for the exact authenticated negotiation transcript.
pub const NEGOTIATION_BINDING_DOMAIN: &[u8] = b"nekomusume/version-negotiation/v1\0";
const NEGOTIATION_MAGIC: [u8; 2] = *b"N1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationRole {
    Client,
    Server,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationState {
    Awaiting,
    Established(u16),
    Rejected,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationError {
    EmptySupported,
    TooManySupported,
    UnsortedSupported,
    DuplicateSupported(u16),
    Malformed,
    TooManyOffered,
    DuplicateOffered(u16),
    NoCompatibleVersion,
    UnexpectedMessage,
    LateMessage,
    UnsupportedSelected(u16),
}

/// A bounded, explicit version handshake. It is transport-independent and is
/// intended to run before any Session data is accepted or emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionNegotiator {
    role: NegotiationRole,
    supported: Vec<u16>,
    state: NegotiationState,
    // The accepted hello and response are retained for loss recovery.
    accepted_hello: Option<Vec<u8>>,
    accepted_response: Option<Vec<u8>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationBinding(Vec<u8>);
impl NegotiationBinding {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl VersionNegotiator {
    pub fn new(role: NegotiationRole, supported: &[u16]) -> Result<Self, NegotiationError> {
        if supported.is_empty() {
            return Err(NegotiationError::EmptySupported);
        }
        if supported.len() > MAX_NEGOTIATION_VERSIONS {
            return Err(NegotiationError::TooManySupported);
        }
        for pair in supported.windows(2) {
            if pair[0] >= pair[1] {
                return Err(if pair[0] == pair[1] {
                    NegotiationError::DuplicateSupported(pair[0])
                } else {
                    NegotiationError::UnsortedSupported
                });
            }
        }
        Ok(Self {
            role,
            supported: supported.to_vec(),
            state: NegotiationState::Awaiting,
            accepted_hello: None,
            accepted_response: None,
        })
    }
    pub fn state(&self) -> NegotiationState {
        self.state
    }
    pub fn is_established(&self) -> bool {
        matches!(self.state, NegotiationState::Established(_))
    }
    pub fn selected(&self) -> Option<u16> {
        match self.state {
            NegotiationState::Established(v) => Some(v),
            _ => None,
        }
    }
    /// Client's first flight. Calling it after negotiation is a deterministic rejection.
    pub fn client_hello(&self) -> Result<Vec<u8>, NegotiationError> {
        if self.role != NegotiationRole::Client {
            return Err(NegotiationError::UnexpectedMessage);
        }
        if self.state != NegotiationState::Awaiting {
            return Err(NegotiationError::LateMessage);
        }
        encode_hello(&self.supported)
    }
    /// Server consumes exactly one client hello and returns a selected-version response.
    pub fn server_accept_hello(&mut self, input: &[u8]) -> Result<Vec<u8>, NegotiationError> {
        if self.role != NegotiationRole::Server {
            return Err(NegotiationError::UnexpectedMessage);
        }
        if self.state != NegotiationState::Awaiting {
            if matches!(self.state, NegotiationState::Established(_))
                && self.accepted_hello.as_deref() == Some(input)
            {
                return Ok(self.accepted_response.clone().expect("accepted response"));
            }
            return Err(NegotiationError::LateMessage);
        }
        let offered = match decode_hello(input) {
            Ok(offered) => offered,
            Err(error) => {
                self.state = NegotiationState::Rejected;
                return Err(error);
            }
        };
        let selected = self
            .supported
            .iter()
            .rev()
            .find(|v| offered.binary_search(v).is_ok())
            .copied();
        let Some(selected) = selected else {
            self.state = NegotiationState::Rejected;
            return Err(NegotiationError::NoCompatibleVersion);
        };
        self.state = NegotiationState::Established(selected);
        let response = encode_response(selected);
        self.accepted_hello = Some(input.to_vec());
        self.accepted_response = Some(response.clone());
        Ok(response)
    }
    /// Client consumes exactly one server response.
    pub fn client_accept_response(&mut self, input: &[u8]) -> Result<u16, NegotiationError> {
        if self.role != NegotiationRole::Client {
            return Err(NegotiationError::UnexpectedMessage);
        }
        if self.state != NegotiationState::Awaiting {
            return Err(NegotiationError::LateMessage);
        }
        let selected = match decode_response(input) {
            Ok(selected) => selected,
            Err(error) => {
                self.state = NegotiationState::Rejected;
                return Err(error);
            }
        };
        if self.supported.binary_search(&selected).is_err() {
            self.state = NegotiationState::Rejected;
            return Err(NegotiationError::UnsupportedSelected(selected));
        }
        self.state = NegotiationState::Established(selected);
        self.accepted_hello = Some(encode_hello(&self.supported)?);
        self.accepted_response = Some(input.to_vec());
        Ok(selected)
    }
    /// Admission guard for the wire/session boundary: data is forbidden pre-negotiation.
    pub fn admit_data(&self) -> Result<u16, NegotiationError> {
        self.selected().ok_or(NegotiationError::UnexpectedMessage)
    }
    /// Exact canonical transcript committed to the authenticated transport.
    pub fn authenticated_binding(&self) -> Result<NegotiationBinding, NegotiationError> {
        let selected = self.admit_data()?;
        let hello = self
            .accepted_hello
            .as_deref()
            .ok_or(NegotiationError::UnexpectedMessage)?;
        let response = self
            .accepted_response
            .as_deref()
            .ok_or(NegotiationError::UnexpectedMessage)?;
        let mut out =
            Vec::with_capacity(NEGOTIATION_BINDING_DOMAIN.len() + 6 + hello.len() + response.len());
        out.extend_from_slice(NEGOTIATION_BINDING_DOMAIN);
        out.extend_from_slice(&(hello.len() as u16).to_be_bytes());
        out.extend_from_slice(hello);
        out.extend_from_slice(&(response.len() as u16).to_be_bytes());
        out.extend_from_slice(response);
        out.extend_from_slice(&selected.to_be_bytes());
        Ok(NegotiationBinding(out))
    }
}
fn encode_hello(versions: &[u16]) -> Result<Vec<u8>, NegotiationError> {
    if versions.is_empty() || versions.len() > MAX_NEGOTIATION_VERSIONS {
        return Err(NegotiationError::TooManyOffered);
    }
    let mut out = Vec::with_capacity(4 + versions.len() * 2);
    out.extend_from_slice(&NEGOTIATION_MAGIC);
    out.push(1);
    out.push(versions.len() as u8);
    for v in versions {
        out.extend_from_slice(&v.to_be_bytes());
    }
    Ok(out)
}
fn decode_hello(input: &[u8]) -> Result<Vec<u16>, NegotiationError> {
    if input.len() < 4 || input[..2] != NEGOTIATION_MAGIC || input[2] != 1 {
        return Err(NegotiationError::Malformed);
    }
    let count = input[3] as usize;
    if count == 0 || count > MAX_NEGOTIATION_VERSIONS || input.len() != 4 + count * 2 {
        return Err(if count > MAX_NEGOTIATION_VERSIONS {
            NegotiationError::TooManyOffered
        } else {
            NegotiationError::Malformed
        });
    }
    let mut versions = Vec::with_capacity(count);
    let bytes = &input[4..];
    let mut index = 0;
    while index < bytes.len() {
        let v = u16::from_be_bytes([bytes[index], bytes[index + 1]]);
        index += 2;
        if versions.last().is_some_and(|last| *last == v) {
            return Err(NegotiationError::DuplicateOffered(v));
        }
        if versions.last().is_some_and(|last| *last > v) {
            return Err(NegotiationError::Malformed);
        }
        versions.push(v);
    }
    Ok(versions)
}
fn encode_response(version: u16) -> Vec<u8> {
    vec![
        NEGOTIATION_MAGIC[0],
        NEGOTIATION_MAGIC[1],
        2,
        0,
        (version >> 8) as u8,
        version as u8,
    ]
}
fn decode_response(input: &[u8]) -> Result<u16, NegotiationError> {
    if input.len() != 6 || input[..2] != NEGOTIATION_MAGIC || input[2] != 2 || input[3] != 0 {
        return Err(NegotiationError::Malformed);
    }
    Ok(u16::from_be_bytes([input[4], input[5]]))
}

#[cfg(test)]
mod negotiation_tests {
    use super::*;
    fn pair() -> (VersionNegotiator, VersionNegotiator) {
        (
            VersionNegotiator::new(NegotiationRole::Client, &[0, 2]).unwrap(),
            VersionNegotiator::new(NegotiationRole::Server, &[0, 1]).unwrap(),
        )
    }
    #[test]
    fn established_peers_produce_identical_exact_bindings() {
        let (mut c, mut s) = pair();
        assert!(c.authenticated_binding().is_err());
        let hello = c.client_hello().unwrap();
        let response = s.server_accept_hello(&hello).unwrap();
        c.client_accept_response(&response).unwrap();
        let cb = c.authenticated_binding().unwrap();
        let sb = s.authenticated_binding().unwrap();
        assert_eq!(cb, sb);
        assert!(cb.as_bytes().windows(hello.len()).any(|w| w == hello));
        assert!(cb.as_bytes().windows(response.len()).any(|w| w == response));
    }
    #[test]
    fn same_selection_with_different_offer_has_different_binding() {
        let mut a = VersionNegotiator::new(NegotiationRole::Client, &[0]).unwrap();
        let mut b = VersionNegotiator::new(NegotiationRole::Client, &[0, 2]).unwrap();
        let response = encode_response(0);
        a.client_accept_response(&response).unwrap();
        b.client_accept_response(&response).unwrap();
        assert_ne!(
            a.authenticated_binding().unwrap(),
            b.authenticated_binding().unwrap()
        );
    }
    #[test]
    fn compatible_selects_highest_and_establishes_only_after_both_messages() {
        let (mut c, mut s) = pair();
        assert_eq!(c.admit_data(), Err(NegotiationError::UnexpectedMessage));
        let hello = c.client_hello().unwrap();
        let response = s.server_accept_hello(&hello).unwrap();
        assert_eq!(s.selected(), Some(0));
        assert_eq!(c.client_accept_response(&response), Ok(0));
        assert_eq!(c.admit_data(), Ok(0));
    }
    #[test]
    fn no_overlap_is_terminal_and_future_is_not_accepted() {
        let mut s = VersionNegotiator::new(NegotiationRole::Server, &[7]).unwrap();
        let h = encode_hello(&[8]).unwrap();
        assert_eq!(
            s.server_accept_hello(&h),
            Err(NegotiationError::NoCompatibleVersion)
        );
        assert_eq!(s.state(), NegotiationState::Rejected);
        assert_eq!(
            s.server_accept_hello(&encode_hello(&[7]).unwrap()),
            Err(NegotiationError::LateMessage)
        );
    }
    #[test]
    fn malformed_and_duplicate_are_deterministic_and_bounded() {
        let duplicate = [b'N', b'1', 1, 2, 0, 0, 0, 0];
        let truncated = [b'N', b'1', 1, 1, 0];
        for (input, expected) in [
            (&duplicate[..], NegotiationError::DuplicateOffered(0)),
            (&truncated[..], NegotiationError::Malformed),
        ] {
            let mut s = VersionNegotiator::new(NegotiationRole::Server, &[0]).unwrap();
            assert_eq!(s.server_accept_hello(input), Err(expected));
            assert_eq!(s.state(), NegotiationState::Rejected);
        }
        let mut oversized = vec![b'N', b'1', 1, 17];
        oversized.extend([0; 34]);
        let mut s = VersionNegotiator::new(NegotiationRole::Server, &[0]).unwrap();
        assert_eq!(
            s.server_accept_hello(&oversized),
            Err(NegotiationError::TooManyOffered)
        );
    }
    #[test]
    fn established_server_replays_exact_response_for_duplicate_hello_only() {
        let mut s = VersionNegotiator::new(NegotiationRole::Server, &[0, 1]).unwrap();
        let hello = encode_hello(&[0, 1]).unwrap();
        let response = s.server_accept_hello(&hello).unwrap();
        assert_eq!(response, encode_response(1));
        assert_eq!(s.server_accept_hello(&hello), Ok(response.clone()));
        assert_eq!(s.state(), NegotiationState::Established(1));
        assert_eq!(s.admit_data(), Ok(1));

        for late in [
            encode_hello(&[0]).unwrap(),
            vec![b'N', b'1', 1, 1, 0],
            encode_response(1),
            vec![0xff],
        ] {
            assert_eq!(
                s.server_accept_hello(&late),
                Err(NegotiationError::LateMessage)
            );
            assert_eq!(s.state(), NegotiationState::Established(1));
        }
    }
    #[test]
    fn duplicate_supported_future_response_and_late_messages_reject() {
        assert_eq!(
            VersionNegotiator::new(NegotiationRole::Client, &[1, 1]),
            Err(NegotiationError::DuplicateSupported(1))
        );
        let (mut c, mut s) = pair();
        let h = c.client_hello().unwrap();
        let r = s.server_accept_hello(&h).unwrap();
        assert_eq!(c.client_accept_response(&r), Ok(0));
        assert_eq!(
            c.client_accept_response(&r),
            Err(NegotiationError::LateMessage)
        );
        assert_eq!(
            VersionNegotiator::new(NegotiationRole::Client, &[0])
                .unwrap()
                .client_accept_response(&encode_response(9)),
            Err(NegotiationError::UnsupportedSelected(9))
        );
    }
}
