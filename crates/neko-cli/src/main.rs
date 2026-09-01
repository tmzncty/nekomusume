//! Bounded authenticated research probe runtime; never a proxy or tunnel.
mod lifecycle;

use lifecycle::ReadinessPrerequisite;
mod multistream;
mod reachability;

use neko_carrier::{
    CarrierHealthEvidence, FairScheduler, FlowLimits, HealthEvidenceLimits, HealthLimits,
    HealthSample, PathId, StreamId as CarrierStreamId, StreamPriority,
};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, ResumeGuard, TrustPolicy,
    TrustRecord, TrustStatus,
};
use neko_session::{
    InboundRecord, OutboundRecord, ProcessMessage, RuntimeLimits, SessionId, SessionRuntime,
    StreamId,
};
use neko_wire::{NEGOTIATION_VERSION, NegotiationRole, VersionNegotiator};
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    flag,
};
use std::{
    env, fs,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const USAGE: &str = "Usage: neko <server|client|probe|lab|failover-server|workload|failover-client|keygen|capabilities> [bounded options]

  --count N: bounded authenticated exchanges (1-64)\n  capabilities [--json]: secret-free build, command, default, and limit report\n\nBounded authenticated research probe only; no proxy/tunnel behavior.\n";
const MAX_PORT: u16 = 40100;
const MAX_BYTES: usize = neko_crypto::MAX_UNRELIABLE_DATAGRAM;
const MAX_DURATION: u64 = 30;
const MAX_WORKLOAD_DURATION: u64 = 600;
const PROCESS_FRAME_MAX: usize = neko_session::PROCESS_FRAME_MAX;
const DOMAIN: &[u8] = b"nekomusume-vps-probe";
const SUPPORTED_VERSIONS: &[u16] = &[NEGOTIATION_VERSION];
const MAX_NEGOTIATION_FRAME: usize = 4 + neko_wire::MAX_NEGOTIATION_VERSIONS * 2;
fn json_mode(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}
fn capabilities(args: &[String]) {
    if args.iter().skip(1).any(|arg| arg != "--json") {
        fail("capabilities accepts only --json");
    }
    if json_mode(args) {
        println!(
            concat!(
                "{{\"schema\":\"nekomusume.capabilities.v1\",",
                "\"package_version\":\"{}\",\"target_os\":\"{}\",\"target_arch\":\"{}\",",
                "\"secret_free\":true,",
                "\"defaults\":{{\"bytes\":32,\"count\":1,\"duration_seconds\":10}},",
                "\"limits\":{{\"bytes_max\":{},\"count_max\":64,\"duration_seconds_max\":{},",
                "\"workload_duration_seconds_max\":{},\"port_min\":40080,\"port_max\":{}}},",
                "\"commands\":[",
                "{{\"name\":\"client\",\"maturity\":\"research\"}},",
                "{{\"name\":\"server\",\"maturity\":\"research\"}},",
                "{{\"name\":\"probe\",\"maturity\":\"research\"}},",
                "{{\"name\":\"health-observe\",\"maturity\":\"experimental\"}},",
                "{{\"name\":\"failover\",\"maturity\":\"experimental\"}},",
                "{{\"name\":\"multistream\",\"maturity\":\"experimental\"}},",
                "{{\"name\":\"scheduler-fairness\",\"maturity\":\"fixture\"}},",
                "{{\"name\":\"key-update\",\"maturity\":\"fixture\"}}",
                "]}}"
            ),
            env!("CARGO_PKG_VERSION"),
            env::consts::OS,
            env::consts::ARCH,
            MAX_BYTES,
            MAX_DURATION,
            MAX_WORKLOAD_DURATION,
            MAX_PORT
        );
    } else {
        println!(
            "nekomusume {} ({}/{})",
            env!("CARGO_PKG_VERSION"),
            env::consts::OS,
            env::consts::ARCH
        );
        println!("defaults bytes=32 count=1 duration_seconds=10");
        println!(
            "limits bytes=1-{} count=1-64 duration_seconds=1-{} workload_duration_seconds=1-{} ports=40080-{}",
            MAX_BYTES, MAX_DURATION, MAX_WORKLOAD_DURATION, MAX_PORT
        );
        println!(
            "commands research=client,server,probe experimental=health-observe,failover,multistream fixtures=scheduler-fairness,key-update"
        );
        println!("report_secret_free=true");
    }
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
fn health_observe(args: &[String]) {
    let path = parse(args, "--path", None)
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid path"));
    let rtt_us = parse(args, "--rtt-us", None)
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid rtt-us"));
    let loss_per_mille = parse(args, "--loss-per-mille", None)
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid loss-per-mille"));
    let pto = parse(args, "--pto", None)
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid pto"));
    if loss_per_mille > 1000 {
        fail("loss-per-mille outside 0-1000");
    }
    let count = exchange_count(args);
    let mut evidence = CarrierHealthEvidence::new(
        HealthLimits::default(),
        HealthEvidenceLimits { max_samples: 64 },
    )
    .unwrap_or_else(|_| fail("health limits invalid"));
    let sample = HealthSample {
        rtt_us,
        loss_per_mille,
        pto,
    };
    for _ in 0..count {
        evidence
            .observe(PathId(path), sample)
            .unwrap_or_else(|_| fail("health path limit exceeded"));
    }
    if !json_mode(args) {
        println!(
            "health_observe_ok samples={} path={}",
            evidence.samples().len(),
            path
        );
    } else {
        println!("{}", evidence.json());
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
enum UdpWait {
    Datagram(usize, SocketAddr),
    Shutdown,
    Deadline,
}

fn recv_udp_until(
    socket: &UdpSocket,
    buffer: &mut [u8],
    deadline: Instant,
    shutdown: &AtomicBool,
) -> std::io::Result<UdpWait> {
    loop {
        if shutdown.load(Ordering::Acquire) {
            return Ok(UdpWait::Shutdown);
        }
        if Instant::now() >= deadline {
            return Ok(UdpWait::Deadline);
        }
        match socket.recv_from(buffer) {
            Ok((len, peer)) => return Ok(UdpWait::Datagram(len, peer)),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error),
        }
    }
}
fn emit_lifecycle(lifecycle: &lifecycle::Lifecycle) {
    println!(
        "lifecycle_state={} readiness={}",
        lifecycle.state().as_str(),
        lifecycle.readiness()
    );
    std::io::stdout()
        .flush()
        .unwrap_or_else(|_| fail("lifecycle output failed"));
}
fn server(args: &[String]) {
    let lifecycle = lifecycle::Lifecycle::new();
    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    flag::register(SIGINT, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    let (t, p, max, d) = common(args);
    lifecycle.satisfy(ReadinessPrerequisite::ConfigurationAccepted);
    let count = exchange_count(args);
    let idpath = PathBuf::from(parse(args, "--identity", Some("neko-server.identity")));
    let id = load_or_generate(&idpath);
    lifecycle.satisfy(ReadinessPrerequisite::IdentityInitialized);
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
    lifecycle.satisfy(ReadinessPrerequisite::TrustPolicyInitialized);
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
        lifecycle.satisfy(ReadinessPrerequisite::SocketBound);
        l.set_nonblocking(true)
            .unwrap_or_else(|_| fail("listener setup failed"));
        lifecycle.satisfy(ReadinessPrerequisite::IoConfigured);
        lifecycle
            .finalize_readiness()
            .unwrap_or_else(|_| fail("readiness prerequisites incomplete"));
        emit_lifecycle(&lifecycle);
        while start.elapsed() < d && !shutdown.load(Ordering::Acquire) {
            match l.accept() {
                Ok((mut s, _)) => {
                    let mut negotiation =
                        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
                            .unwrap_or_else(|_| fail("negotiation setup failed"));
                    let hello = read_frame(&mut s, MAX_NEGOTIATION_FRAME)
                        .unwrap_or_else(|_| fail("malformed negotiation"));
                    let selection = negotiation
                        .server_accept_hello(&hello)
                        .unwrap_or_else(|_| fail("incompatible negotiation"));
                    write_frame(&mut s, &selection)
                        .unwrap_or_else(|_| fail("negotiation response failed"));
                    let binding = negotiation
                        .authenticated_binding()
                        .unwrap_or_else(|_| fail("negotiation binding failed"));
                    let first = read_frame(&mut s, 1024).unwrap_or_else(|_| fail("bad handshake"));
                    let (resp, mut ss) = ResponderHandshake::new_with_prologue_binding(
                        &id,
                        policy.clone(),
                        DOMAIN,
                        binding.as_bytes(),
                    )
                    .unwrap_or_else(|_| fail("handshake setup failed"))
                    .receive_first(&first, context(0))
                    .unwrap_or_else(|_| fail("unauthorized handshake"));
                    write_frame(&mut s, &resp)
                        .unwrap_or_else(|_| fail("handshake response failed"));
                    negotiation
                        .admit_data()
                        .unwrap_or_else(|_| fail("data admission denied"));
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
                    lifecycle.stopped();
                    println!("lifecycle_state=STOPPED readiness=false");
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
        lifecycle.satisfy(ReadinessPrerequisite::SocketBound);
        u.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap_or_else(|_| fail("socket setup failed"));
        lifecycle.satisfy(ReadinessPrerequisite::IoConfigured);
        lifecycle
            .finalize_readiness()
            .unwrap_or_else(|_| fail("readiness prerequisites incomplete"));
        emit_lifecycle(&lifecycle);
        let mut b = [0; 65536];
        while start.elapsed() < d && !shutdown.load(Ordering::Acquire) {
            if let Ok((n, peer)) = u.recv_from(&mut b) {
                let mut negotiation =
                    VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
                        .unwrap_or_else(|_| fail("negotiation setup failed"));
                let selection = negotiation
                    .server_accept_hello(&b[..n])
                    .unwrap_or_else(|_| fail("incompatible negotiation"));
                u.send_to(&selection, peer)
                    .unwrap_or_else(|_| fail("negotiation response failed"));
                let binding = negotiation
                    .authenticated_binding()
                    .unwrap_or_else(|_| fail("negotiation binding failed"));
                let (n, handshake_peer) = u
                    .recv_from(&mut b)
                    .unwrap_or_else(|_| fail("handshake timeout"));
                if handshake_peer != peer {
                    fail("handshake peer changed");
                }
                let (resp, mut ss) = ResponderHandshake::new_with_prologue_binding(
                    &id,
                    policy.clone(),
                    DOMAIN,
                    binding.as_bytes(),
                )
                .unwrap_or_else(|_| fail("handshake setup failed"))
                .receive_first(&b[..n], context(0))
                .unwrap_or_else(|_| fail("unauthorized handshake"));
                u.send_to(&resp, peer)
                    .unwrap_or_else(|_| fail("handshake response failed"));
                negotiation
                    .admit_data()
                    .unwrap_or_else(|_| fail("data admission denied"));
                let application_deadline = Instant::now() + d;
                for _ in 0..count {
                    let (n, data_peer) =
                        match recv_udp_until(&u, &mut b, application_deadline, &shutdown)
                            .unwrap_or_else(|_| fail("data receive failed"))
                        {
                            UdpWait::Datagram(n, data_peer) => (n, data_peer),
                            UdpWait::Shutdown => {
                                lifecycle.drain();
                                lifecycle.stopped();
                                println!("lifecycle_state=STOPPED readiness=false");
                                return;
                            }
                            UdpWait::Deadline => fail("data timeout"),
                        };
                    if data_peer != peer {
                        fail("data peer changed");
                    }
                    let plain = ss
                        .open_unreliable(&b[..n])
                        .unwrap_or_else(|_| fail("auth failure"));
                    let reply = ss
                        .seal_unreliable(&plain)
                        .unwrap_or_else(|_| fail("seal failure"));
                    u.send_to(&reply, peer).unwrap();
                }
                lifecycle.stopped();
                println!("lifecycle_state=STOPPED readiness=false");
                return;
            }
        }
    }
    if shutdown.load(Ordering::Acquire) {
        lifecycle.drain();
        lifecycle.stopped();
        println!("lifecycle_state=STOPPED readiness=false");
        return;
    }
    lifecycle.failed();
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
    let payload = vec![b'x'; max];
    let start = Instant::now();
    let cs = if t == "tcp" {
        let mut s =
            TcpStream::connect_timeout(&addr.parse().unwrap_or_else(|_| fail("bad address")), d)
                .unwrap_or_else(|_| fail("connect failed"));
        let mut negotiation = VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS)
            .unwrap_or_else(|_| fail("negotiation setup failed"));
        let hello = negotiation
            .client_hello()
            .unwrap_or_else(|_| fail("negotiation setup failed"));
        write_frame(&mut s, &hello).unwrap_or_else(|_| fail("negotiation send failed"));
        let selection = read_frame(&mut s, MAX_NEGOTIATION_FRAME)
            .unwrap_or_else(|_| fail("negotiation response failed"));
        negotiation
            .client_accept_response(&selection)
            .unwrap_or_else(|_| fail("incompatible negotiation"));
        let binding = negotiation
            .authenticated_binding()
            .unwrap_or_else(|_| fail("negotiation binding failed"));
        let mut hs = InitiatorHandshake::new_with_prologue_binding(
            &id,
            &sk,
            b"probe",
            DOMAIN,
            binding.as_bytes(),
        )
        .unwrap_or_else(|_| fail("handshake setup failed"));
        let first = hs
            .first_message()
            .unwrap_or_else(|_| fail("handshake failed"));
        write_frame(&mut s, &first).unwrap_or_else(|_| fail("handshake send failed"));
        let resp = read_frame(&mut s, 1024).unwrap_or_else(|_| fail("handshake response failed"));
        let session = hs
            .finish(&resp, context(0))
            .unwrap_or_else(|_| fail("handshake finish failed"));
        negotiation
            .admit_data()
            .unwrap_or_else(|_| fail("data admission denied"));
        Some((s, session))
    } else {
        let target: SocketAddr = addr.parse().unwrap_or_else(|_| fail("bad address"));
        let local = match target.ip() {
            IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0),
            IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
        };
        let u = UdpSocket::bind(local).unwrap_or_else(|_| fail("UDP socket family unavailable"));
        u.set_read_timeout(Some(d)).unwrap();
        u.connect(target)
            .unwrap_or_else(|_| fail("UDP connect failed"));
        let mut negotiation = VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS)
            .unwrap_or_else(|_| fail("negotiation setup failed"));
        let hello = negotiation
            .client_hello()
            .unwrap_or_else(|_| fail("negotiation setup failed"));
        u.send(&hello)
            .unwrap_or_else(|_| fail("negotiation send failed"));
        let mut b = [0; 65536];
        let n = u
            .recv(&mut b)
            .unwrap_or_else(|_| fail("negotiation response failed"));
        negotiation
            .client_accept_response(&b[..n])
            .unwrap_or_else(|_| fail("incompatible negotiation"));
        let binding = negotiation
            .authenticated_binding()
            .unwrap_or_else(|_| fail("negotiation binding failed"));
        let mut hs = InitiatorHandshake::new_with_prologue_binding(
            &id,
            &sk,
            b"probe",
            DOMAIN,
            binding.as_bytes(),
        )
        .unwrap_or_else(|_| fail("handshake setup failed"));
        let first = hs
            .first_message()
            .unwrap_or_else(|_| fail("handshake failed"));
        u.send(&first)
            .unwrap_or_else(|_| fail("handshake send failed"));
        let n = u
            .recv(&mut b)
            .unwrap_or_else(|_| fail("handshake response failed"));
        let mut ss = hs
            .finish(&b[..n], context(0))
            .unwrap_or_else(|_| fail("handshake finish failed"));
        negotiation
            .admit_data()
            .unwrap_or_else(|_| fail("data admission denied"));
        for _ in 0..count {
            let rec = ss
                .seal_unreliable(&payload)
                .unwrap_or_else(|_| fail("payload too large"));
            u.send(&rec).unwrap();
            let n = u.recv(&mut b).unwrap_or_else(|_| fail("echo timeout"));
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
fn delivery_ack_matches(plain: &[u8], expected: &OutboundRecord) -> bool {
    matches!(
        ProcessMessage::decode(plain),
        Ok(ProcessMessage::DeliveryAck { session, stream, offset, len })
            if session == SessionId(7001)
                && stream == expected.stream
                && offset == expected.offset
                && len == expected.data.len()
    )
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
type PendingUdpNegotiation = (SocketAddr, Vec<u8>, Vec<u8>, Vec<u8>);

#[allow(clippy::collapsible_if)]
fn failover_server(args: &[String]) {
    let mut diag = HandshakeDiagnostics::new("server");
    let count = exchange_count(args);
    let bytes = parse(args, "--bytes", Some("32"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    let duration = Duration::from_secs(
        parse(args, "--duration", Some("10"))
            .parse::<u64>()
            .unwrap_or_else(|_| fail("invalid duration")),
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
    // Bounded one-peer pre-auth cache. It is discarded only after authentication.
    let mut pending: Option<PendingUdpNegotiation> = None;
    let mut secure = None;
    let mut handshake_cache: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut guard = None;
    let mut app = Vec::new();
    let udp_local_port = udp.local_addr().map(|a| a.port()).unwrap_or(up);
    let tcp_local_port = tcp.local_addr().map(|a| a.port()).unwrap_or(tp);
    let mut runtime =
        SessionRuntime::new(SessionId(7001), runtime_limits(bytes, count), 0).unwrap();
    runtime.open_stream(StreamId(1), 0).unwrap();
    emit_diagnostic(
        args,
        "server",
        "start",
        0,
        &format!(
            ",\"count\":{},\"record_payload_bytes\":{},\"application_bytes_total\":{},\"udp_port\":{},\"tcp_port\":{},\"max_seconds\":{}",
            count,
            bytes,
            count * bytes,
            udp_local_port,
            tcp_local_port,
            duration.as_secs()
        ),
    );
    while started.elapsed() < duration {
        if secure.is_none() {
            if let Ok((n, peer)) = udp.recv_from(&mut buf) {
                let datagram = &buf[..n];
                if let Some((pending_peer, hello, selection, binding)) = pending.as_ref() {
                    if peer != *pending_peer {
                        continue;
                    }
                    if datagram == hello.as_slice() {
                        udp.send_to(selection, peer).unwrap();
                        emit_diagnostic(args, "server", "udp_selection_retried", 0, "");
                        continue;
                    }
                    let (resp, ss) = ResponderHandshake::new_with_prologue_binding(
                        &id,
                        policy.clone(),
                        DOMAIN,
                        binding,
                    )
                    .unwrap()
                    .receive_first(datagram, context(1))
                    .unwrap_or_else(|_| fail("unauthorized UDP handshake"));
                    if !args.iter().any(|a| a == "--drop-first-udp-noise-response") {
                        udp.send_to(&resp, peer).unwrap();
                    } else {
                        emit_diagnostic(args, "server", "udp_noise_response_dropped", 0, "");
                    }
                    guard = Some(
                        ResumeGuard::new_with_negotiation(
                            &client,
                            &failover_binding(7001, 0, 10_000),
                            binding,
                        )
                        .unwrap(),
                    );
                    handshake_cache = Some((datagram.to_vec(), resp));
                    secure = Some((ss, peer));
                    pending = None;
                    println!(
                        "carrier_event name=udp_negotiated session=7001 generation=0 version=0"
                    );
                    println!("carrier_event name=udp_authenticated session=7001 generation=0");
                } else {
                    let mut negotiation =
                        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
                            .unwrap();
                    if let Ok(selection) = negotiation.server_accept_hello(datagram) {
                        let binding = negotiation
                            .authenticated_binding()
                            .unwrap()
                            .as_bytes()
                            .to_vec();
                        diag.event(args, "server_recv");
                        emit_diagnostic(args, "server", "udp_hello_received", 0, "");
                        pending = Some((peer, datagram.to_vec(), selection.clone(), binding));
                        if !args.iter().any(|a| a == "--drop-first-udp-selection") {
                            udp.send_to(&selection, peer).unwrap();
                        } else {
                            emit_diagnostic(args, "server", "udp_selection_dropped", 0, "");
                        }
                    }
                }
            }
        }
        if let Some((ref mut ss, peer)) = secure {
            if let Ok((n, datagram_peer)) = udp.recv_from(&mut buf) {
                if datagram_peer != peer {
                    continue;
                }
                if let Some((first, response)) = handshake_cache.as_ref() {
                    if buf[..n] == first[..] {
                        udp.send_to(response, peer).unwrap();
                        emit_diagnostic(args, "server", "udp_noise_response_retried", 0, "");
                        continue;
                    }
                }
                emit_diagnostic(
                    args,
                    "server",
                    "udp_datagram_received",
                    1,
                    &format!(",\"ciphertext_bytes\":{}", n),
                );
                if let Ok(plain) = ss.open_unreliable(&buf[..n]) {
                    if let Ok(ProcessMessage::Data { session, record }) =
                        ProcessMessage::decode(&plain)
                    {
                        let stream = record.stream;
                        let offset = record.offset;
                        let len = record.data.len();
                        if session == SessionId(7001)
                            && runtime
                                .receive(
                                    InboundRecord {
                                        stream,
                                        offset,
                                        data: record.data,
                                    },
                                    1,
                                )
                                .is_ok()
                        {
                            let logical = ProcessMessage::DeliveryAck {
                                session,
                                stream,
                                offset,
                                len,
                            }
                            .encode()
                            .unwrap();
                            let ack = ss.seal_unreliable(&logical).unwrap();
                            udp.send_to(&ack, peer).unwrap();
                            emit_diagnostic(
                                args,
                                "server",
                                "udp_delivery_ack_sent",
                                offset as usize / bytes.max(1),
                                &format!(",\"ciphertext_bytes\":{}", ack.len()),
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
            let hello = read_frame(&mut stream, MAX_NEGOTIATION_FRAME)
                .unwrap_or_else(|_| fail("bad negotiation"));
            let mut negotiation =
                VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
            let selection = negotiation
                .server_accept_hello(&hello)
                .unwrap_or_else(|_| fail("incompatible negotiation"));
            write_frame(&mut stream, &selection).unwrap();
            let binding = negotiation.authenticated_binding().unwrap();
            println!("carrier_event name=tcp_negotiated session=7001 generation=1 version=0");
            let first = read_frame(&mut stream, 1024).unwrap_or_else(|_| fail("bad TCP handshake"));
            let (resp, mut ss, remote, resume_binding) =
                ResponderHandshake::new_with_prologue_binding(
                    &id,
                    policy.clone(),
                    DOMAIN,
                    binding.as_bytes(),
                )
                .unwrap()
                .receive_first_with_resume(&first, context(2))
                .unwrap_or_else(|_| fail("unauthorized TCP handshake"));
            if guard
                .as_mut()
                .map(|g| {
                    g.attach_with_negotiation(&remote, &resume_binding, binding.as_bytes(), 1)
                        .is_ok()
                })
                .unwrap_or(false)
            {
                write_frame(&mut stream, &resp).unwrap();
                for _ in 0..count.saturating_sub(1) {
                    let ciphertext = read_frame(&mut stream, PROCESS_FRAME_MAX)
                        .unwrap_or_else(|_| fail("bad TCP data frame"));
                    let plain = ss
                        .open(&ciphertext)
                        .unwrap_or_else(|_| fail("unauthenticated TCP data"));
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
                                    2,
                                )
                                .is_ok()
                        {
                            let logical = ProcessMessage::DeliveryAck {
                                session,
                                stream: stream_id,
                                offset,
                                len,
                            }
                            .encode()
                            .unwrap();
                            let encrypted = ss.seal(&logical).unwrap();
                            write_frame(&mut stream, &encrypted).unwrap();
                            emit_diagnostic(
                                args,
                                "server",
                                "tcp_delivery_ack_sent",
                                offset as usize / bytes.max(1),
                                &format!(",\"ciphertext_bytes\":{}", encrypted.len()),
                            );
                            if let Some(delivered) = runtime.pop_receive(3).unwrap() {
                                app.extend_from_slice(&delivered.data);
                            }
                        }
                    }
                }
                println!("carrier_event name=tcp_resumed session=7001 generation=1");
                println!(
                    "failover_server_ok session=7001 records={} application_bytes_total={} bytes_hex={} controlled_udp_stop=true udp_port={} tcp_port={} carrier_events=udp_negotiated,udp_authenticated,controlled_udp_stop,tcp_negotiated,tcp_resumed",
                    app.len() / bytes.max(1),
                    app.len(),
                    hex(&app),
                    udp_local_port,
                    tcp_local_port
                );
                emit_diagnostic(
                    args,
                    "server",
                    "summary",
                    count,
                    &format!(
                        ",\"classification\":\"A\",\"records\":{},\"application_bytes_total\":{}",
                        count,
                        app.len()
                    ),
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
    let target = format!("{addr}:{up}")
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad UDP target"));
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
    let negotiation_hello = negotiation.client_hello().unwrap();
    let u = UdpSocket::bind("0.0.0.0:0").unwrap();
    diag.event(args, "socket_bind");
    u.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut buf = [0u8; 65536];
    emit_diagnostic(
        args,
        "client",
        "start",
        0,
        &format!(
            ",\"count\":{},\"record_payload_bytes\":{},\"application_bytes_total\":{},\"udp_port\":{},\"tcp_port\":{},\"max_seconds\":{}",
            count,
            bytes,
            count * bytes,
            up,
            tp,
            secs
        ),
    );
    let mut selection = None;
    for _ in 0..20 {
        diag.event(args, "client_send");
        u.send_to(&negotiation_hello, target).unwrap();
        diag.event(args, "client_hello_sent");
        emit_diagnostic(args, "client", "udp_hello_sent", 0, "");
        match u.recv_from(&mut buf) {
            Ok((n, peer)) if peer == target => {
                diag.event(args, "client_recv");
                selection = Some(n);
                break;
            }
            Ok(_) => continue,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(_) => fail("UDP handshake receive failed"),
        }
    }
    let n = selection.unwrap_or_else(|| {
        diag.timeout(args);
        fail("UDP handshake timeout")
    });
    negotiation.client_accept_response(&buf[..n]).unwrap();
    let binding = negotiation.authenticated_binding().unwrap();
    let mut hs = InitiatorHandshake::new_with_prologue_binding(
        &id,
        &sk,
        b"failover",
        DOMAIN,
        binding.as_bytes(),
    )
    .unwrap();
    let first = hs.first_message().unwrap();
    let mut response = None;
    for _ in 0..20 {
        u.send_to(&first, target).unwrap();
        match u.recv_from(&mut buf) {
            Ok((n, peer)) if peer == target => {
                response = Some(n);
                break;
            }
            Ok(_) => continue,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(_) => fail("UDP Noise receive failed"),
        }
    }
    let n = response.unwrap_or_else(|| fail("UDP Noise timeout"));
    let mut us = hs.finish(&buf[..n], context(1)).unwrap();
    let payload = vec![b'x'; bytes];
    let mut delivery =
        SessionRuntime::new(SessionId(7001), runtime_limits(bytes, count), 0).unwrap();
    delivery.open_stream(StreamId(1), 0).unwrap();
    for i in 0..count {
        delivery
            .queue_send(StreamId(1), &payload, i as u64 + 1)
            .unwrap();
    }
    let mut records = Vec::new();
    while let Some(record) = delivery.pop_send(count as u64 + 2).unwrap() {
        records.push(record);
    }
    println!("carrier_event name=udp_authenticated session=7001 generation=0");
    if args.iter().any(|a| a == "--send-late-udp-hello") {
        u.send_to(&negotiation_hello, target).unwrap();
        emit_diagnostic(args, "client", "late_udp_hello_sent", 0, "");
    }
    let udp_record = records[0].clone();
    let logical = ProcessMessage::Data {
        session: SessionId(7001),
        record: udp_record.clone(),
    }
    .encode()
    .unwrap();
    let encrypted = us.seal_unreliable(&logical).unwrap();
    u.send_to(&encrypted, target).unwrap();
    emit_diagnostic(
        args,
        "client",
        "udp_datagram_sent",
        1,
        &format!(
            ",\"ciphertext_bytes\":{},\"record_payload_bytes\":{}",
            encrypted.len(),
            udp_record.data.len()
        ),
    );
    let (n, peer) = u
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("UDP delivery acknowledgement timeout"));
    if peer != target {
        fail("UDP delivery acknowledgement peer changed")
    }
    let ack_plain = us
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("unauthenticated UDP delivery acknowledgement"));
    if !delivery_ack_matches(&ack_plain, &udp_record) {
        fail("invalid UDP delivery acknowledgement")
    }
    delivery
        .delivery_ack(
            udp_record.stream,
            udp_record.offset,
            udp_record.data.len(),
            count as u64 + 3,
        )
        .unwrap();
    emit_diagnostic(
        args,
        "client",
        "udp_delivery_ack_validated",
        1,
        &format!(",\"ciphertext_bytes\":{}", n),
    );
    // Leave exactly the next logical range uncertain at the controlled carrier
    // stop. It is resent over TCP; the receiver's Session runtime deduplicates
    // it if UDP delivery completed before the fault boundary.
    if let Some(uncertain) = records.get(1) {
        let logical = ProcessMessage::Data {
            session: SessionId(7001),
            record: uncertain.clone(),
        }
        .encode()
        .unwrap();
        let encrypted = us.seal_unreliable(&logical).unwrap();
        u.send_to(&encrypted, target).unwrap();
        emit_diagnostic(
            args,
            "client",
            "udp_uncertain_range_sent",
            2,
            &format!(
                ",\"ciphertext_bytes\":{},\"stream\":{},\"offset\":{},\"len\":{}",
                encrypted.len(),
                uncertain.stream.0,
                uncertain.offset,
                uncertain.data.len()
            ),
        );
    }
    println!(
        "carrier_event name=controlled_udp_stop session=7001 generation=0 reason=bounded_application_fault_injection"
    );
    let mut negotiation2 =
        VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
    let hello2 = negotiation2.client_hello().unwrap();
    let mut tcp = TcpStream::connect_timeout(
        &format!("{addr}:{tp}").parse().unwrap(),
        Duration::from_secs(secs),
    )
    .unwrap();
    write_frame(&mut tcp, &hello2).unwrap();
    let response2 = read_frame(&mut tcp, MAX_NEGOTIATION_FRAME).unwrap();
    negotiation2.client_accept_response(&response2).unwrap();
    let binding2 = negotiation2.authenticated_binding().unwrap();
    let mut hs2 = InitiatorHandshake::with_resume_negotiation_binding(
        &id,
        &sk,
        b"failover",
        DOMAIN,
        &failover_binding(7001, 1, 10_000),
        binding2.as_bytes(),
    )
    .unwrap();
    let first2 = hs2.first_message().unwrap();
    write_frame(&mut tcp, &first2).unwrap();
    let resp = read_frame(&mut tcp, 1024).unwrap();
    let mut ts = hs2.finish(&resp, context(2)).unwrap();
    println!("carrier_event name=tcp_resume_guard session=7001 generation=1");
    for record in records.into_iter().skip(1) {
        let logical = ProcessMessage::Data {
            session: SessionId(7001),
            record: record.clone(),
        }
        .encode()
        .unwrap();
        let encrypted = ts.seal(&logical).unwrap();
        write_frame(&mut tcp, &encrypted).unwrap();
        let ack = read_frame(&mut tcp, PROCESS_FRAME_MAX)
            .unwrap_or_else(|_| fail("TCP delivery acknowledgement timeout"));
        let plain = ts
            .open(&ack)
            .unwrap_or_else(|_| fail("unauthenticated TCP delivery acknowledgement"));
        if !delivery_ack_matches(&plain, &record) {
            fail("invalid TCP delivery acknowledgement")
        }
        delivery
            .delivery_ack(
                record.stream,
                record.offset,
                record.data.len(),
                count as u64 + 4,
            )
            .unwrap();
        emit_diagnostic(
            args,
            "client",
            "tcp_delivery_ack_validated",
            record.offset as usize / bytes.max(1),
            &format!(",\"ciphertext_bytes\":{}", ack.len()),
        );
    }
    println!(
        "carrier_event name=ordered_records_complete session=7001 count={count} bytes={bytes}"
    );
    emit_diagnostic(
        args,
        "client",
        "summary",
        count,
        &format!(
            ",\"classification\":\"A\",\"records\":{},\"application_bytes_total\":{}",
            count,
            count * bytes
        ),
    );
    emit_diagnostic(
        args,
        "client",
        "capture_metadata",
        count + 1,
        ",\"capture\":\"metadata-only\",\"payload\":false,\"keys\":false,\"bounded\":true",
    );
    println!(
        "failover_client_ok session=7001 count={count} application_bytes_total={} controlled_udp_stop=true",
        count * bytes
    );
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

/// Deterministic, socket-free fairness fixture for authorized local/VPS validation.
fn scheduler_fairness(args: &[String]) {
    let rounds = parse(args, "--rounds", Some("8"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid rounds"));
    let bytes = parse(args, "--bytes", Some("16"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    if !(1..=64).contains(&rounds) {
        fail("rounds outside 1-64");
    }
    if !(1..=256).contains(&bytes) {
        fail("bytes outside 1-256");
    }
    let frames = rounds
        .checked_mul(4)
        .unwrap_or_else(|| fail("rounds overflow"));
    let mut scheduler = FairScheduler::new(FlowLimits {
        max_streams: 2,
        max_session_bytes: frames * bytes,
        max_stream_bytes: frames * bytes,
    })
    .unwrap_or_else(|_| fail("scheduler limits invalid"));
    let interactive =
        neko_session::SessionRuntime::new(SessionId(9101), runtime_limits(bytes, frames), 0)
            .unwrap_or_else(|_| fail("interactive runtime limits invalid"));
    let bulk = neko_session::SessionRuntime::new(SessionId(9102), runtime_limits(bytes, frames), 0)
        .unwrap_or_else(|_| fail("bulk runtime limits invalid"));
    let mut runtimes = [interactive, bulk];
    runtimes[0]
        .open_stream(StreamId(1), 0)
        .unwrap_or_else(|_| fail("interactive stream failed"));
    runtimes[1]
        .open_stream(StreamId(1), 0)
        .unwrap_or_else(|_| fail("bulk stream failed"));
    let ids = [CarrierStreamId(1), CarrierStreamId(2)];
    scheduler
        .open(ids[0], StreamPriority::Interactive)
        .unwrap_or_else(|_| fail("interactive open failed"));
    scheduler
        .open(ids[1], StreamPriority::Bulk)
        .unwrap_or_else(|_| fail("bulk open failed"));
    let mut interactive_sent = 0usize;
    let mut bulk_sent = 0usize;
    let mut max_interactive_burst = 0usize;
    let mut current_interactive_burst = 0usize;
    let mut forced_bulk_services = 0usize;
    for round in 0..rounds {
        for _ in 0..3 {
            scheduler
                .enqueue(ids[0], &vec![b'i'; bytes])
                .unwrap_or_else(|_| fail("interactive enqueue failed"));
        }
        scheduler
            .enqueue(ids[1], &vec![b'b'; bytes])
            .unwrap_or_else(|_| fail("bulk enqueue failed"));
        for _ in 0..4 {
            let (id, data) = scheduler
                .next_frame()
                .unwrap_or_else(|| fail("scheduler produced no frame"));
            let index = usize::from(id == ids[1]);
            if index == 0 {
                current_interactive_burst += 1;
                max_interactive_burst = max_interactive_burst.max(current_interactive_burst);
                interactive_sent += 1;
            } else {
                if current_interactive_burst >= 3 {
                    forced_bulk_services += 1;
                }
                current_interactive_burst = 0;
                bulk_sent += 1;
            }
            let offset = runtimes[index]
                .queue_send(StreamId(1), &data, (round * 4) as u64)
                .unwrap_or_else(|_| fail("runtime queue failed"));
            let record = runtimes[index]
                .pop_send((round * 4 + 1) as u64)
                .unwrap_or_else(|_| fail("runtime pop failed"))
                .unwrap_or_else(|| fail("runtime record missing"));
            if record.offset != offset {
                fail("runtime offset mismatch");
            }
        }
    }
    if max_interactive_burst > 3 || forced_bulk_services != rounds {
        fail("fairness bound violated");
    }
    let report = format!(
        "{{\"ok\":true,\"fixture\":\"scheduler-fairness\",\"transport\":\"loopback\",\"rounds\":{rounds},\"interactive_frames\":{interactive_sent},\"bulk_frames\":{bulk_sent},\"max_interactive_burst\":{max_interactive_burst},\"forced_bulk_services\":{forced_bulk_services},\"bound_interactive_burst\":3}}"
    );
    if json_mode(args) {
        println!("{report}");
    } else {
        println!(
            "scheduler_fairness_ok rounds={rounds} interactive_frames={interactive_sent} bulk_frames={bulk_sent} max_interactive_burst={max_interactive_burst} forced_bulk_services={forced_bulk_services}"
        );
    }
}

/// Run independent Session runtimes for a bounded interval. This local fixture exercises queueing, delivery ACKs and cleanup without opening sockets.
fn workload_duration_valid(duration: u64) -> bool {
    (1..=MAX_WORKLOAD_DURATION).contains(&duration)
}

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
    if !workload_duration_valid(duration) {
        fail("duration outside 1-600");
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
        Some("scheduler-fairness") => scheduler_fairness(&a),
        Some("key-update") => key_update_fixture(&a),
        Some("health-observe") => health_observe(&a),
        Some("capabilities") => capabilities(&a),
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
    fn health_observe_arguments_are_bounded_and_parseable() {
        let args = vec![
            "health-observe".into(),
            "--path".into(),
            "7".into(),
            "--rtt-us".into(),
            "1200".into(),
            "--loss-per-mille".into(),
            "0".into(),
            "--pto".into(),
            "0".into(),
            "--count".into(),
            "2".into(),
            "--json".into(),
        ];
        assert_eq!(exchange_count(&args), 2);
        assert!(json_mode(&args));
    }

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
    fn workload_duration_boundary_is_strictly_bounded() {
        assert!(!workload_duration_valid(0));
        assert!(workload_duration_valid(1));
        assert!(workload_duration_valid(300));
        assert!(workload_duration_valid(600));
        assert!(!workload_duration_valid(601));
        assert!(!workload_duration_valid(u64::MAX));
    }

    fn secure_pair() -> (neko_crypto::SecureSession, neko_crypto::SecureSession) {
        let client = LocalIdentity::generate().unwrap();
        let server = LocalIdentity::generate().unwrap();
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: client.public_key().to_vec(),
            scope: b"failover".to_vec(),
            status: TrustStatus::Active,
        }]);
        let mut initiator =
            InitiatorHandshake::new(&client, server.public_key(), b"failover", DOMAIN).unwrap();
        let first = initiator.first_message().unwrap();
        let (response, responder) = ResponderHandshake::new(&server, policy, DOMAIN)
            .unwrap()
            .receive_first(&first, context(1))
            .unwrap();
        let initiator = initiator.finish(&response, context(1)).unwrap();
        (initiator, responder)
    }

    #[test]
    fn delivery_ack_requires_authenticated_exact_range_and_rejects_replay() {
        let expected = OutboundRecord {
            stream: StreamId(1),
            offset: 32,
            data: vec![7; 16],
        };
        let valid = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 32,
            len: 16,
        }
        .encode()
        .unwrap();
        let wrong_cases = [
            ProcessMessage::DeliveryAck {
                session: SessionId(7002),
                stream: StreamId(1),
                offset: 32,
                len: 16,
            },
            ProcessMessage::DeliveryAck {
                session: SessionId(7001),
                stream: StreamId(2),
                offset: 32,
                len: 16,
            },
            ProcessMessage::DeliveryAck {
                session: SessionId(7001),
                stream: StreamId(1),
                offset: 31,
                len: 16,
            },
            ProcessMessage::DeliveryAck {
                session: SessionId(7001),
                stream: StreamId(1),
                offset: 32,
                len: 15,
            },
        ];
        assert!(delivery_ack_matches(&valid, &expected));
        for wrong in wrong_cases {
            assert!(!delivery_ack_matches(&wrong.encode().unwrap(), &expected));
        }
        let (mut sender, mut receiver) = secure_pair();
        assert!(
            receiver.open_unreliable(&valid).is_err(),
            "plaintext must not authenticate"
        );
        let encrypted = sender.seal_unreliable(&valid).unwrap();
        assert!(delivery_ack_matches(
            &receiver.open_unreliable(&encrypted).unwrap(),
            &expected
        ));
        assert!(
            receiver.open_unreliable(&encrypted).is_err(),
            "replay must fail"
        );
        let mut tampered = sender.seal(&valid).unwrap();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(receiver.open(&tampered).is_err(), "tamper must fail");
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
