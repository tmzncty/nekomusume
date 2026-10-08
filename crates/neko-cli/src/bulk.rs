//! D068 W1 bulk workload: one-way authenticated bulk transfer with a
//! counted, checksum-verified final receipt (sink semantics — the receiver
//! never echoes data, so the lifetime accumulator counts exactly the data
//! volume, per D068's single-direction design).
//!
//! Measurement-gated: only reachable with NEKO_MEASUREMENT=1; the runtime
//! uses `SessionRuntime::new_measurement` (closed three-axis exception
//! set). Without the variable the command refuses.
use super::*;
use crate::framed::{FrameRead, FramedReader};
use sha2::{Digest, Sha256};

const SESSION: SessionId = SessionId(7301);
const STREAM: StreamId = StreamId(1);
/// Default record size: the largest payload that fits a framed Process
/// data record inside one 1200B datagram-equivalent seal (periodic's #3
/// ceiling applies identically here).
const DEFAULT_RECORD_BYTES: usize = 1170;
const DEFAULT_BYTES_TOTAL: usize = 64 << 20; // 64 MiB
const MAX_BYTES_TOTAL: usize = neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES;
const RATE_WINDOW: Duration = Duration::from_millis(100);

struct BulkConfig {
    port: u16,
    bytes_total: usize,
    record_bytes: usize,
    /// None = unthrottled; Some(bytes/sec)
    rate_limit: Option<usize>,
    duration: Duration,
}

fn config(args: &[String]) -> Result<BulkConfig, &'static str> {
    if !crate::measurement::enabled() {
        return Err("bulk requires NEKO_MEASUREMENT=1 (research-only)");
    }
    let num = |key: &str, default: &str| -> Result<u64, &'static str> {
        crate::parse(args, key, Some(default))
            .parse()
            .map_err(|_| "invalid numeric argument")
    };
    let port = num("--port", "40080")? as u16;
    if !(40080..=MAX_PORT).contains(&port) {
        return Err("port outside 40080-40100");
    }
    let record_bytes = num("--record-bytes", &DEFAULT_RECORD_BYTES.to_string())? as usize;
    if record_bytes == 0 || record_bytes > crate::PERIODIC_BYTES_MAX {
        return Err("record-bytes outside 1-1170");
    }
    let bytes_total = num("--bytes-total", &DEFAULT_BYTES_TOTAL.to_string())? as usize;
    if bytes_total == 0
        || !bytes_total.is_multiple_of(record_bytes)
        || bytes_total > MAX_BYTES_TOTAL
    {
        return Err("bytes-total must be a positive multiple of record-bytes, <= 2147483648");
    }
    let rate_limit = match value(args, "--rate-limit") {
        Some(v) => Some(
            v.parse::<usize>()
                .map_err(|_| "invalid rate-limit")?
                .max(record_bytes),
        ),
        None => None,
    };
    let duration = num("--duration", "86400")?;
    if duration == 0 || duration > crate::MEASUREMENT_DURATION_MAX {
        return Err("duration outside 1-86400");
    }
    Ok(BulkConfig {
        port,
        bytes_total,
        record_bytes,
        rate_limit,
        duration: Duration::from_secs(duration),
    })
}

fn value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn limits(_cfg: &BulkConfig) -> neko_session::RuntimeLimits {
    neko_session::RuntimeLimits {
        max_streams: 1,
        max_queue_records: neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_RECORDS,
        max_queue_bytes: neko_session::MEASUREMENT_MAX_RUNTIME_QUEUE_BYTES,
        max_total_bytes: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        max_record_bytes: crate::PERIODIC_BYTES_MAX,
        max_session_window: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        max_stream_window: neko_session::MEASUREMENT_MAX_RUNTIME_TOTAL_BYTES,
        ..neko_session::RuntimeLimits::default()
    }
}

/// Deterministic non-uniform payload (counter-seeded blocks): real data for
/// compression/structure, verifiable per-block without echoing.
fn payload_block(index: u64, len: usize) -> Vec<u8> {
    let mut block = Vec::with_capacity(len);
    let mut seed = index.wrapping_mul(0x9E3779B97F4A7C15).max(1);
    while block.len() < len {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        block.extend_from_slice(&seed.to_be_bytes());
    }
    block.truncate(len);
    block
}

fn install_shutdown() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&shutdown)).unwrap();
    signal_hook::flag::register(SIGINT, Arc::clone(&shutdown)).unwrap();
    shutdown
}

fn handshake_server(
    stream: &mut TcpStream,
    reader: &mut FramedReader,
    deadline: Instant,
    id: &LocalIdentity,
    policy: TrustPolicy,
) -> Result<neko_crypto::SecureSession, &'static str> {
    bound_stream_to_deadline(stream, deadline, None).map_err(|_| "setup deadline elapsed")?;
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
    let hello = match reader.read_until(stream, deadline) {
        Ok(FrameRead::Complete(f)) => f,
        _ => return Err("malformed negotiation"),
    };
    let selection = negotiation
        .server_accept_hello(&hello)
        .map_err(|_| "incompatible negotiation")?;
    write_frame_single(stream, &selection).map_err(|_| "negotiation response failed")?;
    let binding = negotiation.authenticated_binding().unwrap();
    let first = match reader.read_until(stream, deadline) {
        Ok(FrameRead::Complete(f)) => f,
        _ => return Err("bad handshake"),
    };
    let (response, secure) =
        ResponderHandshake::new_with_prologue_binding(id, policy, DOMAIN, binding.as_bytes())
            .map_err(|_| "handshake setup failed")?
            .receive_first(&first, context(0))
            .map_err(|_| "unauthorized handshake")?;
    write_frame_single(stream, &response).map_err(|_| "handshake response failed")?;
    negotiation
        .admit_data()
        .map_err(|_| "data admission denied")?;
    Ok(secure)
}

pub(super) fn server(args: &[String]) {
    crate::measurement::emit_marker();
    println!("measurement_limits_exceptions=total_bytes,queue_records,queue_bytes");
    let cfg = config(args).unwrap_or_else(|e| fail(e));
    let client_key = peer_public_key(&parse(args, "--client-key", None));
    let bind = parse(args, "--bind", Some(&format!("0.0.0.0:{}", cfg.port)))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad bind address"));
    let shutdown = install_shutdown();
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    println!("server_public_key={}", hex(id.public_key()));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: b"probe".to_vec(),
        status: TrustStatus::Active,
    }]);
    let listener = TcpListener::bind(bind).unwrap_or_else(|_| fail("bind failed"));
    listener.set_nonblocking(true).unwrap();
    println!("bulk_server_ready transport=tcp port={}", cfg.port);
    let start = Instant::now();
    let mut framed = FramedReader::new(PROCESS_FRAME_MAX + 128);
    let mut authenticated: Option<(TcpStream, neko_crypto::SecureSession)> = None;
    while authenticated.is_none()
        && start.elapsed() < cfg.duration
        && !shutdown.load(Ordering::Acquire)
    {
        match listener.accept() {
            Ok((mut stream, _peer)) => {
                apply_f1_nodelay(&stream);
                stream
                    .set_read_timeout(Some(Duration::from_millis(100)))
                    .unwrap();
                let deadline = start + cfg.duration;
                match handshake_server(&mut stream, &mut framed, deadline, &id, policy.clone()) {
                    Ok(secure) => authenticated = Some((stream, secure)),
                    Err(reason) => {
                        println!("bulk_server_rejected reason={reason}");
                        let _ = stream.shutdown(Shutdown::Both);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => fail("accept failed"),
        }
    }
    let Some((mut stream, mut secure)) = authenticated else {
        fail("duration expired before authenticated Session");
    };
    // D069 R2: in measurement mode the server must admit sealed batch
    // frames (up to MAX_BATCH_PLAINTEXT + record overhead + TCP frame
    // header). Default mode keeps the single-record bound.
    let frame_bound = if crate::measurement::enabled() {
        neko_crypto::MAX_BATCH_PLAINTEXT + neko_crypto::RECORD_SEAL_OVERHEAD + 128
    } else {
        PROCESS_FRAME_MAX + 128
    };
    framed.set_max_frame_len(frame_bound).unwrap();
    println!("bulk_server_authenticated session=7301 stream=1");
    let mut runtime =
        neko_session::SessionRuntime::new_measurement(SESSION, limits(&cfg), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let total_records = cfg.bytes_total / cfg.record_bytes;
    let mut hasher = Sha256::new();
    let mut received = 0usize;
    let mut received_bytes = 0usize;
    let deadline = start + cfg.duration;
    while received < total_records && Instant::now() < deadline {
        match framed.read_until(&mut stream, deadline) {
            Ok(FrameRead::Complete(frame)) => {
                // D069 R2: try batch first, fall back to the single-record
                // path (compat window — both shapes may arrive during the
                // rollout; the receipt frame from client-side batching is
                // also a single sealed ProcessMessage).
                let plains: Vec<Vec<u8>> = match secure.open_batch(&frame) {
                    Ok(records) => records,
                    Err(_) => vec![
                        secure
                            .open_unreliable(&frame)
                            .unwrap_or_else(|_| fail("unauthenticated bulk data")),
                    ],
                };
                for plain in plains {
                    let ProcessMessage::Data { session, record } = ProcessMessage::decode(&plain)
                        .unwrap_or_else(|_| fail("malformed bulk data"))
                    else {
                        fail("expected bulk data");
                    };
                    if session != SESSION || record.stream != STREAM {
                        fail("bulk record outside contract");
                    }
                    let expected =
                        payload_block(record.offset / cfg.record_bytes as u64, cfg.record_bytes);
                    if record.data != expected {
                        fail("bulk payload checksum mismatch");
                    }
                    runtime
                        .receive(
                            InboundRecord {
                                stream: record.stream,
                                offset: record.offset,
                                data: record.data.clone(),
                            },
                            start.elapsed().as_millis() as u64,
                        )
                        .unwrap_or_else(|_| fail("bulk record rejected by runtime"));
                    let _ = runtime.pop_receive(start.elapsed().as_millis() as u64);
                    hasher.update(&record.data);
                    received += 1;
                    received_bytes += record.data.len();
                }
            }
            Ok(FrameRead::Deadline)
            | Ok(FrameRead::CleanEof)
            | Ok(FrameRead::Truncated)
            | Err(_) => {
                break;
            }
        }
    }
    let digest_bytes = hasher.finalize();
    let digest = hex(&digest_bytes[..]);
    // final receipt: one ProcessMessage frame the client can verify
    let receipt = format!(
        "{{\"ok\":true,\"role\":\"bulk-server\",\"records\":{},\"bytes\":{},\"sha256\":\"{}\"}}",
        received, received_bytes, digest
    );
    println!("bulk_server_summary {receipt}");
    // send the receipt over the wire (one sealed ProcessMessage frame) so
    // the client can verify counts+digest without trusting stdout
    let receipt_record = neko_session::OutboundRecord {
        stream: STREAM,
        offset: 0,
        data: receipt.into_bytes(),
    };
    let receipt_plain = ProcessMessage::Data {
        session: SESSION,
        record: receipt_record,
    }
    .encode()
    .unwrap();
    let receipt_frame = secure
        .seal_unreliable(&receipt_plain)
        .unwrap_or_else(|_| fail("receipt seal failed"));
    write_frame_single(&mut stream, &receipt_frame).unwrap_or_else(|_| fail("receipt send failed"));
    let _ = stream.shutdown(Shutdown::Both);
}

pub(super) fn client(args: &[String]) {
    crate::measurement::emit_marker();
    println!("measurement_limits_exceptions=total_bytes,queue_records,queue_bytes");
    let cfg = config(args).unwrap_or_else(|e| fail(e));
    let addr = parse(args, "--addr", None)
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad address"));
    let server_key = peer_public_key(&parse(args, "--server-key", None));
    let shutdown = install_shutdown();
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
    println!("client_public_key={}", hex(id.public_key()));
    let start = Instant::now();
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
        .unwrap_or_else(|_| fail("connect failed"));
    apply_f1_nodelay(&stream);
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut framed = FramedReader::new(PROCESS_FRAME_MAX + 128);
    let mut secure = crate::periodic::handshake_client(
        &mut stream,
        &mut framed,
        start + Duration::from_secs(10),
        &id,
        &server_key,
    );
    framed.set_max_frame_len(PROCESS_FRAME_MAX + 128).unwrap();
    println!("bulk_client_authenticated session=7301 stream=1");
    let mut runtime =
        neko_session::SessionRuntime::new_measurement(SESSION, limits(&cfg), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let total_records = cfg.bytes_total / cfg.record_bytes;
    let deadline = start + cfg.duration;
    let mut sent = 0usize;
    let mut sent_bytes = 0usize;
    let mut window_bytes = 0usize;
    let mut window_start = Instant::now();
    // D069 R2: batch packing. `--batch-n` (1-64, default 8) records per
    // sealed batch; measurement-gated (default mode never batches — the
    // plain single-record path stays the default surface). Each batch is
    // ONE write_frame_single (94b0eb4 base), so per-batch framing and
    // syscalls amortize across N records.
    let batch_n = if crate::measurement::enabled() {
        parse(args, "--batch-n", Some("8"))
            .parse::<usize>()
            .unwrap_or_else(|_| fail("--batch-n must be an integer 1-64"))
    } else {
        1
    };
    if !(1..=64).contains(&batch_n) {
        fail("--batch-n must be within 1-64");
    }
    // Transport-budget auto-N (D069): the staged batch body must stay
    // within MAX_BATCH_PLAINTEXT (snow MAXMSGLEN-bound). Flush by BYTE
    // budget at flush time so a large --batch-n clamps itself to what the
    // actual encoded record size fits (--batch-n 64 with 1170B records
    // runs at effective N≈54) instead of failing.
    let mut pending: Vec<Vec<u8>> = Vec::with_capacity(batch_n);
    let flush_batch = |pending: &mut Vec<Vec<u8>>,
                       stream: &mut TcpStream,
                       secure: &mut neko_crypto::SecureSession| {
        if pending.is_empty() {
            return;
        }
        let refs: Vec<&[u8]> = pending.iter().map(|v| v.as_slice()).collect();
        let encrypted = secure
            .seal_batch(&refs)
            .unwrap_or_else(|_| fail("batch seal failed"));
        write_frame_single(stream, &encrypted).unwrap_or_else(|_| fail("bulk batch send failed"));
        pending.clear();
    };
    while sent < total_records && Instant::now() < deadline && !shutdown.load(Ordering::Acquire) {
        if let Some(rate) = cfg.rate_limit
            && window_bytes >= rate
        {
            let elapsed = window_start.elapsed();
            if elapsed < RATE_WINDOW {
                std::thread::sleep(RATE_WINDOW - elapsed);
            }
            window_bytes = 0;
            window_start = Instant::now();
        }
        let payload = payload_block(sent as u64, cfg.record_bytes);
        runtime
            .queue_send(STREAM, &payload, start.elapsed().as_millis() as u64)
            .unwrap_or_else(|_| fail("bulk queue rejected"));
        let record = runtime
            .pop_send(start.elapsed().as_millis() as u64)
            .unwrap()
            .unwrap();
        let plain = ProcessMessage::Data {
            session: SESSION,
            record,
        }
        .encode()
        .unwrap();
        if batch_n > 1 {
            // Budget check BEFORE pushing: if adding this record would push
            // the staged body over the transport budget, flush the current
            // batch first so no seal_batch call ever sees an over-budget body.
            let incoming = 4 + plain.len();
            let staged_after = 4 + pending.iter().map(|p| 4 + p.len()).sum::<usize>() + incoming;
            if staged_after > neko_crypto::MAX_BATCH_PLAINTEXT.saturating_sub(64) {
                flush_batch(&mut pending, &mut stream, &mut secure);
            }
            pending.push(plain);
            if pending.len() >= batch_n {
                flush_batch(&mut pending, &mut stream, &mut secure);
            }
        } else {
            let encrypted = secure
                .seal_unreliable(&plain)
                .unwrap_or_else(|_| fail("payload too large"));
            write_frame_single(&mut stream, &encrypted)
                .unwrap_or_else(|_| fail("bulk send failed"));
        }
        sent += 1;
        sent_bytes += cfg.record_bytes;
        window_bytes += cfg.record_bytes;
    }
    // flush any trailing partial batch before half-close
    flush_batch(&mut pending, &mut stream, &mut secure);
    // half-close and await the server's final summary line (bounded)
    let _ = stream.shutdown(Shutdown::Write);
    let receipt: Option<String> =
        match framed.read_until(&mut stream, Instant::now() + Duration::from_secs(10)) {
            Ok(FrameRead::Complete(frame)) => secure
                .open_unreliable(&frame)
                .ok()
                .and_then(|plain| ProcessMessage::decode(&plain).ok())
                .and_then(|msg| match msg {
                    ProcessMessage::Data { record, .. } => String::from_utf8(record.data).ok(),
                    _ => None,
                }),
            _ => None,
        };
    let elapsed = start.elapsed();
    let goodput_bps = if elapsed.as_secs_f64() > 0.0 {
        (sent_bytes as f64 / elapsed.as_secs_f64()) as u64
    } else {
        0
    };
    println!(
        "bulk_client_summary sent_records={} sent_bytes={} goodput_bps={} elapsed_ms={} server_receipt={}",
        sent,
        sent_bytes,
        goodput_bps,
        elapsed.as_millis(),
        receipt.unwrap_or_else(|| "unverified".to_string())
    );
}

use std::net::TcpStream;

/// F1 (review-2026-10-08, Yvonne ADR §0): per-record stop-and-wait writes a
/// 4-byte header then a ~1170-byte body. With Nagle enabled the body write
/// can be held until the header segment is ACKed, and the receiver's
/// delayed-ACK holds that ACK ~40ms — on real-RTT paths this folds into
/// every exchange. Loopback never exposes it (ACKs are local), which is why
/// the loopback ceiling is CPU-paced. Every TCP socket these measurement
/// fixtures put on the wire must disable Nagle at connect/accept time.
pub fn apply_f1_nodelay(stream: &TcpStream) {
    if let Err(e) = stream.set_nodelay(true) {
        // Non-fatal: NODELAY is a performance hint; measurement validity on
        // loopback is unaffected and WAN runs record the failure in-band.
        eprintln!("neko: set_nodelay failed ({e}); continuing with Nagle");
    }
}

#[cfg(test)]
mod tests {
    //! F1 hypothesis pin (review-2026-10-08, Yvonne ADR §0): per-record
    //! stop-and-wait sends a small write and immediately blocks on read. With
    //! Nagle on, that write can wait on the previous ACK, folding the
    //! delayed-ACK timer into every exchange on real-RTT paths. Loopback
    //! never exposes it (ACKs are local), which is why the loopback ceiling
    //! is CPU-paced. Two pins: the helper actually flips the socket flag, and
    //! every fixture file still carries its call sites.
    use std::net::{TcpListener, TcpStream};

    #[test]
    fn f1_nodelay_helper_flips_socket_flag() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        assert!(
            !client.nodelay().unwrap(),
            "fresh sockets start with Nagle on"
        );
        super::apply_f1_nodelay(&client);
        super::apply_f1_nodelay(&server);
        assert!(
            client.nodelay().unwrap(),
            "client socket must have TCP_NODELAY"
        );
        assert!(
            server.nodelay().unwrap(),
            "server socket must have TCP_NODELAY"
        );
    }

    #[test]
    fn f1_nodelay_call_sites_present_in_every_tcp_fixture() {
        // bulk.rs: definition + server accept + client connect = 3+;
        // periodic.rs / multistream.rs: accept + connect each = 2+.
        let bulk = include_str!("bulk.rs");
        let periodic = include_str!("periodic.rs");
        let multistream = include_str!("multistream.rs");
        let count = |s: &str| s.matches("apply_f1_nodelay").count();
        assert!(
            count(bulk) >= 3,
            "bulk.rs must keep TCP_NODELAY at accept+connect (found {})",
            count(bulk)
        );
        assert!(
            count(periodic) >= 2,
            "periodic.rs must keep TCP_NODELAY at accept+connect (found {})",
            count(periodic)
        );
        assert!(
            count(multistream) >= 2,
            "multistream.rs must keep TCP_NODELAY at accept+connect (found {})",
            count(multistream)
        );
    }

    /// P1-pre pin (follow-up to b81785e): the measurement fixtures' framing
    /// must stage the length prefix and payload into ONE buffer and issue
    /// ONE write_all per frame. The b81785e NODELAY change made the old
    /// two-write shape emit two TCP segments per record — doubling packet
    /// count on WAN paths (measured 0.77x on the 165ms path) and adding
    /// loopback bimodal noise. write_frame itself intentionally keeps the
    /// two-write shape (failover r9 tests pin segment-timing contracts on
    /// it); bulk/periodic call write_frame_single and multistream's
    /// frame_write stages directly. TcpStream's write_all hides syscall
    /// granularity from behavioral tests, so this pins the source shape.
    #[test]
    fn write_frame_issues_single_write_per_frame() {
        // Strip comment lines before counting: doc comments may legitimately
        // mention write_all while describing the single-write contract.
        let strip_comments = |body: &str| -> String {
            body.lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let main_src = include_str!("main.rs");
        let wf = main_src
            .split("fn write_frame_single(")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("write_frame_single body present");
        let wf = strip_comments(wf);
        assert!(
            wf.contains("staged.extend_from_slice"),
            "write_frame_single must stage header+payload into one buffer"
        );
        assert_eq!(
            wf.matches("write_all").count(),
            1,
            "write_frame_single must issue exactly one write_all per frame (found {})",
            wf.matches("write_all").count()
        );
        // And the fixtures actually route through it.
        let bulk_src = include_str!("bulk.rs");
        let periodic_src = include_str!("periodic.rs");
        assert!(
            bulk_src.matches("write_frame_single(").count() >= 3,
            "bulk.rs data path must use write_frame_single (found {})",
            bulk_src.matches("write_frame_single(").count()
        );
        assert!(
            periodic_src.matches("write_frame_single(").count() >= 3,
            "periodic.rs data path must use write_frame_single (found {})",
            periodic_src.matches("write_frame_single(").count()
        );
        let ms_src = include_str!("multistream.rs");
        let fw = ms_src
            .split("fn frame_write(")
            .nth(1)
            .and_then(|rest| rest.split("fn limits(").next())
            .expect("frame_write body present");
        let fw = strip_comments(fw);
        assert!(
            fw.contains("staged.extend_from_slice"),
            "frame_write must stage header+payload into one buffer"
        );
        assert_eq!(
            fw.matches("write_all").count(),
            1,
            "frame_write must issue exactly one write_all per frame"
        );
    }
}
