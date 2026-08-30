//! Bounded TCP multi-stream executable fixture for deterministic VPS validation.
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, SecureSession,
    TrustPolicy, TrustRecord, TrustStatus,
};
use neko_session::{
    InboundRecord, ProcessMessage, RuntimeLimits, SessionId, SessionRuntime, StreamId,
};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const MAX_STREAMS: usize = 16;
const MAX_RECORDS: usize = 64;
const MAX_BYTES: usize = 1024;
const MAX_FRAME: usize = neko_session::PROCESS_FRAME_MAX + 8 + neko_crypto::RECORD_CONTEXT_LEN + 16;
const DOMAIN: &[u8] = b"nekomusume/neko-cli/multistream/v1";
const SCOPE: &[u8] = b"neko-cli/multistream";
const SESSION: SessionId = SessionId(0x4e454b4f);

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|x| x[0] == name).map(|x| x[1].clone())
}
fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() != 64 || !s.is_ascii() {
        return Err("key must be 32-byte hex".into());
    }
    (0..64)
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| "invalid key hex".to_string()))
        .collect()
}
fn identity(args: &[String]) -> LocalIdentity {
    let path = option(args, "--identity").unwrap_or_else(|| fail("--identity is required".into()));
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|_| fail("identity read failed".into()));
    let mut parts = text.trim().split(':');
    let private = hex_decode(parts.next().unwrap_or("")).unwrap_or_else(|e| fail(e));
    let public = hex_decode(parts.next().unwrap_or("")).unwrap_or_else(|e| fail(e));
    if parts.next().is_some() {
        fail("invalid identity file".into());
    }
    LocalIdentity::from_keypair(&private, &public)
        .unwrap_or_else(|_| fail("invalid identity file".into()))
}
fn context() -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 0,
        direction: 0,
    }
}
fn secure_write(s: &mut TcpStream, session: &mut SecureSession, msg: &ProcessMessage) {
    let wire = session
        .seal(
            &msg.encode()
                .unwrap_or_else(|_| fail("message encode failed".into())),
        )
        .unwrap_or_else(|_| fail("encryption failed".into()));
    frame_write(s, &wire).unwrap_or_else(|e| fail(e));
}
fn secure_read(s: &mut TcpStream, session: &mut SecureSession) -> ProcessMessage {
    let wire = frame_read(s).unwrap_or_else(|e| fail(e));
    let plain = session
        .open(&wire)
        .unwrap_or_else(|_| fail("unauthenticated transport frame".into()));
    ProcessMessage::decode(&plain).unwrap_or_else(|_| fail("malformed process message".into()))
}
fn server_handshake(
    socket: &mut TcpStream,
    id: LocalIdentity,
    client_key: Vec<u8>,
) -> SecureSession {
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: SCOPE.to_vec(),
        status: TrustStatus::Active,
    }]);
    let hs = ResponderHandshake::new(&id, policy, DOMAIN)
        .unwrap_or_else(|_| fail("handshake setup failed".into()));
    let first = frame_read(socket).unwrap_or_else(|e| fail(e));
    let (response, session) = hs
        .receive_first(&first, context())
        .unwrap_or_else(|_| fail("unauthorized client".into()));
    frame_write(socket, &response).unwrap_or_else(|e| fail(e));
    session
}
fn client_handshake(
    socket: &mut TcpStream,
    id: LocalIdentity,
    server_key: Vec<u8>,
) -> SecureSession {
    let mut hs = InitiatorHandshake::new(&id, &server_key, SCOPE, DOMAIN)
        .unwrap_or_else(|_| fail("handshake setup failed".into()));
    frame_write(
        socket,
        &hs.first_message()
            .unwrap_or_else(|_| fail("handshake failed".into())),
    )
    .unwrap_or_else(|e| fail(e));
    let response = frame_read(socket).unwrap_or_else(|e| fail(e));
    hs.finish(&response, context())
        .unwrap_or_else(|_| fail("handshake rejected".into()))
}

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
        let client_key = hex_decode(
            &option(args, "--client-key")
                .unwrap_or_else(|| fail("--client-key is required".into())),
        )
        .unwrap_or_else(|e| fail(e));
        let mut secure = server_handshake(&mut socket, identity(args), client_key);
        let mut runtime = SessionRuntime::new(SESSION, limits(streams, records, bytes), 0).unwrap();
        for stream in 0..streams {
            runtime.open_stream(StreamId(stream as u64 + 1), 0).unwrap();
        }
        let mut received = 0usize;
        for _ in 0..streams * records {
            let msg = secure_read(&mut socket, &mut secure);
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
            secure_write(&mut socket, &mut secure, &ack);
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
        let server_key = hex_decode(
            &option(args, "--server-key")
                .unwrap_or_else(|| fail("--server-key is required".into())),
        )
        .unwrap_or_else(|e| fail(e));
        let mut secure = client_handshake(&mut socket, identity(args), server_key);
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
                secure_write(
                    &mut socket,
                    &mut secure,
                    &ProcessMessage::Data {
                        session: SESSION,
                        record: record.clone(),
                    },
                );
                let ack = secure_read(&mut socket, &mut secure);
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
