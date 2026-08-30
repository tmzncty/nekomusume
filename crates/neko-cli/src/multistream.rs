//! Bounded TCP multi-stream executable fixture for deterministic VPS validation.
use neko_session::{
    InboundRecord, ProcessMessage, RuntimeLimits, SessionId, SessionRuntime, StreamId,
};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const MAX_STREAMS: usize = 16;
const MAX_RECORDS: usize = 64;
const MAX_BYTES: usize = 1024;
const MAX_FRAME: usize = neko_session::PROCESS_FRAME_MAX;
const SESSION: SessionId = SessionId(0x4e454b4f);

fn arg(args: &[String], name: &str, default: usize) -> usize {
    args.windows(2)
        .find(|x| x[0] == name)
        .and_then(|x| x[1].parse().ok())
        .unwrap_or(default)
}
fn bounds(streams: usize, records: usize, bytes: usize) -> Result<(), &'static str> {
    if !(1..=MAX_STREAMS).contains(&streams) {
        return Err("streams outside 1-16");
    }
    if !(1..=MAX_RECORDS).contains(&records) {
        return Err("records outside 1-64");
    }
    if !(1..=MAX_BYTES).contains(&bytes) {
        return Err("bytes outside 1-1024");
    }
    streams
        .checked_mul(records)
        .and_then(|n| n.checked_mul(bytes))
        .filter(|n| *n <= 1 << 20)
        .ok_or("total payload outside 1 MiB")?;
    Ok(())
}
fn frame_read(s: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut h = [0; 4];
    s.read_exact(&mut h).map_err(|e| e.to_string())?;
    let n = u32::from_be_bytes(h) as usize;
    if n > MAX_FRAME {
        return Err("frame exceeds bound".into());
    }
    let mut b = vec![0; n];
    s.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(b)
}
fn frame_write(s: &mut TcpStream, b: &[u8]) -> Result<(), String> {
    if b.len() > MAX_FRAME {
        return Err("frame exceeds bound".into());
    }
    s.write_all(&(b.len() as u32).to_be_bytes())
        .map_err(|e| e.to_string())?;
    s.write_all(b).map_err(|e| e.to_string())?;
    s.flush().map_err(|e| e.to_string())
}
fn limits(streams: usize, records: usize, bytes: usize) -> RuntimeLimits {
    RuntimeLimits {
        max_streams: streams,
        max_queue_records: streams * records + 1,
        max_queue_bytes: 1 << 20,
        max_total_bytes: streams * records * bytes,
        max_record_bytes: bytes,
        max_session_window: 1 << 20,
        max_stream_window: records * bytes,
        ..RuntimeLimits::default()
    }
}
fn fail(e: String) -> ! {
    eprintln!("neko: {e}");
    std::process::exit(2)
}
fn config(args: &[String]) -> (usize, usize, usize) {
    let v = (
        arg(args, "--streams", 2),
        arg(args, "--records", 3),
        arg(args, "--bytes", 32),
    );
    bounds(v.0, v.1, v.2).unwrap_or_else(|e| fail(e.into()));
    v
}
pub fn run(args: &[String]) {
    let mode = args
        .windows(2)
        .find(|x| x[0] == "--mode")
        .map(|x| x[1].as_str())
        .unwrap_or("client");
    let (streams, records, bytes) = config(args);
    let port = arg(args, "--port", 40082);
    if !(40080..=40100).contains(&(port as u16)) {
        fail("port outside 40080-40100".into());
    }
    let addr = args
        .windows(2)
        .find(|x| x[0] == "--addr")
        .map(|x| x[1].clone())
        .unwrap_or_else(|| format!("127.0.0.1:{port}"));
    let total = streams * records * bytes;
    if mode == "server" {
        let listener = TcpListener::bind(&addr).unwrap_or_else(|e| fail(format!("bind: {e}")));
        let (mut socket, peer) = listener
            .accept()
            .unwrap_or_else(|e| fail(format!("accept: {e}")));
        let mut runtime = SessionRuntime::new(SESSION, limits(streams, records, bytes), 0).unwrap();
        for stream in 0..streams {
            runtime.open_stream(StreamId(stream as u64 + 1), 0).unwrap();
        }
        let mut received = 0usize;
        for _ in 0..streams * records {
            let msg = ProcessMessage::decode(&frame_read(&mut socket).unwrap_or_else(|e| fail(e)))
                .unwrap_or_else(|_| fail("malformed process message".into()));
            let ProcessMessage::Data { session, record } = msg else {
                fail("expected data message".into())
            };
            if session != SESSION || record.data.len() != bytes {
                fail("session or record bound mismatch".into());
            }
            runtime
                .receive(
                    InboundRecord {
                        stream: record.stream,
                        offset: record.offset,
                        data: record.data.clone(),
                    },
                    received as u64,
                )
                .unwrap_or_else(|_| fail("out-of-order or over-limit record".into()));
            let ack = ProcessMessage::DeliveryAck {
                session: SESSION,
                stream: record.stream,
                offset: record.offset,
                len: record.data.len(),
            };
            frame_write(&mut socket, &ack.encode().unwrap()).unwrap_or_else(|e| fail(e));
            received += 1;
        }
        let json = format!(
            "{{\"ok\":true,\"role\":\"server\",\"peer\":\"{}\",\"streams\":{},\"records_per_stream\":{},\"bytes_per_record\":{},\"records\":{},\"payload_bytes\":{}}}",
            peer, streams, records, bytes, received, total
        );
        println!("{json}");
    } else if mode == "client" {
        let mut socket =
            TcpStream::connect(&addr).unwrap_or_else(|e| fail(format!("connect: {e}")));
        let mut runtime = SessionRuntime::new(SESSION, limits(streams, records, bytes), 0).unwrap();
        for stream in 0..streams {
            runtime.open_stream(StreamId(stream as u64 + 1), 0).unwrap();
        }
        let mut sent = 0usize;
        for round in 0..records {
            for stream in 0..streams {
                let id = StreamId(stream as u64 + 1);
                let data = vec![(stream + round) as u8; bytes];
                runtime.queue_send(id, &data, sent as u64).unwrap();
                let record = runtime.pop_send(sent as u64).unwrap().unwrap();
                frame_write(
                    &mut socket,
                    &ProcessMessage::Data {
                        session: SESSION,
                        record: record.clone(),
                    }
                    .encode()
                    .unwrap(),
                )
                .unwrap();
                let ack =
                    ProcessMessage::decode(&frame_read(&mut socket).unwrap_or_else(|e| fail(e)))
                        .unwrap_or_else(|_| fail("malformed acknowledgement".into()));
                let ProcessMessage::DeliveryAck {
                    session,
                    stream: ack_stream,
                    offset,
                    len,
                } = ack
                else {
                    fail("expected acknowledgement".into())
                };
                if session != SESSION || ack_stream != id || offset != record.offset || len != bytes
                {
                    fail("acknowledgement mismatch".into());
                }
                runtime
                    .delivery_ack(id, record.offset, bytes, sent as u64)
                    .unwrap();
                sent += 1;
            }
        }
        println!(
            "{{\"ok\":true,\"role\":\"client\",\"streams\":{},\"records_per_stream\":{},\"bytes_per_record\":{},\"records\":{},\"payload_bytes\":{}}}",
            streams, records, bytes, sent, total
        );
    } else {
        fail("mode must be client or server".into());
    }
}
