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
