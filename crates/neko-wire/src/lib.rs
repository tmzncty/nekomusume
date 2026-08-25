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
