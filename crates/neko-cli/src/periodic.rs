//! One bounded authenticated TCP Session carrying periodic logical records.
use super::*;
use crate::framed::{FrameRead, FramedReader};
use std::collections::BTreeSet;

const SESSION: SessionId = SessionId(7201);
const STREAM: StreamId = StreamId(1);
const MAX_COUNT: usize = 600;
const MAX_TOTAL_BYTES: usize = 1024 * 1024;
const MIN_INTERVAL_MS: u64 = 100;
const MAX_INTERVAL_MS: u64 = 5_000;
const DEFAULT_SETUP_TIMEOUT_MS: u64 = 5_000;
const MAX_SETUP_TIMEOUT_MS: u64 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Config {
    port: u16,
    bytes: usize,
    count: usize,
    duration: Duration,
    interval: Duration,
    setup_timeout: Duration,
    ack_timeout: Duration,
}

fn value(args: &[String], key: &str, default: &str) -> Result<String, &'static str> {
    Ok(args
        .windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default.to_owned()))
}

fn config(args: &[String]) -> Result<Config, &'static str> {
    let parse_u64 = |key, default| {
        value(args, key, default)?
            .parse::<u64>()
            .map_err(|_| "invalid numeric argument")
    };
    let port = parse_u64("--port", "40080")?;
    let bytes = parse_u64("--bytes", "32")?;
    let count = parse_u64("--count", "1")?;
    let duration = parse_u64("--duration", "10")?;
    let interval = parse_u64("--interval-ms", "1000")?;
    let setup_timeout_default = DEFAULT_SETUP_TIMEOUT_MS.to_string();
    let setup_timeout = parse_u64("--setup-timeout-ms", &setup_timeout_default)?;
    let ack_timeout = parse_u64("--ack-timeout-ms", "1000")?;
    if !(40080..=MAX_PORT as u64).contains(&port) {
        return Err("port outside 40080-40100");
    }
    if bytes == 0 || bytes > MAX_BYTES as u64 {
        return Err("bytes outside 1-1200");
    }
    if count == 0 || count > MAX_COUNT as u64 {
        return Err("count outside 1-600");
    }
    if duration == 0 || duration > MAX_WORKLOAD_DURATION {
        return Err("duration outside 1-600");
    }
    if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&interval) {
        return Err("interval-ms outside 100-5000");
    }
    if setup_timeout == 0 || setup_timeout > MAX_SETUP_TIMEOUT_MS {
        return Err("setup-timeout-ms outside 1-10000");
    }
    if ack_timeout == 0 || ack_timeout > 10_000 {
        return Err("ack-timeout-ms outside 1-10000");
    }
    let total = bytes
        .checked_mul(count)
        .ok_or("application byte bound overflow")?;
    if total > MAX_TOTAL_BYTES as u64 {
        return Err("application bytes exceed 1048576");
    }
    Ok(Config {
        port: port as u16,
        bytes: bytes as usize,
        count: count as usize,
        duration: Duration::from_secs(duration),
        interval: Duration::from_millis(interval),
        setup_timeout: Duration::from_millis(setup_timeout),
        ack_timeout: Duration::from_millis(ack_timeout),
    })
}

fn ack_matches(plain: &[u8], expected: &OutboundRecord) -> bool {
    matches!(
        ProcessMessage::decode(plain),
        Ok(ProcessMessage::DeliveryAck { session, stream, offset, len })
            if session == SESSION && stream == expected.stream
                && offset == expected.offset && len == expected.data.len()
    )
}

fn percentile(sorted: &[u128], numerator: usize, denominator: usize) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    sorted[((sorted.len() * numerator).div_ceil(denominator))
        .saturating_sub(1)
        .min(sorted.len() - 1)]
}

fn install_shutdown() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    flag::register(SIGINT, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    shutdown
}

fn limits(cfg: Config) -> RuntimeLimits {
    RuntimeLimits {
        max_streams: 1,
        max_queue_records: cfg.count + 2,
        max_queue_bytes: cfg.bytes * (cfg.count + 1),
        max_total_bytes: cfg.bytes * cfg.count * 2,
        max_record_bytes: cfg.bytes,
        max_stream_window: cfg.bytes * cfg.count,
        max_session_window: cfg.bytes * cfg.count,
        ..RuntimeLimits::default()
    }
}

fn frame_or_fail(
    reader: &mut FramedReader,
    stream: &mut TcpStream,
    max_frame_len: usize,
    deadline: Instant,
    message: &str,
) -> Vec<u8> {
    reader
        .set_max_frame_len(max_frame_len)
        .unwrap_or_else(|_| fail("partial frame crossed protocol stage"));
    match reader.read_until(stream, deadline) {
        Ok(FrameRead::Complete(frame)) => frame,
        Ok(FrameRead::Deadline | FrameRead::CleanEof | FrameRead::Truncated) | Err(_) => {
            fail(message)
        }
    }
}

fn bound_setup(stream: &TcpStream, deadline: Instant, message: &str) {
    bound_stream_to_deadline(stream, deadline, None).unwrap_or_else(|_| fail(message));
}

fn handshake_server(
    stream: &mut TcpStream,
    reader: &mut FramedReader,
    deadline: Instant,
    id: &LocalIdentity,
    policy: TrustPolicy,
    admission: &mut preauth::ListenerAdmission,
    ticket: &mut preauth::AdmissionTicket,
) -> neko_crypto::SecureSession {
    bound_setup(stream, deadline, "setup deadline elapsed");
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
    let hello = frame_or_fail(
        reader,
        stream,
        MAX_NEGOTIATION_FRAME,
        deadline,
        "malformed negotiation",
    );
    admission
        .charge_input(ticket, hello.len() + 4, 64)
        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
    let selection = negotiation
        .server_accept_hello(&hello)
        .unwrap_or_else(|_| fail("incompatible negotiation"));
    bound_setup(stream, deadline, "setup deadline elapsed");
    let response_permit_1 = admission
        .charge_response(ticket, selection.len() + 4)
        .unwrap_or_else(|_| fail("pre-auth response rejected"));
    write_frame(stream, &selection).unwrap_or_else(|_| fail("negotiation response failed"));
    admission
        .complete_response(response_permit_1)
        .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
    let binding = negotiation.authenticated_binding().unwrap();
    let first = frame_or_fail(reader, stream, 1024, deadline, "bad handshake");
    admission
        .charge_input(ticket, first.len() + 4, 4096)
        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
    let (response, secure) =
        ResponderHandshake::new_with_prologue_binding(id, policy, DOMAIN, binding.as_bytes())
            .unwrap()
            .receive_first(&first, context(0))
            .unwrap_or_else(|_| fail("unauthorized handshake"));
    bound_setup(stream, deadline, "setup deadline elapsed");
    let response_permit_2 = admission
        .charge_response(ticket, response.len() + 4)
        .unwrap_or_else(|_| fail("pre-auth response rejected"));
    write_frame(stream, &response).unwrap_or_else(|_| fail("handshake response failed"));
    admission
        .complete_response(response_permit_2)
        .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
    negotiation
        .admit_data()
        .unwrap_or_else(|_| fail("data admission denied"));
    secure
}

fn handshake_client(
    stream: &mut TcpStream,
    reader: &mut FramedReader,
    deadline: Instant,
    id: &LocalIdentity,
    server_key: &[u8],
) -> neko_crypto::SecureSession {
    bound_setup(stream, deadline, "setup deadline elapsed");
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
    let hello = negotiation.client_hello().unwrap();
    bound_setup(stream, deadline, "setup deadline elapsed");
    write_frame(stream, &hello).unwrap_or_else(|_| fail("negotiation send failed"));
    let selection = frame_or_fail(
        reader,
        stream,
        MAX_NEGOTIATION_FRAME,
        deadline,
        "negotiation response failed",
    );
    negotiation
        .client_accept_response(&selection)
        .unwrap_or_else(|_| fail("incompatible negotiation"));
    let binding = negotiation.authenticated_binding().unwrap();
    let mut hs = InitiatorHandshake::new_with_prologue_binding(
        id,
        server_key,
        b"probe",
        DOMAIN,
        binding.as_bytes(),
    )
    .unwrap();
    let first = hs.first_message().unwrap();
    bound_setup(stream, deadline, "setup deadline elapsed");
    write_frame(stream, &first).unwrap_or_else(|_| fail("handshake send failed"));
    let response = frame_or_fail(reader, stream, 1024, deadline, "handshake response failed");
    let secure = hs
        .finish(&response, context(0))
        .unwrap_or_else(|_| fail("handshake finish failed"));
    negotiation
        .admit_data()
        .unwrap_or_else(|_| fail("data admission denied"));
    secure
}

pub(super) fn server(args: &[String]) {
    let cfg = config(args).unwrap_or_else(|e| fail(e));
    let shutdown = install_shutdown();
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    println!("server_public_key={}", hex(id.public_key()));
    let client_key = unhex(&parse(args, "--client-key", None));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: b"probe".to_vec(),
        status: TrustStatus::Active,
    }]);
    let bind = parse(args, "--bind", Some(&format!("0.0.0.0:{}", cfg.port)))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad bind address"));
    if bind.port() != cfg.port {
        fail("bind port must equal --port");
    }
    let listener = TcpListener::bind(bind).unwrap_or_else(|_| fail("bind failed"));
    listener.set_nonblocking(true).unwrap();
    println!(
        "periodic_server_ready transport=tcp port={} reconnect=unsupported",
        cfg.port
    );
    std::io::stdout().flush().unwrap();
    let start = Instant::now();
    let mut preauth = preauth::ListenerAdmission::new();
    let mut accepted = None;
    while start.elapsed() < cfg.duration && !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, peer)) => {
                let ticket = preauth
                    .admit(peer)
                    .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                accepted = Some((stream, ticket));
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(_) => fail("accept failed"),
        }
    }
    let Some((mut stream, mut admission)) = accepted else {
        println!(
            "periodic_server_summary authenticated=false received=0 confirmed=0 duplicates=0 elapsed_ms={} cleanup=verified",
            start.elapsed().as_millis()
        );
        if shutdown.load(Ordering::Acquire) {
            return;
        }
        fail("duration expired before authenticated Session");
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut framed = FramedReader::new(PROCESS_FRAME_MAX + 128);
    let accepted_at = Instant::now();
    let setup_deadline = (accepted_at + cfg.setup_timeout).min(start + cfg.duration);
    let setup_delay = args
        .windows(2)
        .find(|w| w[0] == "--test-setup-delay-ms")
        .map(|w| {
            w[1].parse::<u64>()
                .unwrap_or_else(|_| fail("invalid setup delay"))
        })
        .unwrap_or(0);
    if setup_delay > MAX_SETUP_TIMEOUT_MS {
        fail("setup delay outside 0-10000");
    }
    if setup_delay > 0 {
        std::thread::sleep(Duration::from_millis(setup_delay));
    }
    let mut secure = handshake_server(
        &mut stream,
        &mut framed,
        setup_deadline,
        &id,
        policy,
        &mut preauth,
        &mut admission,
    );
    preauth.release(admission);
    framed.set_max_frame_len(PROCESS_FRAME_MAX + 128).unwrap();
    println!("periodic_server_authenticated session=7201 stream=1");
    let mut runtime = SessionRuntime::new(SESSION, limits(cfg), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let deadline = start + cfg.duration;
    let ack_delay = args
        .windows(2)
        .find(|w| w[0] == "--test-ack-delay-ms")
        .map(|w| {
            w[1].parse::<u64>()
                .unwrap_or_else(|_| fail("invalid ack delay"))
        })
        .unwrap_or(0);
    if ack_delay > 10_000 {
        fail("ack delay outside 0-10000");
    }
    let duplicate_ack = args.iter().any(|arg| arg == "--test-duplicate-ack");
    let drop_ack = args
        .windows(2)
        .find(|w| w[0] == "--test-drop-ack")
        .map(|w| {
            w[1].parse::<usize>()
                .unwrap_or_else(|_| fail("invalid drop ack"))
        });
    let mut received = 0usize;
    let mut confirmed = 0usize;
    let mut duplicates = 0usize;
    while Instant::now() < deadline && received < cfg.count && !shutdown.load(Ordering::Acquire) {
        match framed.read_until(&mut stream, deadline) {
            Ok(FrameRead::Complete(frame)) => {
                let plain = secure
                    .open_unreliable(&frame)
                    .unwrap_or_else(|_| fail("unauthenticated periodic data"));
                let ProcessMessage::Data { session, record } = ProcessMessage::decode(&plain)
                    .unwrap_or_else(|_| fail("malformed periodic data"))
                else {
                    fail("expected periodic data");
                };
                if session != SESSION || record.stream != STREAM || record.data.len() != cfg.bytes {
                    fail("periodic record outside contract");
                }
                let before = runtime.events().count();
                runtime
                    .receive(
                        InboundRecord {
                            stream: record.stream,
                            offset: record.offset,
                            data: record.data.clone(),
                        },
                        start.elapsed().as_millis() as u64,
                    )
                    .unwrap_or_else(|_| fail("periodic record rejected"));
                let _ = runtime
                    .pop_receive(start.elapsed().as_millis() as u64)
                    .unwrap();
                if runtime
                    .events()
                    .skip(before)
                    .any(|e| matches!(e.kind, neko_session::RuntimeEventKind::DuplicateDedup))
                {
                    duplicates += 1;
                }
                let ack = ProcessMessage::DeliveryAck {
                    session: SESSION,
                    stream: STREAM,
                    offset: record.offset,
                    len: record.data.len(),
                }
                .encode()
                .unwrap();
                received += 1;
                let send_ack = drop_ack != Some(received);
                if send_ack {
                    if ack_delay > 0 {
                        std::thread::sleep(Duration::from_millis(ack_delay));
                    }
                    let encrypted = secure.seal_unreliable(&ack).unwrap();
                    write_frame(&mut stream, &encrypted)
                        .unwrap_or_else(|_| fail("delivery acknowledgement send failed"));
                    if duplicate_ack {
                        let duplicate = secure.seal_unreliable(&ack).unwrap();
                        write_frame(&mut stream, &duplicate)
                            .unwrap_or_else(|_| fail("duplicate acknowledgement send failed"));
                    }
                    confirmed += 1;
                }
                println!(
                    "periodic_server_interval seq={} received=true confirmed={} duplicate={}",
                    received,
                    send_ack,
                    duplicates > 0
                );
            }
            Ok(FrameRead::Deadline) => break,
            Ok(FrameRead::CleanEof | FrameRead::Truncated) | Err(_) => {
                fail("periodic Session disconnected; reconnect/resume unsupported")
            }
        }
    }
    let now = start.elapsed().as_millis() as u64;
    let _ = runtime.close_stream(STREAM, now);
    let _ = runtime.close_graceful(now);
    println!(
        "periodic_server_summary authenticated=true received={} confirmed={} duplicates={} elapsed_ms={} cleanup=verified signal={}",
        received,
        confirmed,
        duplicates,
        start.elapsed().as_millis(),
        shutdown.load(Ordering::Acquire)
    );
    std::io::stdout().flush().unwrap();
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

pub(super) fn client(args: &[String]) {
    let cfg = config(args).unwrap_or_else(|e| fail(e));
    let shutdown = install_shutdown();
    let addr = parse(args, "--addr", None)
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad address"));
    if addr.port() != cfg.port {
        fail("address port must equal --port");
    }
    let server_key = unhex(&parse(args, "--server-key", None));
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
    println!("client_public_key={}", hex(id.public_key()));
    let start = Instant::now();
    let mut stream = TcpStream::connect_timeout(&addr, cfg.setup_timeout)
        .unwrap_or_else(|_| fail("connect failed; reconnect/resume unsupported"));
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut framed = FramedReader::new(PROCESS_FRAME_MAX + 128);
    let mut secure = handshake_client(
        &mut stream,
        &mut framed,
        start + cfg.setup_timeout,
        &id,
        &server_key,
    );
    framed.set_max_frame_len(PROCESS_FRAME_MAX + 128).unwrap();
    println!("periodic_client_authenticated session=7201 stream=1 reconnect=unsupported");
    let mut runtime = SessionRuntime::new(SESSION, limits(cfg), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let deadline = start + cfg.duration;
    let payload = vec![b'p'; cfg.bytes];
    let mut attempted = 0usize;
    let mut confirmed = 0usize;
    let mut missing = 0usize;
    let mut duplicates = 0usize;
    let reconnects = 0usize;
    let mut latencies = Vec::new();
    let mut acknowledged = BTreeSet::new();
    for seq in 1..=cfg.count {
        if shutdown.load(Ordering::Acquire) || Instant::now() >= deadline {
            break;
        }
        let scheduled = start + cfg.interval * (seq as u32 - 1);
        while Instant::now() < scheduled && !shutdown.load(Ordering::Acquire) {
            std::thread::sleep(
                Duration::from_millis(10).min(scheduled.saturating_duration_since(Instant::now())),
            );
        }
        if shutdown.load(Ordering::Acquire) || Instant::now() >= deadline {
            break;
        }
        runtime
            .queue_send(STREAM, &payload, start.elapsed().as_millis() as u64)
            .unwrap();
        let record = runtime
            .pop_send(start.elapsed().as_millis() as u64)
            .unwrap()
            .unwrap();
        let plain = ProcessMessage::Data {
            session: SESSION,
            record: record.clone(),
        }
        .encode()
        .unwrap();
        let encrypted = secure.seal_unreliable(&plain).unwrap();
        attempted += 1;
        let sent_at = Instant::now();
        write_frame(&mut stream, &encrypted).unwrap_or_else(|_| {
            fail("periodic Session disconnected; reconnect/resume unsupported")
        });
        let mut ok = false;
        loop {
            match framed.read_until(&mut stream, sent_at + cfg.ack_timeout) {
                Ok(FrameRead::Complete(frame)) => {
                    let plain = secure
                        .open_unreliable(&frame)
                        .unwrap_or_else(|_| fail("unauthenticated delivery acknowledgement"));
                    let decoded = ProcessMessage::decode(&plain)
                        .unwrap_or_else(|_| fail("malformed delivery acknowledgement"));
                    if ack_matches(&plain, &record) {
                        let key = (record.offset, record.data.len());
                        if !acknowledged.insert(key) {
                            duplicates += 1;
                            continue;
                        }
                        runtime
                            .delivery_ack(
                                record.stream,
                                record.offset,
                                record.data.len(),
                                start.elapsed().as_millis() as u64,
                            )
                            .unwrap();
                        confirmed += 1;
                        latencies.push(sent_at.elapsed().as_millis());
                        ok = true;
                        break;
                    }
                    if let ProcessMessage::DeliveryAck {
                        session,
                        stream,
                        offset,
                        len,
                    } = decoded
                        && session == SESSION
                        && stream == STREAM
                        && acknowledged.contains(&(offset, len))
                    {
                        duplicates += 1;
                        continue;
                    }
                    fail("non-matching authenticated delivery acknowledgement");
                }
                Ok(FrameRead::Deadline | FrameRead::CleanEof) => {
                    missing += 1;
                    break;
                }
                Ok(FrameRead::Truncated) | Err(_) => {
                    fail("periodic Session disconnected; reconnect/resume unsupported")
                }
            }
        }
        println!(
            "periodic_interval seq={} sent=true confirmed={} missing={} duplicate=false latency_ms={}",
            seq,
            ok,
            !ok,
            if ok {
                sent_at.elapsed().as_millis().to_string()
            } else {
                "null".into()
            }
        );
    }
    latencies.sort_unstable();
    let elapsed = start.elapsed().as_millis();
    if missing == 0 {
        runtime.close_stream(STREAM, elapsed as u64).unwrap();
        runtime.close_graceful(elapsed as u64).unwrap();
    } else {
        runtime.close_remote(elapsed as u64).unwrap();
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
    println!(
        "periodic_summary transport=tcp session=7201 stream=1 attempted={} confirmed={} missing={} duplicates={} p50_confirmation_latency_ms={} p95_confirmation_latency_ms={} reconnects={} elapsed_ms={} application_bytes={} cleanup=verified signal={}",
        attempted,
        confirmed,
        missing,
        duplicates,
        percentile(&latencies, 50, 100),
        percentile(&latencies, 95, 100),
        reconnects,
        elapsed,
        attempted * cfg.bytes,
        shutdown.load(Ordering::Acquire)
    );
    if missing > 0 {
        fail("periodic Session completed with missing records");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn bounds_fail_closed() {
        assert!(config(&args(&["periodic-client", "--duration", "0"])).is_err());
        assert!(config(&args(&["periodic-client", "--duration", "601"])).is_err());
        assert!(config(&args(&["periodic-client", "--interval-ms", "99"])).is_err());
        assert!(config(&args(&["periodic-client", "--count", "601"])).is_err());
        assert!(
            config(&args(&[
                "periodic-client",
                "--bytes",
                "1200",
                "--count",
                "600"
            ]))
            .is_ok()
        );
    }
    #[test]
    fn percentile_is_nearest_rank() {
        assert_eq!(percentile(&[1, 2, 3, 100], 50, 100), 2);
        assert_eq!(percentile(&[1, 2, 3, 100], 95, 100), 100);
        assert_eq!(percentile(&[], 95, 100), 0);
    }
}
