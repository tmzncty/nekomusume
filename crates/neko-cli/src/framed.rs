use std::io::{self, Read};
use std::time::Instant;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FrameRead {
    Complete(Vec<u8>),
    Deadline,
    CleanEof,
    Truncated,
}

#[derive(Debug)]
pub(crate) struct FramedReader {
    header: [u8; 4],
    header_len: usize,
    payload: Vec<u8>,
    payload_len: usize,
    max_frame_len: usize,
}

impl FramedReader {
    pub(crate) fn new(max_frame_len: usize) -> Self {
        Self {
            header: [0; 4],
            header_len: 0,
            payload: Vec::new(),
            payload_len: 0,
            max_frame_len,
        }
    }

    pub(crate) fn set_max_frame_len(&mut self, max_frame_len: usize) -> io::Result<()> {
        if self.is_partial() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot change frame limit with a partial frame",
            ));
        }
        self.max_frame_len = max_frame_len;
        Ok(())
    }

    pub(crate) fn read_until<R: Read>(
        &mut self,
        reader: &mut R,
        deadline: Instant,
    ) -> io::Result<FrameRead> {
        self.read_until_with_clock(reader, deadline, Instant::now)
    }

    fn read_until_with_clock<R: Read, N: FnMut() -> Instant>(
        &mut self,
        reader: &mut R,
        deadline: Instant,
        mut now: N,
    ) -> io::Result<FrameRead> {
        loop {
            if now() >= deadline {
                return Ok(if self.is_partial() {
                    FrameRead::Truncated
                } else {
                    FrameRead::Deadline
                });
            }
            let result = if self.header_len < self.header.len() {
                reader.read(&mut self.header[self.header_len..])
            } else {
                reader.read(&mut self.payload[self.payload_len..])
            };
            match result {
                Ok(0) => {
                    return Ok(if self.is_partial() {
                        FrameRead::Truncated
                    } else {
                        FrameRead::CleanEof
                    });
                }
                Ok(n) if self.header_len < self.header.len() => {
                    self.header_len += n;
                    if self.header_len == self.header.len() {
                        let len = u32::from_be_bytes(self.header) as usize;
                        if len > self.max_frame_len {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "frame too large",
                            ));
                        }
                        self.payload = vec![0; len];
                        self.payload_len = 0;
                        if len == 0 {
                            return Ok(FrameRead::Complete(self.take_frame()));
                        }
                    }
                }
                Ok(n) => {
                    self.payload_len += n;
                    if self.payload_len == self.payload.len() {
                        return Ok(FrameRead::Complete(self.take_frame()));
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => return Err(error),
            }
        }
    }

    fn is_partial(&self) -> bool {
        self.header_len != 0 || self.payload_len != 0 || !self.payload.is_empty()
    }

    fn take_frame(&mut self) -> Vec<u8> {
        self.header_len = 0;
        self.payload_len = 0;
        std::mem::take(&mut self.payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::time::Duration;

    enum Step {
        Bytes(Vec<u8>),
        Timeout,
        Eof,
    }

    struct ScriptedRead(VecDeque<Step>);

    impl Read for ScriptedRead {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            match self.0.pop_front().expect("script exhausted") {
                Step::Bytes(mut bytes) => {
                    let n = bytes.len().min(output.len());
                    output[..n].copy_from_slice(&bytes[..n]);
                    if n < bytes.len() {
                        bytes.drain(..n);
                        self.0.push_front(Step::Bytes(bytes));
                    }
                    Ok(n)
                }
                Step::Timeout => Err(io::Error::new(io::ErrorKind::TimedOut, "poll")),
                Step::Eof => Ok(0),
            }
        }
    }

    fn times(base: Instant, millis: &[u64]) -> impl FnMut() -> Instant {
        let mut values: VecDeque<_> = millis
            .iter()
            .map(|millis| base + Duration::from_millis(*millis))
            .collect();
        move || values.pop_front().unwrap_or(base + Duration::from_secs(60))
    }

    #[test]
    fn preserves_fragmented_header_and_payload_across_poll_timeouts() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(vec![0]),
            Step::Timeout,
            Step::Bytes(vec![0, 0]),
            Step::Timeout,
            Step::Bytes(vec![5]),
            Step::Bytes(b"ab".to_vec()),
            Step::Timeout,
            Step::Bytes(b"cde".to_vec()),
        ]));
        let mut framed = FramedReader::new(8);
        let result = framed
            .read_until_with_clock(
                &mut input,
                base + Duration::from_secs(1),
                times(base, &[0, 1, 2, 3, 4, 5, 6, 7]),
            )
            .unwrap();
        assert_eq!(result, FrameRead::Complete(b"abcde".to_vec()));
    }

    #[test]
    fn delayed_delivery_ack_fragments_remain_within_absolute_deadline() {
        let base = Instant::now();
        let ack = b"authenticated-delivery-ack";
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes((ack.len() as u32).to_be_bytes()[..2].to_vec()),
            Step::Timeout,
            Step::Bytes((ack.len() as u32).to_be_bytes()[2..].to_vec()),
            Step::Bytes(ack[..7].to_vec()),
            Step::Timeout,
            Step::Bytes(ack[7..].to_vec()),
        ]));
        let mut framed = FramedReader::new(64);
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(200),
                    times(base, &[0, 50, 51, 100, 150, 199]),
                )
                .unwrap(),
            FrameRead::Complete(ack.to_vec())
        );
    }

    #[test]
    fn rejects_oversize_before_payload_allocation() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(9u32.to_be_bytes().to_vec())]));
        let mut framed = FramedReader::new(8);
        let error = framed
            .read_until_with_clock(&mut input, base + Duration::from_secs(1), times(base, &[0]))
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(framed.payload.is_empty());
    }

    #[test]
    fn distinguishes_clean_eof_partial_header_partial_payload_and_idle_deadline() {
        let base = Instant::now();
        let deadline = base + Duration::from_millis(10);
        for (steps, expected, clock) in [
            (vec![Step::Eof], FrameRead::CleanEof, vec![0]),
            (
                vec![Step::Bytes(vec![0, 0]), Step::Eof],
                FrameRead::Truncated,
                vec![0, 1],
            ),
            (
                vec![
                    Step::Bytes(3u32.to_be_bytes().to_vec()),
                    Step::Bytes(vec![1]),
                    Step::Eof,
                ],
                FrameRead::Truncated,
                vec![0, 1, 2],
            ),
            (vec![Step::Timeout], FrameRead::Deadline, vec![0, 10]),
        ] {
            let mut input = ScriptedRead(VecDeque::from(steps));
            let mut framed = FramedReader::new(8);
            assert_eq!(
                framed
                    .read_until_with_clock(&mut input, deadline, times(base, &clock))
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn partial_frame_at_deadline_is_terminally_truncated() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(3u32.to_be_bytes().to_vec()),
            Step::Bytes(vec![1]),
            Step::Timeout,
        ]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(10),
                    times(base, &[0, 1, 2, 10]),
                )
                .unwrap(),
            FrameRead::Truncated
        );
    }
}
