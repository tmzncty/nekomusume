use std::io::{self, Read};
use std::time::Instant;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FrameRead {
    Complete(Vec<u8>),
    Deadline,
    CleanEof,
    Truncated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameStage {
    Header { bytes: usize, work: usize },
    Body { bytes: usize, work: usize },
    Complete,
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

    /// Reads one frame while charging its fixed header and declared body as stages of a single logical input record.
    pub(crate) fn read_until_staged<R, F>(
        &mut self,
        reader: &mut R,
        deadline: Instant,
        mut stage: F,
    ) -> io::Result<FrameRead>
    where
        R: Read,
        F: FnMut(FrameStage) -> io::Result<()>,
    {
        self.read_until_staged_with_clock(reader, deadline, &mut stage, Instant::now)
    }

    fn read_until_staged_with_clock<R, F, N>(
        &mut self,
        reader: &mut R,
        deadline: Instant,
        stage: &mut F,
        mut now: N,
    ) -> io::Result<FrameRead>
    where
        R: Read,
        F: FnMut(FrameStage) -> io::Result<()>,
        N: FnMut() -> Instant,
    {
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
                        stage(FrameStage::Header { bytes: 4, work: 64 })?;
                        let len = u32::from_be_bytes(self.header) as usize;
                        if len > self.max_frame_len {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "frame too large",
                            ));
                        }
                        stage(FrameStage::Body {
                            bytes: len,
                            work: 4096,
                        })?;
                        self.payload = vec![0; len];
                        self.payload_len = 0;
                        if len == 0 {
                            stage(FrameStage::Complete)?;
                            return Ok(FrameRead::Complete(self.take_frame()));
                        }
                    }
                }
                Ok(n) => {
                    self.payload_len += n;
                    if self.payload_len == self.payload.len() {
                        stage(FrameStage::Complete)?;
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

    /// A clock pinned to `base` for every poll, so multi-step reads are not
    /// cut off by an exhausted tick list.
    fn pinned(base: Instant) -> impl FnMut() -> Instant {
        move || base
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
    fn staged_reader_reports_ordered_success_and_failure_stages() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(vec![0]),
            Step::Bytes(vec![0, 0, 3]),
            Step::Bytes(b"abc".to_vec()),
        ]));
        let mut framed = FramedReader::new(8);
        let mut stages = Vec::new();
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |stage| {
                        stages.push(stage);
                        Ok(())
                    },
                    times(base, &[0, 1, 2]),
                )
                .unwrap(),
            FrameRead::Complete(b"abc".to_vec())
        );
        assert_eq!(
            stages,
            vec![
                FrameStage::Header { bytes: 4, work: 64 },
                FrameStage::Body {
                    bytes: 3,
                    work: 4096
                },
                FrameStage::Complete,
            ]
        );

        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(3u32.to_be_bytes().to_vec()),
            Step::Bytes(vec![1]),
            Step::Eof,
        ]));
        let mut framed = FramedReader::new(8);
        let mut stages = Vec::new();
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |stage| {
                        stages.push(stage);
                        Ok(())
                    },
                    times(base, &[0, 1, 2]),
                )
                .unwrap(),
            FrameRead::Truncated
        );
        assert_eq!(
            stages,
            vec![
                FrameStage::Header { bytes: 4, work: 64 },
                FrameStage::Body {
                    bytes: 3,
                    work: 4096
                },
            ]
        );
    }

    #[test]
    fn staged_reader_rejects_oversize_after_header_before_body_read() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(9u32.to_be_bytes().to_vec())]));
        let mut framed = FramedReader::new(8);
        let mut stages = Vec::new();
        let error = framed
            .read_until_staged_with_clock(
                &mut input,
                base + Duration::from_secs(1),
                &mut |stage| {
                    stages.push(stage);
                    Ok(())
                },
                times(base, &[0]),
            )
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(stages, vec![FrameStage::Header { bytes: 4, work: 64 }]);
        assert!(framed.payload.is_empty());
    }

    #[test]
    fn staged_reader_zero_body_completes_once_and_callback_rejection_stops_read() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(0u32.to_be_bytes().to_vec())]));
        let mut framed = FramedReader::new(8);
        let mut stages = Vec::new();
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |stage| {
                        stages.push(stage);
                        Ok(())
                    },
                    times(base, &[0]),
                )
                .unwrap(),
            FrameRead::Complete(Vec::new())
        );
        assert_eq!(
            stages,
            vec![
                FrameStage::Header { bytes: 4, work: 64 },
                FrameStage::Body {
                    bytes: 0,
                    work: 4096
                },
                FrameStage::Complete,
            ]
        );

        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(3u32.to_be_bytes().to_vec())]));
        let mut framed = FramedReader::new(8);
        let mut calls = 0;
        assert!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |_| {
                        calls += 1;
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "reject"))
                    },
                    times(base, &[0]),
                )
                .is_err()
        );
        assert_eq!(calls, 1);
        assert!(framed.payload.is_empty());
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

    #[test]
    fn staged_reader_deadline_and_eof_outcomes_match_the_non_staged_reader() {
        let base = Instant::now();
        let deadline = base + Duration::from_millis(10);
        // No partial frame buffered: the deadline is a Deadline, not a truncation.
        let mut input = ScriptedRead(VecDeque::from([Step::Timeout]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    deadline,
                    &mut |_| Ok(()),
                    times(base, &[0, 10]),
                )
                .unwrap(),
            FrameRead::Deadline
        );
        // A half-read header at the deadline is Truncated, not Deadline.
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(vec![0, 0]), Step::Timeout]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    deadline,
                    &mut |_| Ok(()),
                    times(base, &[0, 1, 10]),
                )
                .unwrap(),
            FrameRead::Truncated
        );
        // A declared body with no bytes yet is likewise Truncated.
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(3u32.to_be_bytes().to_vec()),
            Step::Timeout,
        ]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    deadline,
                    &mut |_| Ok(()),
                    times(base, &[0, 1, 10]),
                )
                .unwrap(),
            FrameRead::Truncated
        );
        // A clean EOF with nothing buffered is CleanEof, not a deadline expiry.
        let mut input = ScriptedRead(VecDeque::from([Step::Eof]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_staged_with_clock(&mut input, deadline, &mut |_| Ok(()), pinned(base),)
                .unwrap(),
            FrameRead::CleanEof
        );
    }

    #[test]
    fn staged_reader_tolerates_poll_timeouts_while_completing_a_frame() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(vec![0]),
            Step::Timeout,
            Step::Bytes(vec![0, 0, 2]),
            Step::Timeout,
            Step::Bytes(b"ab".to_vec()),
        ]));
        let mut framed = FramedReader::new(8);
        let mut stages = Vec::new();
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |stage| {
                        stages.push(stage);
                        Ok(())
                    },
                    times(base, &[0, 1, 2, 3, 4]),
                )
                .unwrap(),
            FrameRead::Complete(b"ab".to_vec())
        );
        assert_eq!(
            stages,
            vec![
                FrameStage::Header { bytes: 4, work: 64 },
                FrameStage::Body {
                    bytes: 2,
                    work: 4096
                },
                FrameStage::Complete,
            ]
        );
    }

    #[test]
    fn a_frame_of_exactly_the_maximum_length_is_accepted_on_both_readers() {
        let base = Instant::now();
        let payload = b"abcd";
        for staged in [false, true] {
            let mut wire = (payload.len() as u32).to_be_bytes().to_vec();
            wire.extend_from_slice(payload);
            let mut input = ScriptedRead(VecDeque::from([Step::Bytes(wire)]));
            let mut framed = FramedReader::new(4);
            let got = if staged {
                framed
                    .read_until_staged_with_clock(
                        &mut input,
                        base + Duration::from_secs(1),
                        &mut |_| Ok(()),
                        pinned(base),
                    )
                    .unwrap()
            } else {
                framed
                    .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                    .unwrap()
            };
            assert_eq!(got, FrameRead::Complete(payload.to_vec()));
            // One byte more than the maximum is refused before allocating.
            let mut input =
                ScriptedRead(VecDeque::from([Step::Bytes(5u32.to_be_bytes().to_vec())]));
            let mut framed = FramedReader::new(4);
            let err = if staged {
                framed
                    .read_until_staged_with_clock(
                        &mut input,
                        base + Duration::from_secs(1),
                        &mut |_| Ok(()),
                        pinned(base),
                    )
                    .unwrap_err()
            } else {
                framed
                    .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                    .unwrap_err()
            };
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn a_reader_reused_for_a_second_frame_starts_clean() {
        let base = Instant::now();
        // Two frames back to back on the SAME reader: the first frame must not
        // leave any buffered state behind, or the second read would see a
        // stale header counter and truncate.
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(2u32.to_be_bytes().to_vec()),
            Step::Bytes(b"ab".to_vec()),
            Step::Bytes(0u32.to_be_bytes().to_vec()),
            Step::Bytes(1u32.to_be_bytes().to_vec()),
            Step::Bytes(b"z".to_vec()),
        ]));
        let mut framed = FramedReader::new(8);
        for expected in [b"ab".to_vec(), Vec::new(), b"z".to_vec()] {
            assert_eq!(
                framed
                    .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base),)
                    .unwrap(),
                FrameRead::Complete(expected)
            );
        }
        // The staged reader must reuse just as cleanly.
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(1u32.to_be_bytes().to_vec()),
            Step::Bytes(b"q".to_vec()),
            Step::Bytes(1u32.to_be_bytes().to_vec()),
            Step::Bytes(b"r".to_vec()),
        ]));
        let mut framed = FramedReader::new(8);
        for expected in [b"q".to_vec(), b"r".to_vec()] {
            assert_eq!(
                framed
                    .read_until_staged_with_clock(
                        &mut input,
                        base + Duration::from_secs(1),
                        &mut |_| Ok(()),
                        pinned(base),
                    )
                    .unwrap(),
                FrameRead::Complete(expected)
            );
        }
    }

    #[test]
    fn set_max_frame_len_applies_later_and_refuses_a_partial_frame() {
        let base = Instant::now();
        // A 6-byte frame is refused by the construction limit of 4...
        let mut framed = FramedReader::new(4);
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(6u32.to_be_bytes().to_vec()),
            Step::Bytes(b"abcdef".to_vec()),
        ]));
        assert_eq!(
            framed
                .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        // ...and accepted once the limit is raised.
        let mut framed = FramedReader::new(4);
        framed.set_max_frame_len(6).unwrap();
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(6u32.to_be_bytes().to_vec()),
            Step::Bytes(b"abcdef".to_vec()),
        ]));
        assert_eq!(
            framed
                .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                .unwrap(),
            FrameRead::Complete(b"abcdef".to_vec())
        );
        // A truncated read leaves a partial frame buffered: the limit must not
        // be changeable, and the refusal must leave the old limit in force.
        let mut framed = FramedReader::new(4);
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(vec![0, 0])]));
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(5),
                    times(base, &[0, 5]),
                )
                .unwrap(),
            FrameRead::Truncated
        );
        assert_eq!(
            framed.set_max_frame_len(64).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        // Still limited by the original 4: completing the buffered header with
        // a declared length of 9 is refused.
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(vec![9, 0])]));
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(5),
                    times(base, &[0, 5]),
                )
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn a_zero_length_frame_is_complete_on_both_readers() {
        let base = Instant::now();
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(0u32.to_be_bytes().to_vec())]));
        let mut framed = FramedReader::new(8);
        assert_eq!(
            framed
                .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                .unwrap(),
            FrameRead::Complete(Vec::new())
        );
    }

    #[test]
    fn a_reader_that_finished_a_frame_is_not_partial() {
        let base = Instant::now();
        // After a completed frame the reader holds no buffered state, so an
        // already-expired deadline is a Deadline - not a truncation of the
        // frame that was just returned.
        let mut framed = FramedReader::new(8);
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(2u32.to_be_bytes().to_vec()),
            Step::Bytes(b"ab".to_vec()),
        ]));
        assert_eq!(
            framed
                .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                .unwrap(),
            FrameRead::Complete(b"ab".to_vec())
        );
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(10),
                    times(base, &[10]),
                )
                .unwrap(),
            FrameRead::Deadline
        );

        let mut framed = FramedReader::new(8);
        let mut input = ScriptedRead(VecDeque::from([
            Step::Bytes(2u32.to_be_bytes().to_vec()),
            Step::Bytes(b"ab".to_vec()),
        ]));
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_secs(1),
                    &mut |_| Ok(()),
                    pinned(base),
                )
                .unwrap(),
            FrameRead::Complete(b"ab".to_vec())
        );
        assert_eq!(
            framed
                .read_until_staged_with_clock(
                    &mut input,
                    base + Duration::from_millis(10),
                    &mut |_| Ok(()),
                    times(base, &[10]),
                )
                .unwrap(),
            FrameRead::Deadline
        );

        // A zero-length frame must likewise leave no residue behind.
        let mut framed = FramedReader::new(8);
        let mut input = ScriptedRead(VecDeque::from([Step::Bytes(0u32.to_be_bytes().to_vec())]));
        assert_eq!(
            framed
                .read_until_with_clock(&mut input, base + Duration::from_secs(1), pinned(base))
                .unwrap(),
            FrameRead::Complete(Vec::new())
        );
        assert_eq!(
            framed
                .read_until_with_clock(
                    &mut input,
                    base + Duration::from_millis(10),
                    times(base, &[10]),
                )
                .unwrap(),
            FrameRead::Deadline
        );
    }
}
