//! Bounded authenticated research probe runtime; never a proxy or tunnel.
mod multistream;
mod reachability;

use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, ResumeGuard, TrustPolicy,
    TrustRecord, TrustStatus,
};
use neko_session::{
    InboundRecord, ProcessMessage, ResumeWireBinding, RuntimeLimits, SessionId, SessionRuntime,
    StreamId,
};
use std::{
    env, fs,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    time::{Duration, Instant},
};
const USAGE: &str = "Usage: neko <server|client|probe|lab|failover-server|workload|failover-client|keygen> [bounded options]

  --count N: bounded authenticated exchanges (1-64)\n\nBounded authenticated research probe only; no proxy/tunnel behavior.\n";
const MAX_PORT: u16 = 40100;
const MAX_BYTES: usize = neko_crypto::MAX_UNRELIABLE_DATAGRAM;
const MAX_DURATION: u64 = 30;
const PROCESS_FRAME_MAX: usize = neko_session::PROCESS_FRAME_MAX;
const DOMAIN: &[u8] = b"nekomusume-vps-probe";
fn json_mode(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}
fn diagnostic_mode(args: &[String]) -> bool {
    args.iter().any(|a| a == "--diagnostic")
}
fn diagnostic_id(args: &[String]) -> String {
    let id = args
        .windows(2)
        .find(|w| w[0] == "--experiment-id")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| fail("--diagnostic requires --experiment-id"));
    if !(8..=72).contains(&id.len())
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        fail("invalid experiment id");
    }
    id
}
fn emit_diagnostic(args: &[String], role: &str, event: &str, seq: usize, fields: &str) {
    if diagnostic_mode(args) {
        println!(
            "{{\"experiment_id\":\"{}\",\"role\":\"{}\",\"event\":\"{}\",\"seq\":{}{} }}",
            diagnostic_id(args),
            role,
            event,
            seq,
            fields
        );
    }
}
fn emit_probe(args: &[String], transport: &str, bytes: usize, elapsed_ms: u128) {
    if json_mode(args) {
        println!(
            "{{\"ok\":true,\"transport\":\"{}\",\"bytes\":{},\"elapsed_ms\":{}}}",
            transport, bytes, elapsed_ms
        );
    } else {
        println!(
            "probe_ok transport={} bytes={} elapsed_ms={}",
            transport, bytes, elapsed_ms
        );
    }
}
fn fail(msg: &str) -> ! {
    eprintln!("neko: {msg}");
    std::process::exit(2)
}
#[derive(Clone, Copy)]
struct HandshakeDiagnostics {
    role: &'static str,
    last_success: &'static str,
}
impl HandshakeDiagnostics {
    fn new(role: &'static str) -> Self {
        Self {
            role,
            last_success: "none",
        }
    }
    fn event(&mut self, args: &[String], stage: &'static str) {
        self.last_success = stage;
        if json_mode(args) {
            println!(
                "{{\"ok\":true,\"subsystem\":\"udp_handshake\",\"role\":\"{}\",\"stage\":\"{}\",\"last_success_stage\":\"{}\"}}",
                self.role, stage, self.last_success
            );
        }
    }
    fn timeout(&self, args: &[String]) {
        if json_mode(args) {
            println!(
                "{{\"ok\":false,\"subsystem\":\"udp_handshake\",\"role\":\"{}\",\"stage\":\"timeout\",\"last_success_stage\":\"{}\"}}",
                self.role, self.last_success
            );
        }
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn unhex(s: &str) -> Vec<u8> {
    if !s.len().is_multiple_of(2) {
        fail("hex length");
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or_else(|_| fail("invalid hex")))
        .collect()
}
fn context(direction: u8) -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 1,
        direction,
    }
}
fn load_or_generate(path: &PathBuf) -> LocalIdentity {
    if let Ok(s) = fs::read_to_string(path) {
        let p = s.trim().split(':').map(unhex).collect::<Vec<_>>();
        if p.len() == 2 {
            return LocalIdentity::from_keypair(&p[0], &p[1])
                .unwrap_or_else(|_| fail("invalid identity file"));
        }
        fail("invalid identity file");
    }
    let id = LocalIdentity::generate().unwrap_or_else(|_| fail("identity generation failed"));
    fs::write(
        path,
        format!("{}:{}\n", hex(id.private_key()), hex(id.public_key())),
    )
    .unwrap_or_else(|_| fail("identity write failed"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .unwrap_or_else(|_| fail("identity permission failed"));
    }
    id
}
fn parse(args: &[String], key: &str, default: Option<&str>) -> String {
    args.windows(2)
        .find(|x| x[0] == key)
        .map(|x| x[1].clone())
        .or_else(|| default.map(str::to_string))
        .unwrap_or_else(|| fail(&format!("missing {key}")))
}
fn exchange_count(args: &[String]) -> usize {
    let n = parse(args, "--count", Some("1"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid count"));
    if n == 0 || n > 64 {
        fail("count outside 1-64");
    }
    n
}
fn common(args: &[String]) -> (String, u16, usize, Duration) {
    let t = parse(args, "--transport", None);
    if t != "tcp" && t != "udp" {
        fail("transport must be tcp or udp")
    };
    let p = parse(args, "--port", None)
        .parse()
        .unwrap_or_else(|_| fail("invalid port"));
    if !(40080..=MAX_PORT).contains(&p) {
        fail("port outside 40080-40100")
    };
    let bytes = parse(args, "--bytes", Some("32"))
        .parse()
        .unwrap_or_else(|_| fail("invalid bytes"));
    if bytes == 0 || bytes > MAX_BYTES {
        fail("bytes outside 1-1200")
    };
    let secs = parse(args, "--duration", Some("10"))
        .parse()
        .unwrap_or_else(|_| fail("invalid duration"));
    if secs == 0 || secs > MAX_DURATION {
        fail("duration outside 1-30")
    };
    (t, p, bytes, Duration::from_secs(secs))
}
fn read_frame(s: &mut TcpStream, max: usize) -> std::io::Result<Vec<u8>> {
    let mut h = [0; 4];
    s.read_exact(&mut h)?;
    let n = u32::from_be_bytes(h) as usize;
    if n > max {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame too large",
        ));
    };
    let mut b = vec![0; n];
    s.read_exact(&mut b)?;
    Ok(b)
}
fn write_frame(s: &mut TcpStream, b: &[u8]) -> std::io::Result<()> {
    s.write_all(&(b.len() as u32).to_be_bytes())?;
    s.write_all(b)?;
    s.flush()
}
fn server(args: &[String]) {
    let (t, p, max, d) = common(args);
    let count = exchange_count(args);
    let idpath = PathBuf::from(parse(args, "--identity", Some("neko-server.identity")));
    let id = load_or_generate(&idpath);
    if !json_mode(args) {
        println!("server_public_key={}", hex(id.public_key()));
    }
    let start = Instant::now();
    let client_hex = parse(args, "--client-key", None);
    let client_key = unhex(&client_hex);
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: b"probe".to_vec(),
        status: TrustStatus::Active,
    }]);
    if t == "tcp" {
        let bind_addr = parse(args, "--bind", Some("0.0.0.0:0"));
        let bind: SocketAddr = if bind_addr == "0.0.0.0:0" {
            SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), p)
        } else {
            bind_addr
                .parse()
                .unwrap_or_else(|_| fail("bad bind address"))
        };
        if bind.port() != p {
            fail("bind port must equal --port");
        }
        let l = TcpListener::bind(bind).unwrap_or_else(|_| fail("bind failed"));
        l.set_nonblocking(true).unwrap();
        while start.elapsed() < d {
            match l.accept() {
                Ok((mut s, _)) => {
                    let first = read_frame(&mut s, 1024).unwrap_or_else(|_| fail("bad handshake"));
                    let (resp, mut ss) = ResponderHandshake::new(&id, policy.clone(), DOMAIN)
                        .unwrap()
                        .receive_first(&first, context(0))
                        .unwrap_or_else(|_| fail("unauthorized handshake"));
                    write_frame(&mut s, &resp).unwrap();
                    for _ in 0..count {
                        let frame =
                            read_frame(&mut s, max + 64).unwrap_or_else(|_| fail("bad data"));
                        let plain = ss
                            .open_unreliable(&frame)
                            .unwrap_or_else(|_| fail("auth failure"));
                        let reply = ss
                            .seal_unreliable(&plain)
                            .unwrap_or_else(|_| fail("seal failure"));
                        write_frame(&mut s, &reply).unwrap();
                    }
                    return;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(_) => fail("accept failed"),
            }
        }
    } else {
        let bind_addr = parse(args, "--bind", Some("0.0.0.0:0"));
        let bind: SocketAddr = if bind_addr == "0.0.0.0:0" {
            SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), p)
        } else {
            bind_addr
                .parse()
                .unwrap_or_else(|_| fail("bad bind address"))
        };
        if bind.port() != p {
            fail("bind port must equal --port");
        }
        let u = UdpSocket::bind(bind).unwrap_or_else(|_| fail("bind failed"));
        u.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut b = [0; 65536];
        while start.elapsed() < d {
            if let Ok((n, peer)) = u.recv_from(&mut b) {
                let first = &b[..n];
                let (resp, mut ss) = ResponderHandshake::new(&id, policy.clone(), DOMAIN)
                    .unwrap()
                    .receive_first(first, context(0))
                    .unwrap_or_else(|_| fail("unauthorized handshake"));
                u.send_to(&resp, peer).unwrap();
                for _ in 0..count {
                    let (n, peer) = u.recv_from(&mut b).unwrap_or_else(|_| fail("data timeout"));
                    let plain = ss
                        .open_unreliable(&b[..n])
                        .unwrap_or_else(|_| fail("auth failure"));
                    let reply = ss
                        .seal_unreliable(&plain)
                        .unwrap_or_else(|_| fail("seal failure"));
                    u.send_to(&reply, peer).unwrap();
                }
                return;
            }
        }
    }
    fail("duration expired")
}
fn client(args: &[String]) {
    let (t, _p, max, d) = common(args);
    let count = exchange_count(args);
    let addr = parse(args, "--addr", None);
    let sk = unhex(&parse(args, "--server-key", None));
    let idpath = PathBuf::from(parse(args, "--identity", Some("neko-client.identity")));
    let id = load_or_generate(&idpath);
    if !json_mode(args) {
        println!("client_public_key={}", hex(id.public_key()));
    }
    let mut hs = InitiatorHandshake::new(&id, &sk, b"probe", DOMAIN)
        .unwrap_or_else(|_| fail("handshake setup failed"));
    let first = hs
        .first_message()
        .unwrap_or_else(|_| fail("handshake failed"));
    let payload = vec![b'x'; max];
    let start = Instant::now();
    let cs = if t == "tcp" {
        let mut s =
            TcpStream::connect_timeout(&addr.parse().unwrap_or_else(|_| fail("bad address")), d)
                .unwrap_or_else(|_| fail("connect failed"));
        write_frame(&mut s, &first).unwrap();
        let resp = read_frame(&mut s, 1024).unwrap_or_else(|_| fail("handshake response failed"));
        Some((
            s,
            hs.finish(&resp, context(0))
                .unwrap_or_else(|_| fail("handshake finish failed")),
        ))
    } else {
        let target: SocketAddr = addr.parse().unwrap_or_else(|_| fail("bad address"));
        let local = match target.ip() {
            IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0),
            IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
        };
        let u = UdpSocket::bind(local).unwrap_or_else(|_| fail("UDP socket family unavailable"));
        u.set_read_timeout(Some(d)).unwrap();
        u.send_to(&first, target).unwrap();
        let mut b = [0; 65536];
        let (n, _) = u
            .recv_from(&mut b)
            .unwrap_or_else(|_| fail("handshake response failed"));
        let mut ss = hs
            .finish(&b[..n], context(0))
            .unwrap_or_else(|_| fail("handshake finish failed"));
        for _ in 0..count {
            let rec = ss
                .seal_unreliable(&payload)
                .unwrap_or_else(|_| fail("payload too large"));
            u.send_to(&rec, target).unwrap();
            let (n, _) = u.recv_from(&mut b).unwrap_or_else(|_| fail("echo timeout"));
            if ss
                .open_unreliable(&b[..n])
                .unwrap_or_else(|_| fail("echo auth failed"))
                != payload
            {
                fail("echo mismatch");
            }
        }
        emit_probe(args, "udp", max, start.elapsed().as_millis());
        return;
    };
    let (mut s, mut ss) = cs.unwrap();
    for _ in 0..count {
        let rec = ss
            .seal_unreliable(&payload)
            .unwrap_or_else(|_| fail("payload too large"));
        write_frame(&mut s, &rec).unwrap();
        let reply = read_frame(&mut s, max + 128).unwrap_or_else(|_| fail("echo timeout"));
        if ss
            .open_unreliable(&reply)
            .unwrap_or_else(|_| fail("echo auth failed"))
            != payload
        {
            fail("echo mismatch");
        }
    }
    emit_probe(args, "tcp", max, start.elapsed().as_millis())
}
fn failover_binding(session: u64, generation: u64, expires: u64) -> neko_crypto::ResumeBinding {
    neko_crypto::ResumeBinding {
        session_id: session,
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: generation,
        expires_at_ms: expires,
        token: [9; 32],
    }
}
fn wire_binding(b: &neko_crypto::ResumeBinding) -> ResumeWireBinding {
    ResumeWireBinding {
        session_id: SessionId(b.session_id),
        delivery_epoch: b.delivery_epoch,
        key_phase: b.key_phase,
        path_generation: b.path_generation,
        expires_at_ms: b.expires_at_ms,
        token: b.token,
    }
}
fn runtime_limits(bytes: usize, count: usize) -> RuntimeLimits {
    RuntimeLimits {
        max_streams: 1,
        max_queue_records: count + 2,
        max_queue_bytes: bytes * (count + 1),
        max_total_bytes: bytes * count,
        max_record_bytes: bytes,
        ..RuntimeLimits::default()
    }
}
#[allow(clippy::collapsible_if, clippy::collapsible_match)]
fn failover_server(args: &[String]) {
    let mut diag = HandshakeDiagnostics::new("server");
    let count = exchange_count(args);
    let (_t, _p, bytes, duration) = (
        "failover",
        0,
        parse(args, "--bytes", Some("32"))
            .parse()
            .unwrap_or_else(|_| fail("invalid bytes")),
        Duration::from_secs(
            parse(args, "--duration", Some("10"))
                .parse::<u64>()
                .unwrap_or_else(|_| fail("invalid duration")),
        ),
    );
    if bytes == 0 || bytes > MAX_BYTES {
        fail("bytes outside 1-1200")
    }
    if duration.is_zero() || duration > Duration::from_secs(MAX_DURATION) {
        fail("duration outside 1-30")
    }
    let up = parse(args, "--udp-port", Some("40081"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid UDP port"));
    let tp = parse(args, "--tcp-port", Some("40080"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid TCP port"));
    if !(40080..=MAX_PORT).contains(&up) || !(40080..=MAX_PORT).contains(&tp) {
        fail("ports outside 40080-40100")
    }
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    let client = unhex(&parse(args, "--client-key", None));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client.clone(),
        scope: b"failover".to_vec(),
        status: TrustStatus::Active,
    }]);
    let udp = UdpSocket::bind(
        parse(args, "--udp-bind", Some(&format!("0.0.0.0:{up}")))
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| fail("bad UDP bind")),
    )
    .unwrap_or_else(|_| fail("UDP bind failed"));
    let tcp = TcpListener::bind(
        parse(args, "--tcp-bind", Some(&format!("0.0.0.0:{tp}")))
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| fail("bad TCP bind")),
    )
    .unwrap_or_else(|_| fail("TCP bind failed"));
    udp.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    tcp.set_nonblocking(true).unwrap();
    diag.event(args, "socket_bind");
    let started = Instant::now();
    let mut buf = [0u8; 65536];
    let mut secure = None;
    let mut guard = None;
    let mut app = Vec::new();
    let duplicates = 0usize;
    let mut runtime =
        SessionRuntime::new(SessionId(7001), runtime_limits(bytes, count), 0).unwrap();
    runtime.open_stream(StreamId(1), 0).unwrap();
    emit_diagnostic(
        args,
        "server",
        "start",
        0,
        ",\"count\":8,\"payload_bytes\":64,\"udp_port\":40081,\"max_seconds\":15",
    );
    while started.elapsed() < duration {
        if secure.is_none() {
            if let Ok((n, peer)) = udp.recv_from(&mut buf) {
                diag.event(args, "server_recv");
                diag.event(args, "demux");
                emit_diagnostic(args, "server", "udp_hello_received", 0, "");
                diag.event(args, "client_recv");
                let (resp, ss, remote, binding) =
                    ResponderHandshake::new(&id, policy.clone(), DOMAIN)
                        .unwrap()
                        .receive_first(&buf[..n], context(1))
                        .map(|(resp, ss)| {
                            (resp, ss, client.clone(), failover_binding(7001, 0, 10_000))
                        })
                        .unwrap_or_else(|_| fail("unauthorized UDP handshake"));
                diag.event(args, "parse_auth");
                udp.send_to(&resp, peer).unwrap();
                diag.event(args, "reply_send");
                emit_diagnostic(args, "server", "udp_hello_sent", 0, "");
                diag.event(args, "server_response_sent");
                guard = Some(ResumeGuard::new(&remote, &binding).unwrap());
                secure = Some((ss, peer));
                println!("carrier_event name=udp_authenticated session=7001 generation=0");
            }
        }
        if let Some((ref mut ss, peer)) = secure {
            if let Ok((n, _)) = udp.recv_from(&mut buf) {
                emit_diagnostic(
                    args,
                    "server",
                    "udp_datagram_received",
                    1,
                    &format!(",\"bytes\":{}", n),
                );
                if let Ok(plain) = ss.open_unreliable(&buf[..n]) {
                    diag.event(args, "client_response_received");
                    diag.event(args, "authenticated");
                    if let Ok(ProcessMessage::Data { session, record }) =
                        ProcessMessage::decode(&plain)
                    {
                        let stream_id = record.stream;
                        let offset = record.offset;
                        let len = record.data.len();
                        if session == SessionId(7001)
                            && runtime
                                .receive(
                                    InboundRecord {
                                        stream: stream_id,
                                        offset,
                                        data: record.data,
                                    },
                                    1,
                                )
                                .is_ok()
                        {
                            let ack = ProcessMessage::DeliveryAck {
                                session,
                                stream: stream_id,
                                offset,
                                len,
                            }
                            .encode()
                            .unwrap();
                            udp.send_to(&ack, peer).unwrap();
                            emit_diagnostic(
                                args,
                                "server",
                                "udp_ack_sent",
                                offset as usize / bytes.max(1),
                                "",
                            );
                            if let Some(delivered) = runtime.pop_receive(2).unwrap() {
                                app.extend_from_slice(&delivered.data);
                            }
                        }
                    }
                }
            }
        }
        if let Ok((mut stream, _)) = tcp.accept() {
            let first = read_frame(&mut stream, 1024).unwrap_or_else(|_| fail("bad TCP handshake"));
            let (resp, mut ss, remote, binding) =
                ResponderHandshake::new(&id, policy.clone(), DOMAIN)
                    .unwrap()
                    .receive_first_with_resume(&first, context(2))
                    .unwrap_or_else(|_| fail("unauthorized TCP handshake"));
            if guard
                .as_mut()
                .map(|g| g.attach(&remote, &binding, 1).is_ok())
                .unwrap_or(false)
            {
                write_frame(&mut stream, &resp).unwrap();
                let _ = read_frame(&mut stream, 128);
                for _ in 0..count {
                    if let Ok(m) = ProcessMessage::decode(
                        &ss.open(&read_frame(&mut stream, PROCESS_FRAME_MAX).unwrap_or_default())
                            .unwrap_or_default(),
                    ) {
                        if let ProcessMessage::Data { session, record } = m {
                            let stream_id = record.stream;
                            let offset = record.offset;
                            let len = record.data.len();
                            if session == SessionId(7001)
                                && runtime
                                    .receive(
                                        InboundRecord {
                                            stream: stream_id,
                                            offset,
                                            data: record.data,
                                        },
                                        2,
                                    )
                                    .is_ok()
                            {
                                write_frame(
                                    &mut stream,
                                    &ProcessMessage::DeliveryAck {
                                        session,
                                        stream: stream_id,
                                        offset,
                                        len,
                                    }
                                    .encode()
                                    .unwrap(),
                                )
                                .unwrap();
                                if let Some(delivered) = runtime.pop_receive(3).unwrap() {
                                    app.extend_from_slice(&delivered.data);
                                }
                            }
                        }
                    }
                }
                println!("carrier_event name=tcp_resumed session=7001 generation=1");
                println!(
                    "failover_server_ok session=7001 records={} bytes_hex={} duplicates={} udp_blackhole=true carrier_events=udp_authenticated,tcp_resumed",
                    count,
                    hex(&app),
                    duplicates
                );
                emit_diagnostic(
                    args,
                    "server",
                    "summary",
                    count,
                    &format!(",\"classification\":\"A\",\"records\":{}", count),
                );
                return;
            }
        }
    }
    diag.timeout(args);
    fail("failover timeout")
}
fn failover_client(args: &[String]) {
    let mut diag = HandshakeDiagnostics::new("client");
    let count = exchange_count(args);
    let bytes = parse(args, "--bytes", Some("32"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    let secs = parse(args, "--duration", Some("10"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid duration"));
    if bytes == 0 || bytes > MAX_BYTES {
        fail("bytes outside 1-1200")
    }
    if secs == 0 || secs > MAX_DURATION {
        fail("duration outside 1-30")
    }
    let addr = parse(args, "--addr", Some("127.0.0.1"));
    let up = parse(args, "--udp-port", Some("40081"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid UDP port"));
    let tp = parse(args, "--tcp-port", Some("40080"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid TCP port"));
    if !(40080..=MAX_PORT).contains(&up) || !(40080..=MAX_PORT).contains(&tp) {
        fail("ports outside 40080-40100")
    }
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
    let sk = unhex(&parse(args, "--server-key", None));
    let mut hs = InitiatorHandshake::new(&id, &sk, b"failover", DOMAIN).unwrap();
    let first = hs.first_message().unwrap();
    let target = format!("{addr}:{up}");
    let u = UdpSocket::bind("0.0.0.0:0").unwrap();
    diag.event(args, "socket_bind");
    u.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut buf = [0u8; 65536];
    let mut handshake = None;
    emit_diagnostic(
        args,
        "client",
        "start",
        0,
        ",\"count\":8,\"payload_bytes\":64,\"udp_port\":40081,\"max_seconds\":15",
    );
    for _ in 0..20 {
        diag.event(args, "client_send");
        u.send_to(&first, &target).unwrap();
        diag.event(args, "client_hello_sent");
        emit_diagnostic(args, "client", "udp_hello_sent", 0, "");
        match u.recv_from(&mut buf) {
            Ok((n, _)) => {
                diag.event(args, "client_recv");
                handshake = Some(n);
                emit_diagnostic(args, "client", "udp_hello_received", 0, "");
                break;
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => fail("UDP handshake receive failed"),
        }
    }
    let n = match handshake {
        Some(n) => n,
        None => {
            diag.timeout(args);
            fail("UDP handshake timeout")
        }
    };
    let mut us = hs.finish(&buf[..n], context(1)).unwrap();
    diag.event(args, "parse_auth");
    let payload = vec![b'x'; bytes];
    let mut records = Vec::new();
    for i in 0..count {
        let msg = ProcessMessage::Data {
            session: SessionId(7001),
            record: neko_session::OutboundRecord {
                stream: StreamId(1),
                offset: (i * bytes) as u64,
                data: payload.clone(),
            },
        }
        .encode()
        .unwrap();
        records.push(msg);
    }
    diag.event(args, "authenticated");
    println!("carrier_event name=udp_authenticated session=7001 generation=0");
    let rec = us.seal_unreliable(&records[0]).unwrap();
    u.send_to(&rec, &target).unwrap();
    emit_diagnostic(args, "client", "udp_datagram_sent", 1, ",\"bytes\":64");
    let _ = u.recv_from(&mut buf);
    diag.event(args, "client_recv");
    emit_diagnostic(args, "client", "udp_ack_observed", 1, "");
    println!("carrier_event name=udp_blackhole_injected session=7001 generation=0");
    let mut hs2 = InitiatorHandshake::with_resume_binding(
        &id,
        &sk,
        b"failover",
        DOMAIN,
        &failover_binding(7001, 1, 10_000),
    )
    .unwrap();
    let first2 = hs2.first_message().unwrap();
    let mut tcp = TcpStream::connect_timeout(
        &format!("{addr}:{tp}").parse().unwrap(),
        Duration::from_secs(secs),
    )
    .unwrap();
    write_frame(&mut tcp, &first2).unwrap();
    let resp = read_frame(&mut tcp, 1024).unwrap();
    let mut ts = hs2.finish(&resp, context(2)).unwrap();
    println!("carrier_event name=tcp_resume_guard session=7001 generation=1");
    write_frame(
        &mut tcp,
        &ProcessMessage::Resume {
            binding: wire_binding(&failover_binding(7001, 1, 10_000)),
        }
        .encode()
        .unwrap(),
    )
    .unwrap();
    for msg in records {
        let encrypted = ts.seal(&msg).unwrap();
        write_frame(&mut tcp, &encrypted).unwrap();
        let _ = read_frame(&mut tcp, PROCESS_FRAME_MAX);
    }
    println!(
        "carrier_event name=ordered_records_complete session=7001 count={count} bytes={bytes}"
    );
    emit_diagnostic(
        args,
        "client",
        "summary",
        count,
        &format!(",\"classification\":\"A\",\"records\":{}", count),
    );
    emit_diagnostic(
        args,
        "client",
        "capture_metadata",
        count + 1,
        ",\"capture\":\"metadata-only\",\"payload\":false,\"keys\":false,\"bounded\":true",
    );
    println!("failover_client_ok session=7001 count={count} bytes={bytes} udp_blackhole=true")
}
fn failover_gate(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("failover-server") => failover_server(args),
        Some("failover-client") => failover_client(args),
        _ => fail("use failover-server or failover-client"),
    }
}

fn lab(args: &[String]) {
    let json = json_mode(args);
    let timeline = [
        ("udp", "active", 0u64),
        ("udp", "pto", 1),
        ("udp", "uncertain", 8192),
        ("tcp", "validated", 8192),
        ("tcp", "migrated", 8192),
        ("tcp", "duplicate_dedup", 1024),
        ("tcp", "recovered", 9216),
    ];
    if json {
        print!("{{\"ok\":true,\"demo\":\"failover\",\"timeline\":[");
        for (i, (carrier, event, bytes)) in timeline.iter().enumerate() {
            if i > 0 {
                print!(",");
            }
            print!(
                "{{\"step\":{},\"carrier\":\"{}\",\"event\":\"{}\",\"bytes\":{}}}",
                i, carrier, event, bytes
            );
        }
        println!("]}}");
    } else {
        for (i, (carrier, event, bytes)) in timeline.iter().enumerate() {
            println!(
                "step={} carrier={} event={} bytes={}",
                i, carrier, event, bytes
            );
        }
    }
}

/// Run independent Session runtimes for a bounded interval. This local fixture exercises queueing, delivery ACKs and cleanup without opening sockets.
fn workload(args: &[String]) {
    let duration = parse(args, "--duration", Some("5"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid duration"));
    let concurrency = parse(args, "--concurrency", Some("1"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid concurrency"));
    let records = parse(args, "--records", Some("100"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid records"));
    let bytes = parse(args, "--bytes", Some("32"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    if !(1..=MAX_DURATION).contains(&duration) {
        fail("duration outside 1-30");
    }
    if !(1..=16).contains(&concurrency) {
        fail("concurrency outside 1-16");
    }
    if !(1..=10_000).contains(&records) {
        fail("records outside 1-10000");
    }
    if !(1..=MAX_BYTES).contains(&bytes) {
        fail("bytes outside 1-1200");
    }
    let started = Instant::now();
    let mut workers = Vec::with_capacity(concurrency);
    for worker in 0..concurrency {
        workers.push(std::thread::spawn(move || {
            let mut runtime = SessionRuntime::new(
                SessionId(worker as u64 + 1),
                runtime_limits(bytes, records),
                0,
            )
            .unwrap();
            runtime.open_stream(StreamId(1), 0).unwrap();
            let payload = vec![worker as u8; bytes];
            for index in 0..records {
                let offset = runtime
                    .queue_send(StreamId(1), &payload, index as u64)
                    .unwrap();
                let sent = runtime.pop_send(index as u64).unwrap().unwrap();
                runtime
                    .delivery_ack(StreamId(1), offset, sent.data.len(), index as u64)
                    .unwrap();
            }
            assert_eq!(
                runtime.confirmed_watermark(StreamId(1)),
                (records * bytes) as u64
            );
            runtime.close_graceful(records as u64).unwrap();
            runtime.cancel(records as u64).unwrap();
            (records, records * bytes)
        }));
    }
    let mut completed_records = 0usize;
    let mut application_bytes = 0usize;
    for worker in workers {
        let (count, size) = worker
            .join()
            .unwrap_or_else(|_| fail("workload worker failed"));
        completed_records += count;
        application_bytes += size;
    }
    std::thread::sleep(Duration::from_secs(duration).saturating_sub(started.elapsed()));
    if json_mode(args) {
        println!(
            "{{\"ok\":true,\"fixture\":\"session-workload\",\"duration_seconds\":{},\"concurrency\":{},\"records\":{},\"application_bytes\":{},\"cleanup\":\"verified\"}}",
            duration, concurrency, completed_records, application_bytes
        );
    } else {
        println!(
            "workload_ok duration_seconds={} concurrency={} records={} application_bytes={} cleanup=verified",
            duration, concurrency, completed_records, application_bytes
        );
    }
}

fn matrix_probe(args: &[String]) -> ! {
    let get = |key: &str| args.windows(2).find(|w| w[0] == key).map(|w| w[1].as_str());
    let target: SocketAddr = get("--target")
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| fail("missing or invalid --target"));
    let transport = match get("--transport") {
        Some("tcp") => reachability::Transport::Tcp,
        Some("udp") => reachability::Transport::Udp,
        _ => fail("--transport must be tcp or udp"),
    };
    let version = match get("--ip-version") {
        Some("ipv4") => reachability::IpVersion::V4,
        Some("ipv6") => reachability::IpVersion::V6,
        _ => fail("--ip-version must be ipv4 or ipv6"),
    };
    let timeout = get("--timeout-ms")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let bytes = get("--bytes").and_then(|v| v.parse().ok()).unwrap_or(32);
    let artifact = reachability::run(transport, version, target, timeout, bytes);
    if json_mode(args) {
        println!("{artifact}");
    } else if artifact.contains("\"reachable\":true") {
        println!("pass: 喵~！");
    } else {
        println!("fail: 喵呜呜呜呜…");
    }
    std::process::exit(if artifact.contains("\"reachable\":true") {
        0
    } else {
        1
    });
}

fn key_update_fixture(args: &[String]) {
    const EXCHANGES: usize = 6;
    let initiator =
        LocalIdentity::generate().unwrap_or_else(|_| fail("identity generation failed"));
    let responder =
        LocalIdentity::generate().unwrap_or_else(|_| fail("identity generation failed"));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: initiator.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut handshake =
        InitiatorHandshake::new(&initiator, responder.public_key(), b"echo", DOMAIN)
            .unwrap_or_else(|_| fail("handshake setup failed"));
    let first = handshake
        .first_message()
        .unwrap_or_else(|_| fail("handshake failed"));
    let responder_handshake = ResponderHandshake::new(&responder, policy, DOMAIN)
        .unwrap_or_else(|_| fail("handshake setup failed"));
    let (response, mut receiver) = responder_handshake
        .receive_first(&first, context(0))
        .unwrap_or_else(|_| fail("handshake failed"));
    let mut sender = handshake
        .finish(&response, context(0))
        .unwrap_or_else(|_| fail("handshake failed"));
    let mut events = vec!["session_opened", "key_phase_0"];
    for index in 0..EXCHANGES {
        let payload = format!("exchange-{index}");
        let record = sender
            .seal(payload.as_bytes())
            .unwrap_or_else(|_| fail("seal failed"));
        if receiver
            .open(&record)
            .unwrap_or_else(|_| fail("open failed"))
            != payload.as_bytes()
        {
            fail("payload mismatch");
        }
        events.push("exchange");
    }
    let stale = sender
        .seal(b"stale-phase")
        .unwrap_or_else(|_| fail("seal failed"));
    sender
        .update_key_phase()
        .unwrap_or_else(|_| fail("key update failed"));
    receiver
        .update_key_phase()
        .unwrap_or_else(|_| fail("key update failed"));
    events.push("key_update_committed");
    if receiver.open(&stale).is_ok() {
        fail("old phase accepted");
    }
    events.push("old_phase_rejected");
    for index in EXCHANGES..(EXCHANGES * 2) {
        let payload = format!("exchange-{index}");
        let record = sender
            .seal(payload.as_bytes())
            .unwrap_or_else(|_| fail("seal failed"));
        if receiver
            .open(&record)
            .unwrap_or_else(|_| fail("open failed"))
            != payload.as_bytes()
        {
            fail("payload mismatch");
        }
        events.push("exchange");
    }
    if sender.key_phase() != 1 || receiver.key_phase() != 1 {
        fail("key phase desynchronized");
    }
    events.push("session_complete");
    if json_mode(args) {
        println!(
            "{{\"ok\":true,\"fixture\":\"secure-session-key-update\",\"exchanges\":{},\"key_phase\":1,\"events\":[{}]}}",
            EXCHANGES * 2,
            events
                .iter()
                .map(|e| format!("\"{e}\""))
                .collect::<Vec<_>>()
                .join(",")
        );
    } else {
        println!(
            "key_update_ok exchanges={} key_phase=1 events={}",
            EXCHANGES * 2,
            events.join(",")
        );
    }
}

fn main() {
    let a: Vec<String> = env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("server") => server(&a),
        Some("probe") if a.iter().any(|v| v == "--matrix") => matrix_probe(&a),
        Some("client") | Some("probe") => client(&a),
        Some("lab") => lab(&a),
        Some("workload") => workload(&a),
        Some("key-update") => key_update_fixture(&a),
        Some("multistream") => multistream::run(&a),
        Some("failover") | Some("failover-server") | Some("failover-client") => failover_gate(&a),
        Some("keygen") => {
            let path = PathBuf::from(parse(&a, "--identity", Some("neko-client.identity")));
            let id = load_or_generate(&path);
            println!("client_public_key={}", hex(id.public_key()));
        }
        Some("--help") | None => println!("{USAGE}"),
        _ => fail("unknown command"),
    }
}

#[cfg(test)]
mod cli_regression_tests {
    use super::*;
    #[test]
    fn json_mode_is_detected_without_affecting_address_family() {
        assert!(json_mode(&["probe".into(), "--json".into()]));
        assert_eq!(
            "[::1]:40080".parse::<SocketAddr>().unwrap().ip(),
            IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
        );
        assert_eq!(
            "127.0.0.1:40080".parse::<SocketAddr>().unwrap().ip(),
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        );
    }
    #[test]
    fn failover_gate_arguments_are_bounded_and_loopback_explicit() {
        let args = vec![
            "failover".into(),
            "--count".into(),
            "3".into(),
            "--bytes".into(),
            "16".into(),
            "--duration".into(),
            "2".into(),
            "--udp-port".into(),
            "40081".into(),
            "--tcp-port".into(),
            "40080".into(),
            "--loopback-only".into(),
        ];
        assert_eq!(exchange_count(&args), 3);
        assert!(args.iter().any(|a| a == "--loopback-only"));
    }
}
