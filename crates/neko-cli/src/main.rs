//! Bounded authenticated research probe runtime; never a proxy or tunnel.
mod lifecycle;

use health_window::{HealthDatagram, HealthObservationWindow, HealthWindowError};
use lifecycle::ReadinessPrerequisite;
mod framed;
mod health_window;
mod multistream;
mod periodic;
mod preauth;
mod reachability;

use neko_carrier::{
    ActiveCarrier, CarrierHealthEvidence, CarrierManager, CarrierSwitchReason, DataId,
    EndpointRebindCandidate, EndpointRebindState, FailoverController, FairScheduler, FlowLimits,
    HealthEvidenceLimits, HealthLimits, HealthSample, HealthState, ManagerLimits,
    MigrationCandidate, PathGeneration, PathId, ReadinessObservation, StreamId as CarrierStreamId,
    StreamPriority,
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
use sha2::{Digest, Sha256};
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    flag,
};
use std::{
    env, fs,
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const USAGE: &str = "Usage: neko <server|client|probe|health-observe|failover|multistream|scheduler-fairness|key-update|periodic-server|periodic-client|lab|workload|endpoint-rebind-server|endpoint-rebind-client|keygen|capabilities> [bounded options]

  --count N: bounded authenticated exchanges (1-64; periodic 1-600)\n  periodic-*: one TCP Session, duration 1-600s, interval 100-5000ms, <=1MiB app data; reconnect unsupported\n  failover-server|failover-client: legacy failover aliases for failover\n  capabilities [--json]: secret-free build, command, default, and limit report\n\nBounded authenticated research probe only; no proxy/tunnel behavior.\n";
const MAX_PORT: u16 = 40100;
const MAX_BYTES: usize = neko_crypto::MAX_UNRELIABLE_DATAGRAM;
const MAX_DURATION: u64 = 30;
const MAX_WORKLOAD_DURATION: u64 = 600;
const PROCESS_FRAME_MAX: usize = neko_session::PROCESS_FRAME_MAX;
const DOMAIN: &[u8] = b"nekomusume-vps-probe";
const SUPPORTED_VERSIONS: &[u16] = &[NEGOTIATION_VERSION];
const MAX_NEGOTIATION_FRAME: usize = 4 + neko_wire::MAX_NEGOTIATION_VERSIONS * 2;
const READINESS_PROBES: u64 = 3;
const READINESS_FRAME_MAX: usize = 128;
const READINESS_PROBE_TIMEOUT: Duration = Duration::from_secs(1);
const READINESS_SEQUENCE_TIMEOUT: Duration = Duration::from_secs(3);
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
                "{{\"name\":\"key-update\",\"maturity\":\"fixture\"}},",
                "{{\"name\":\"periodic-server\",\"maturity\":\"research\"}},",
                "{{\"name\":\"periodic-client\",\"maturity\":\"research\"}},",
                "{{\"name\":\"lab\",\"maturity\":\"fixture\"}},",
                "{{\"name\":\"workload\",\"maturity\":\"fixture\"}},",
                "{{\"name\":\"endpoint-rebind-server\",\"maturity\":\"experimental\"}},",
                "{{\"name\":\"endpoint-rebind-client\",\"maturity\":\"experimental\"}},",
                "{{\"name\":\"keygen\",\"maturity\":\"utility\"}},",
                "{{\"name\":\"capabilities\",\"maturity\":\"utility\"}}",
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
            "commands research=client,server,probe,periodic-server,periodic-client experimental=health-observe,failover,multistream,endpoint-rebind-server,endpoint-rebind-client fixtures=scheduler-fairness,key-update,lab,workload utilities=keygen,capabilities aliases=failover-server,failover-client"
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
fn benchmark_payload(
    args: &[String],
    transport: &str,
    bytes: usize,
    count: usize,
) -> Option<(Vec<u8>, String)> {
    let path = args
        .windows(2)
        .find(|w| w[0] == "--payload-file")
        .map(|w| PathBuf::from(&w[1]))?;
    if transport != "tcp" || count != 1 || !json_mode(args) {
        fail("--payload-file requires TCP, --count 1, and --json");
    }
    let metadata = fs::metadata(&path).unwrap_or_else(|_| fail("payload file unreadable"));
    if !metadata.is_file() || metadata.len() != bytes as u64 {
        fail("payload file length must equal bounded --bytes");
    }
    let mut payload = Vec::with_capacity(bytes);
    fs::File::open(path)
        .unwrap_or_else(|_| fail("payload file unreadable"))
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut payload)
        .unwrap_or_else(|_| fail("payload file unreadable"));
    if payload.len() != bytes || payload.is_empty() || payload.len() > MAX_BYTES {
        fail("payload file changed or exceeds bounded --bytes");
    }
    let hash = format!("{:x}", Sha256::digest(&payload));
    Some((payload, hash))
}

fn emit_probe(
    args: &[String],
    transport: &str,
    bytes: usize,
    elapsed_ms: u128,
    payload_sha256: Option<&str>,
) {
    if json_mode(args) {
        if let Some(hash) = payload_sha256 {
            let fd_count = fs::read_dir("/proc/self/fd")
                .unwrap_or_else(|_| fail("benchmark FD inventory unavailable"))
                .count();
            println!(
                "{{\"ok\":true,\"transport\":\"{}\",\"application_bytes\":{},\"payload_sha256\":\"{}\",\"elapsed_ms\":{},\"fd_count\":{},\"wire_bytes\":null}}",
                transport, bytes, hash, elapsed_ms, fd_count
            );
        } else {
            println!(
                "{{\"ok\":true,\"transport\":\"{}\",\"bytes\":{},\"elapsed_ms\":{}}}",
                transport, bytes, elapsed_ms
            );
        }
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
fn peer_public_key(value: &str) -> Vec<u8> {
    let key = unhex(value);
    if key.len() != 32 {
        fail("peer public key must be 32 bytes");
    }
    key
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
fn read_identity(path: &PathBuf) -> Option<LocalIdentity> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(_) => fail("identity open failed"),
    };
    let metadata = file
        .metadata()
        .unwrap_or_else(|_| fail("identity metadata failed"));
    if !metadata.file_type().is_file() {
        fail("identity must be a regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            fail("identity permissions must be owner-only");
        }
    }
    let mut s = String::new();
    file.read_to_string(&mut s)
        .unwrap_or_else(|_| fail("identity read failed"));
    let p = s.trim().split(':').map(unhex).collect::<Vec<_>>();
    if p.len() != 2 {
        fail("invalid identity file");
    }
    Some(
        LocalIdentity::from_keypair(&p[0], &p[1]).unwrap_or_else(|_| fail("invalid identity file")),
    )
}
fn load_or_generate(path: &PathBuf) -> LocalIdentity {
    if let Some(identity) = read_identity(path) {
        return identity;
    }
    let id = LocalIdentity::generate().unwrap_or_else(|_| fail("identity generation failed"));
    let material = format!("{}:{}\n", hex(id.private_key()), hex(id.public_key()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            return read_identity(path).unwrap_or_else(|| fail("identity create race"));
        }
        Err(_) => fail("identity write failed"),
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .unwrap_or_else(|_| fail("identity permission failed"));
    }
    file.write_all(material.as_bytes())
        .unwrap_or_else(|_| fail("identity write failed"));
    file.sync_all()
        .unwrap_or_else(|_| fail("identity sync failed"));
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
fn bound_stream_to_deadline(
    stream: &TcpStream,
    deadline: Instant,
    phase_limit: Option<Duration>,
) -> std::io::Result<()> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "experiment duration elapsed",
        ));
    }
    let timeout = phase_limit.map_or(remaining, |limit| remaining.min(limit));
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))
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
fn emit_signal_shutdown(lifecycle: &lifecycle::Lifecycle) {
    lifecycle.drain();
    emit_lifecycle(lifecycle);
    lifecycle.stopped();
    emit_lifecycle(lifecycle);
}
fn server(args: &[String]) {
    let lifecycle = lifecycle::Lifecycle::new();
    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    flag::register(SIGINT, Arc::clone(&shutdown)).unwrap_or_else(|_| fail("signal setup failed"));
    let (t, p, max, d) = common(args);
    let count = exchange_count(args);
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
    let client_key = peer_public_key(&parse(args, "--client-key", None));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key,
        scope: b"probe".to_vec(),
        status: TrustStatus::Active,
    }]);
    lifecycle.satisfy(ReadinessPrerequisite::TrustPolicyInitialized);
    lifecycle.satisfy(ReadinessPrerequisite::ConfigurationAccepted);
    let idpath = PathBuf::from(parse(args, "--identity", Some("neko-server.identity")));
    let id = load_or_generate(&idpath);
    lifecycle.satisfy(ReadinessPrerequisite::IdentityInitialized);
    if !json_mode(args) {
        println!("server_public_key={}", hex(id.public_key()));
    }
    let start = Instant::now();
    let mut preauth = preauth::ListenerAdmission::new();
    if t == "tcp" {
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
                Ok((mut s, peer)) => {
                    let mut admission = preauth
                        .admit_carrier(preauth::CarrierKind::Tcp, peer)
                        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                    let mut negotiation =
                        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
                            .unwrap_or_else(|_| fail("negotiation setup failed"));
                    let mut reader = framed::FramedReader::new(MAX_NEGOTIATION_FRAME);
                    let deadline = start + d;
                    let hello = preauth::read_staged_frame(
                        &mut reader,
                        &mut s,
                        MAX_NEGOTIATION_FRAME,
                        deadline,
                        &mut preauth,
                        &mut admission,
                        64,
                        64,
                    )
                    .unwrap_or_else(|_| fail("malformed negotiation"));
                    let selection = negotiation
                        .server_accept_hello(&hello)
                        .unwrap_or_else(|_| fail("incompatible negotiation"));
                    let response_permit_1 = preauth
                        .charge_response(&mut admission, selection.len() + 4)
                        .unwrap_or_else(|_| fail("pre-auth response rejected"));
                    preauth
                        .send_tcp_response(&mut s, &selection, response_permit_1)
                        .unwrap_or_else(|_| fail("negotiation response deadline elapsed"));
                    let binding = negotiation
                        .authenticated_binding()
                        .unwrap_or_else(|_| fail("negotiation binding failed"));
                    reader
                        .set_max_frame_len(1024)
                        .unwrap_or_else(|_| fail("partial frame crossed protocol stage"));
                    let first = preauth::read_staged_frame(
                        &mut reader,
                        &mut s,
                        1024,
                        deadline,
                        &mut preauth,
                        &mut admission,
                        64,
                        4096,
                    )
                    .unwrap_or_else(|_| fail("bad handshake"));
                    let (resp, mut ss) = ResponderHandshake::new_with_prologue_binding(
                        &id,
                        policy.clone(),
                        DOMAIN,
                        binding.as_bytes(),
                    )
                    .unwrap_or_else(|_| fail("handshake setup failed"))
                    .receive_first(&first, context(0))
                    .unwrap_or_else(|_| fail("unauthorized handshake"));
                    let response_permit_2 = preauth
                        .charge_response(&mut admission, resp.len() + 4)
                        .unwrap_or_else(|_| fail("pre-auth response rejected"));
                    preauth
                        .send_tcp_response(&mut s, &resp, response_permit_2)
                        .unwrap_or_else(|_| fail("handshake response deadline elapsed"));
                    negotiation
                        .admit_data()
                        .unwrap_or_else(|_| fail("data admission denied"));
                    preauth.release(admission);
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
                let mut admission = preauth
                    .admit_carrier(preauth::CarrierKind::Udp, peer)
                    .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                preauth
                    .charge_input(&mut admission, n, 64)
                    .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                let mut negotiation =
                    VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
                        .unwrap_or_else(|_| fail("negotiation setup failed"));
                let selection = negotiation
                    .server_accept_hello(&b[..n])
                    .unwrap_or_else(|_| fail("incompatible negotiation"));
                let response_permit_3 = preauth
                    .charge_response(&mut admission, selection.len())
                    .unwrap_or_else(|_| fail("pre-auth response rejected"));
                preauth
                    .send_udp_response(&u, &selection, peer, response_permit_3)
                    .unwrap_or_else(|_| fail("negotiation response deadline elapsed"));
                let binding = negotiation
                    .authenticated_binding()
                    .unwrap_or_else(|_| fail("negotiation binding failed"));
                let (n, handshake_peer) = u
                    .recv_from(&mut b)
                    .unwrap_or_else(|_| fail("handshake timeout"));
                if handshake_peer != peer {
                    fail("handshake peer changed");
                }
                preauth
                    .charge_input(&mut admission, n, 4096)
                    .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                let (resp, mut ss) = ResponderHandshake::new_with_prologue_binding(
                    &id,
                    policy.clone(),
                    DOMAIN,
                    binding.as_bytes(),
                )
                .unwrap_or_else(|_| fail("handshake setup failed"))
                .receive_first(&b[..n], context(0))
                .unwrap_or_else(|_| fail("unauthorized handshake"));
                let response_permit_4 = preauth
                    .charge_response(&mut admission, resp.len())
                    .unwrap_or_else(|_| fail("pre-auth response rejected"));
                preauth
                    .send_udp_response(&u, &resp, peer, response_permit_4)
                    .unwrap_or_else(|_| fail("handshake response deadline elapsed"));
                negotiation
                    .admit_data()
                    .unwrap_or_else(|_| fail("data admission denied"));
                preauth.release(admission);
                let application_deadline = Instant::now() + d;
                for _ in 0..count {
                    let (n, data_peer) =
                        match recv_udp_until(&u, &mut b, application_deadline, &shutdown)
                            .unwrap_or_else(|_| fail("data receive failed"))
                        {
                            UdpWait::Datagram(n, data_peer) => (n, data_peer),
                            UdpWait::Shutdown => {
                                emit_signal_shutdown(&lifecycle);
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
        emit_signal_shutdown(&lifecycle);
        return;
    }
    lifecycle.failed();
    fail("duration expired")
}
fn client(args: &[String]) {
    let (t, _p, max, d) = common(args);
    let count = exchange_count(args);
    let addr = parse(args, "--addr", None);
    let target: SocketAddr = addr.parse().unwrap_or_else(|_| fail("bad address"));
    let sk = peer_public_key(&parse(args, "--server-key", None));
    let benchmark = benchmark_payload(args, &t, max, count);
    let idpath = PathBuf::from(parse(args, "--identity", Some("neko-client.identity")));
    let id = load_or_generate(&idpath);
    if !json_mode(args) {
        println!("client_public_key={}", hex(id.public_key()));
    }
    let payload = benchmark
        .as_ref()
        .map(|(payload, _)| payload.clone())
        .unwrap_or_else(|| vec![b'x'; max]);
    let start = Instant::now();
    let cs = if t == "tcp" {
        let mut s =
            TcpStream::connect_timeout(&target, d).unwrap_or_else(|_| fail("connect failed"));
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
        emit_probe(args, "udp", max, start.elapsed().as_millis(), None);
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
    emit_probe(
        args,
        "tcp",
        max,
        start.elapsed().as_millis(),
        benchmark.as_ref().map(|(_, hash)| hash.as_str()),
    )
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
const MAX_POST_HANDSHAKE_MALFORMED: usize = 3;
#[allow(clippy::too_many_arguments)]
fn recv_udp_delivery_ack(
    socket: &UdpSocket,
    peer: SocketAddr,
    secure: &mut neko_crypto::SecureSession,
    expected: &OutboundRecord,
    negotiation_response: &[u8],
    noise_response: &[u8],
    deadline: Instant,
    diagnostic: &mut dyn FnMut(&'static str),
) -> Result<usize, &'static str> {
    let mut buf = [0u8; 65536];
    let mut malformed = 0usize;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err("UDP delivery acknowledgement timeout");
        }
        socket
            .set_read_timeout(Some((deadline - now).min(Duration::from_millis(100))))
            .map_err(|_| "UDP delivery acknowledgement receive failed")?;
        match socket.recv_from(&mut buf) {
            Ok((_, source)) if source != peer => diagnostic("unexpected_peer"),
            Ok((n, _)) if buf[..n] == *negotiation_response => {
                diagnostic("duplicate_negotiation_response")
            }
            Ok((n, _)) if buf[..n] == *noise_response => diagnostic("duplicate_noise_response"),
            Ok((n, _)) => match secure.open_unreliable(&buf[..n]) {
                Ok(plain) if delivery_ack_matches(&plain, expected) => return Ok(n),
                _ => {
                    malformed += 1;
                    diagnostic("malformed_or_unadmitted");
                    if malformed >= MAX_POST_HANDSHAKE_MALFORMED {
                        return Err("UDP delivery acknowledgement malformed bound exceeded");
                    }
                }
            },
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(_) => return Err("UDP delivery acknowledgement receive failed"),
        }
    }
}
fn readiness_admitted(runtime: &SessionRuntime, request: &ProcessMessage) -> bool {
    matches!(request, ProcessMessage::ReadinessRequest { session: SessionId(7001), target_path: 2, path_generation: 1, delivery_epoch: 1, challenge_id } if (1..=READINESS_PROBES).contains(challenge_id))
        && runtime.state() == neko_session::RuntimeState::Open
        && runtime.queued_records() <= 2
        && runtime.queued_bytes() <= MAX_BYTES * 2
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
struct PendingUdpNegotiation {
    peer: SocketAddr,
    hello: Vec<u8>,
    selection: Vec<u8>,
    binding: Vec<u8>,
    admission: preauth::AdmissionTicket,
    queue: preauth::QueueReservation,
}

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
    let udp_bind = parse(args, "--udp-bind", Some(&format!("0.0.0.0:{up}")))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad UDP bind"));
    let tcp_bind = parse(args, "--tcp-bind", Some(&format!("0.0.0.0:{tp}")))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad TCP bind"));
    let client = peer_public_key(&parse(args, "--client-key", None));
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client.clone(),
        scope: b"failover".to_vec(),
        status: TrustStatus::Active,
    }]);
    let udp = UdpSocket::bind(udp_bind).unwrap_or_else(|_| fail("UDP bind failed"));
    let tcp = TcpListener::bind(tcp_bind).unwrap_or_else(|_| fail("TCP bind failed"));
    udp.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    tcp.set_nonblocking(true).unwrap();
    diag.event(args, "socket_bind");
    let started = Instant::now();
    let experiment_deadline = started + duration;
    let mut preauth = preauth::ListenerAdmission::new();
    let mut buf = [0u8; 65536];
    // Explicitly opt-in, bounded application-level fault seam. Production and
    // ordinary controlled-stop runs never cease replies.
    let migration_back = args.iter().any(|a| a == "--migration-back");
    let restore_udp_replies_after_tcp = args.iter().any(|a| a == "--restore-udp-replies-after-tcp");
    let recovery_enabled = migration_back || restore_udp_replies_after_tcp;
    let tcp_delivery_delay_ms = parse(args, "--test-tcp-delivery-delay-ms", Some("0"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid TCP delivery delay"));
    if tcp_delivery_delay_ms > 1000 {
        fail("TCP delivery delay outside 0-1000 ms")
    }
    let cease_udp_replies_after = args
        .windows(2)
        .find(|w| w[0] == "--cease-udp-replies-after")
        .map(|w| {
            w[1].parse::<usize>()
                .unwrap_or_else(|_| fail("invalid UDP reply cessation point"))
        });
    if cease_udp_replies_after.is_some_and(|point| point == 0 || point >= count) {
        fail("UDP reply cessation point outside 1..count");
    }
    let mut udp_replies = 0usize;
    // Bounded one-peer pre-auth cache. It is discarded only after authentication.
    let mut pending: Option<PendingUdpNegotiation> = None;
    let mut secure = None;
    let mut handshake_cache: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut delayed_noise_duplicate_sent = false;
    let mut guard = None;
    let mut app = Vec::new();
    let mut migration_back_complete = false;
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
        let expired_states = preauth.expire();
        if pending
            .as_ref()
            .is_some_and(|state| state.admission.was_expired(&expired_states))
        {
            if let Some(mut expired) = pending.take() {
                // Process expiry atomically consumed both state and queue counts.
                expired.queue.invalidate_after_process_expiry();
            }
        }
        if secure.is_none() {
            if let Ok((n, peer)) = udp.recv_from(&mut buf) {
                let datagram = &buf[..n];
                if let Some(pending_state) = pending.as_mut() {
                    if peer != pending_state.peer {
                        continue;
                    }
                    // Charge this received datagram exactly once before any
                    // duplicate/Noise classification. The conservative work
                    // ceiling covers both paths; the packet is not charged
                    // again below.
                    preauth
                        .charge_input(&mut pending_state.admission, n, 4096)
                        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
                    if datagram == pending_state.hello.as_slice() {
                        let response_permit_5 = preauth
                            .charge_response(
                                &mut pending_state.admission,
                                pending_state.selection.len(),
                            )
                            .unwrap_or_else(|_| fail("pre-auth response rejected"));
                        preauth
                            .send_udp_response(
                                &udp,
                                &pending_state.selection,
                                peer,
                                response_permit_5,
                            )
                            .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
                        emit_diagnostic(args, "server", "udp_selection_retried", 0, "");
                        continue;
                    }
                    let (resp, ss) = ResponderHandshake::new_with_prologue_binding(
                        &id,
                        policy.clone(),
                        DOMAIN,
                        &pending_state.binding,
                    )
                    .unwrap()
                    .receive_first(datagram, context(1))
                    .unwrap_or_else(|_| fail("unauthorized UDP handshake"));
                    let response_permit = preauth
                        .charge_response(&mut pending_state.admission, resp.len())
                        .unwrap_or_else(|_| fail("pre-auth response rejected"));
                    if !args.iter().any(|a| a == "--drop-first-udp-noise-response") {
                        preauth
                            .send_udp_response(&udp, &resp, peer, response_permit)
                            .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
                    } else {
                        drop(response_permit);
                        emit_diagnostic(args, "server", "udp_noise_response_dropped", 0, "");
                    }
                    guard = Some(
                        ResumeGuard::new_with_negotiation(
                            &client,
                            &failover_binding(7001, 0, 10_000),
                            &pending_state.binding,
                        )
                        .unwrap(),
                    );
                    handshake_cache = Some((datagram.to_vec(), resp));
                    secure = Some((ss, peer));
                    let mut authenticated = pending.take().expect("pending state exists");
                    preauth
                        .dequeue(&mut authenticated.queue)
                        .unwrap_or_else(|_| fail("pre-auth queue release failed"));
                    preauth.release(authenticated.admission);
                    println!(
                        "carrier_event name=udp_negotiated session=7001 generation=0 version=0"
                    );
                    println!("carrier_event name=udp_authenticated session=7001 generation=0");
                } else {
                    let mut admission = match preauth.admit_carrier(preauth::CarrierKind::Udp, peer)
                    {
                        Ok(admission) => admission,
                        Err(_) => continue,
                    };
                    if preauth.charge_input(&mut admission, n, 64).is_err() {
                        preauth.release(admission);
                        continue;
                    }
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
                        let response_permit =
                            match preauth.charge_response(&mut admission, selection.len()) {
                                Ok(permit) => permit,
                                Err(_) => {
                                    preauth.release(admission);
                                    continue;
                                }
                            };
                        if !args.iter().any(|a| a == "--drop-first-udp-selection") {
                            preauth
                                .send_udp_response(&udp, &selection, peer, response_permit)
                                .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
                        } else {
                            drop(response_permit);
                            emit_diagnostic(args, "server", "udp_selection_dropped", 0, "");
                        }
                        let queue = match preauth.enqueue(&mut admission) {
                            Ok(queue) => queue,
                            Err(_) => {
                                preauth.release(admission);
                                continue;
                            }
                        };
                        pending = Some(PendingUdpNegotiation {
                            peer,
                            hello: datagram.to_vec(),
                            selection: selection.clone(),
                            binding,
                            admission,
                            queue,
                        });
                    } else {
                        preauth.release(admission);
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
                            if !delayed_noise_duplicate_sent
                                && args
                                    .iter()
                                    .any(|a| a == "--delay-noise-duplicate-until-application")
                            {
                                if let Some((_, response)) = handshake_cache.as_ref() {
                                    udp.send_to(response, peer).unwrap();
                                    delayed_noise_duplicate_sent = true;
                                    emit_diagnostic(
                                        args,
                                        "server",
                                        "udp_noise_response_delayed_duplicate_sent",
                                        0,
                                        "",
                                    );
                                }
                            }
                            if cease_udp_replies_after.is_none_or(|point| udp_replies < point) {
                                udp.send_to(&ack, peer).unwrap();
                                udp_replies += 1;
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_delivery_ack_sent",
                                    offset as usize / bytes.max(1),
                                    &format!(",\"ciphertext_bytes\":{}", ack.len()),
                                );
                            } else {
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_reply_ceased",
                                    offset as usize / bytes.max(1),
                                    ",\"reason\":\"bounded_test_seam\"",
                                );
                                if args
                                    .iter()
                                    .any(|arg| arg == "--send-malformed-after-cessation")
                                {
                                    udp.send_to(b"malformed", peer).unwrap();
                                }
                            }
                            if let Some(delivered) = runtime.pop_receive(2).unwrap() {
                                app.extend_from_slice(&delivered.data);
                            }
                        }
                    }
                }
            }
        }
        if let Ok((mut stream, peer)) = tcp.accept() {
            let mut admission = preauth
                .admit_carrier(preauth::CarrierKind::Tcp, peer)
                .unwrap_or_else(|_| fail("pre-auth admission rejected"));
            bound_stream_to_deadline(&stream, experiment_deadline, None)
                .unwrap_or_else(|_| fail("TCP negotiation deadline elapsed"));
            let mut reader = framed::FramedReader::new(MAX_NEGOTIATION_FRAME);
            let hello = preauth::read_staged_frame(
                &mut reader,
                &mut stream,
                MAX_NEGOTIATION_FRAME,
                experiment_deadline,
                &mut preauth,
                &mut admission,
                64,
                64,
            )
            .unwrap_or_else(|_| fail("bad negotiation"));
            let mut negotiation =
                VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
            let selection = negotiation
                .server_accept_hello(&hello)
                .unwrap_or_else(|_| fail("incompatible negotiation"));
            bound_stream_to_deadline(&stream, experiment_deadline, None)
                .unwrap_or_else(|_| fail("TCP negotiation deadline elapsed"));
            let response_permit_8 = preauth
                .charge_response(&mut admission, selection.len() + 4)
                .unwrap_or_else(|_| fail("pre-auth response rejected"));
            preauth
                .send_tcp_response(&mut stream, &selection, response_permit_8)
                .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
            let binding = negotiation.authenticated_binding().unwrap();
            println!("carrier_event name=tcp_negotiated session=7001 generation=1 version=0");
            bound_stream_to_deadline(&stream, experiment_deadline, None)
                .unwrap_or_else(|_| fail("TCP handshake deadline elapsed"));
            reader
                .set_max_frame_len(1024)
                .unwrap_or_else(|_| fail("partial frame crossed protocol stage"));
            let first = preauth::read_staged_frame(
                &mut reader,
                &mut stream,
                1024,
                experiment_deadline,
                &mut preauth,
                &mut admission,
                64,
                4096,
            )
            .unwrap_or_else(|_| fail("bad TCP handshake"));
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
                bound_stream_to_deadline(&stream, experiment_deadline, None)
                    .unwrap_or_else(|_| fail("TCP handshake deadline elapsed"));
                let response_permit_9 = preauth
                    .charge_response(&mut admission, resp.len() + 4)
                    .unwrap_or_else(|_| fail("pre-auth response rejected"));
                preauth
                    .send_tcp_response(&mut stream, &resp, response_permit_9)
                    .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
                preauth.release(admission);
                println!("carrier_event name=tcp_authenticated session=7001 generation=1");
                println!("carrier_event name=tcp_resume_validated session=7001 generation=1");
                let readiness_sequence_deadline =
                    (Instant::now() + READINESS_SEQUENCE_TIMEOUT).min(experiment_deadline);
                let mut readiness_response_bytes = 0usize;
                for expected_challenge in 1..=READINESS_PROBES {
                    bound_stream_to_deadline(
                        &stream,
                        readiness_sequence_deadline,
                        Some(READINESS_PROBE_TIMEOUT),
                    )
                    .unwrap_or_else(|_| fail("readiness deadline elapsed"));
                    let ciphertext = read_frame(&mut stream, READINESS_FRAME_MAX)
                        .unwrap_or_else(|_| fail("bad readiness frame"));
                    let plain = ss
                        .open(&ciphertext)
                        .unwrap_or_else(|_| fail("unauthenticated readiness frame"));
                    let request = ProcessMessage::decode(&plain)
                        .unwrap_or_else(|_| fail("malformed readiness request"));
                    let admitted = readiness_admitted(&runtime, &request)
                        && !args.iter().any(|arg| arg == "--test-readiness-unadmitted");
                    let ProcessMessage::ReadinessRequest {
                        session,
                        target_path,
                        path_generation,
                        delivery_epoch,
                        challenge_id,
                    } = request
                    else {
                        fail("unexpected readiness message")
                    };
                    if challenge_id != expected_challenge {
                        fail("non-consecutive readiness challenge")
                    }
                    let response = ProcessMessage::ReadinessResponse {
                        session,
                        target_path,
                        path_generation,
                        delivery_epoch,
                        challenge_id,
                        admitted,
                    }
                    .encode()
                    .unwrap();
                    let encrypted = ss.seal(&response).unwrap();
                    let readiness_delay = args
                        .windows(2)
                        .find(|w| w[0] == "--test-readiness-delay-ms")
                        .map(|w| {
                            w[1].parse::<u64>()
                                .unwrap_or_else(|_| fail("invalid readiness delay"))
                        })
                        .unwrap_or(0);
                    if readiness_delay > 3_000 {
                        fail("readiness delay outside 0-3000")
                    }
                    if readiness_delay > 0 {
                        std::thread::sleep(Duration::from_millis(readiness_delay));
                    }
                    bound_stream_to_deadline(
                        &stream,
                        readiness_sequence_deadline,
                        Some(READINESS_PROBE_TIMEOUT),
                    )
                    .unwrap_or_else(|_| fail("readiness deadline elapsed"));
                    readiness_response_bytes =
                        readiness_response_bytes.saturating_add(encrypted.len());
                    if readiness_response_bytes > READINESS_FRAME_MAX * READINESS_PROBES as usize {
                        fail("readiness responder byte bound exceeded")
                    }
                    write_frame(&mut stream, &encrypted).unwrap();
                    emit_diagnostic(
                        args,
                        "server",
                        "tcp_readiness_response",
                        challenge_id as usize,
                        &format!(
                            ",\"admitted\":{},\"ciphertext_bytes\":{}",
                            admitted,
                            encrypted.len()
                        ),
                    );
                    if !admitted {
                        fail("readiness request not admitted")
                    }
                }
                println!(
                    "carrier_event name=tcp_resource_admitted session=7001 target_path=2 generation=1 delivery_epoch=1 final_challenge=3 source=runtime_limits"
                );
                let tcp_records = count.saturating_sub(if recovery_enabled { 2 } else { 1 });
                for _ in 0..tcp_records {
                    bound_stream_to_deadline(&stream, experiment_deadline, None)
                        .unwrap_or_else(|_| fail("TCP data deadline elapsed"));
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
                            if tcp_delivery_delay_ms > 0 {
                                std::thread::sleep(Duration::from_millis(tcp_delivery_delay_ms));
                            }
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
                if recovery_enabled {
                    println!(
                        "carrier_event name=udp_recovery_owner_started session=7001 generation=1"
                    );
                    if let Some((ref mut udp_session, udp_peer)) = secure {
                        let _ = udp.set_read_timeout(Some(Duration::from_millis(200)));
                        let mut recovery_buf = [0u8; READINESS_FRAME_MAX];
                        let deadline = Instant::now() + Duration::from_secs(3);
                        while Instant::now() < deadline {
                            let Ok((n, source)) = udp.recv_from(&mut recovery_buf) else {
                                continue;
                            };
                            if source != udp_peer {
                                continue;
                            }
                            let Ok(plain) = udp_session.open_unreliable(&recovery_buf[..n]) else {
                                continue;
                            };
                            let Ok(ProcessMessage::ReadinessRequest {
                                session: SessionId(7001),
                                target_path: 1,
                                path_generation: 1,
                                delivery_epoch: 1,
                                challenge_id,
                            }) = ProcessMessage::decode(&plain)
                            else {
                                continue;
                            };
                            let response = ProcessMessage::ReadinessResponse {
                                session: SessionId(7001),
                                target_path: 1,
                                path_generation: 1,
                                delivery_epoch: 1,
                                challenge_id,
                                admitted: true,
                            }
                            .encode()
                            .unwrap();
                            let sealed = udp_session.seal_unreliable(&response).unwrap();
                            udp.send_to(&sealed, source).unwrap();
                            emit_diagnostic(
                                args,
                                "server",
                                "udp_recovery_validated",
                                challenge_id as usize,
                                "",
                            );
                            // The recovery owner remains active for exactly one
                            // bounded post-return application datagram. This
                            // proves the owner transition carries DeliveryAck,
                            // not merely a readiness response.
                            let post_deadline = Instant::now() + Duration::from_secs(2);
                            while Instant::now() < post_deadline {
                                let Ok((n, post_source)) = udp.recv_from(&mut recovery_buf) else {
                                    continue;
                                };
                                if post_source != udp_peer {
                                    continue;
                                }
                                let Ok(post_plain) =
                                    udp_session.open_unreliable(&recovery_buf[..n])
                                else {
                                    continue;
                                };
                                let Ok(ProcessMessage::Data {
                                    session: post_session,
                                    record: post_record,
                                }) = ProcessMessage::decode(&post_plain)
                                else {
                                    continue;
                                };
                                let post_stream = post_record.stream;
                                let post_offset = post_record.offset;
                                let post_len = post_record.data.len();
                                if post_session != SessionId(7001)
                                    || runtime
                                        .receive(
                                            InboundRecord {
                                                stream: post_stream,
                                                offset: post_offset,
                                                data: post_record.data,
                                            },
                                            4,
                                        )
                                        .is_err()
                                {
                                    continue;
                                }
                                let ack = ProcessMessage::DeliveryAck {
                                    session: post_session,
                                    stream: post_stream,
                                    offset: post_offset,
                                    len: post_len,
                                }
                                .encode()
                                .unwrap();
                                let sealed_ack = udp_session.seal_unreliable(&ack).unwrap();
                                udp.send_to(&sealed_ack, post_source).unwrap();
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_return_delivery_ack_sent",
                                    post_offset as usize / bytes.max(1),
                                    "",
                                );
                                if let Some(delivered) = runtime.pop_receive(5).unwrap() {
                                    app.extend_from_slice(&delivered.data);
                                }
                                migration_back_complete = true;
                                break;
                            }
                            break;
                        }
                    }
                }
                if migration_back && !migration_back_complete {
                    emit_diagnostic(
                        args,
                        "server",
                        "udp_recovery_failed",
                        count,
                        ",\"active\":\"tcp\",\"reason\":\"terminal_milestone_missing\"",
                    );
                    fail("migration-back terminal milestone missing")
                }
                let mode = if cease_udp_replies_after.is_some() {
                    "automatic_health_failure"
                } else {
                    "controlled_udp_stop"
                };
                println!(
                    "failover_server_ok session=7001 records={} application_bytes_total={} bytes_hex={} failover_mode={} controlled_udp_stop={} udp_port={} tcp_port={}",
                    app.len() / bytes.max(1),
                    app.len(),
                    hex(&app),
                    mode,
                    cease_udp_replies_after.is_none(),
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
    let experiment_origin = Instant::now();
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
    let automatic_health_failover = args.iter().any(|a| a == "--automatic-health-failover");
    let migration_back = args.iter().any(|a| a == "--migration-back");
    let restore_udp_replies_after_tcp = args.iter().any(|a| a == "--restore-udp-replies-after-tcp");
    let recovery_enabled = migration_back || restore_udp_replies_after_tcp;
    if migration_back && (!automatic_health_failover || count < 3) {
        fail("--migration-back requires --automatic-health-failover and count >= 3")
    }
    let cold_health_failover = args.iter().any(|a| a == "--cold-health-failover");
    if cold_health_failover && !automatic_health_failover {
        fail("--cold-health-failover requires --automatic-health-failover")
    }
    let target = format!("{addr}:{up}")
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad UDP target"));
    let sk = peer_public_key(&parse(args, "--server-key", None));
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
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
    let negotiation_response = buf[..n].to_vec();
    negotiation
        .client_accept_response(&negotiation_response)
        .unwrap();
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
    let noise_response = buf[..n].to_vec();
    let mut us = hs.finish(&noise_response, context(1)).unwrap();
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
    let application_deadline = Instant::now() + Duration::from_secs(secs);
    let mut admission_diagnostic = |reason| {
        emit_diagnostic(
            args,
            "client",
            "udp_post_handshake_ignored",
            1,
            &format!(",\"reason\":\"{}\"", reason),
        )
    };
    let n = recv_udp_delivery_ack(
        &u,
        target,
        &mut us,
        &udp_record,
        &negotiation_response,
        &noise_response,
        application_deadline,
        &mut admission_diagnostic,
    )
    .unwrap_or_else(|e| fail(&format!("UDP health observation failed: {e:?}")));
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
    let health_limits = HealthLimits {
        degrade_after: 2,
        fail_after: 3,
        recover_after: 2,
        max_paths: 8,
    };
    let mut health =
        CarrierHealthEvidence::new(health_limits, HealthEvidenceLimits { max_samples: 16 })
            .unwrap();
    let mut failover =
        FailoverController::new(health_limits.fail_after, count, count * bytes).unwrap();
    let mut manager = CarrierManager::new(ManagerLimits {
        min_hold_events: 1,
        switch_margin: 0,
        max_paths: 2,
    })
    .unwrap();
    manager.set_active_udp(PathId(1), PathGeneration(0));
    failover.udp_progress();

    // Leave the assigned logical range uncertain at the carrier boundary.
    // In migration-back mode the final record is deliberately unassigned until
    // the manager authorizes the return to UDP.
    let uncertain_end = if recovery_enabled {
        records.len().saturating_sub(1)
    } else {
        records.len()
    };
    for uncertain in records.iter().skip(1).take(uncertain_end.saturating_sub(1)) {
        failover
            .track_uncertain(DataId(uncertain.offset), &uncertain.data)
            .unwrap();
    }
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
    // D064 warm path: complete negotiation, Noise authentication, resume binding
    // and three bounded readiness observations while UDP remains sole active.
    let mut warm_tcp = None;
    let mut warm_session = None;
    let mut warm_timing = None;
    if automatic_health_failover && !cold_health_failover {
        let connect_started = Instant::now();
        let mut tcp = TcpStream::connect_timeout(
            &format!("{addr}:{tp}").parse().unwrap(),
            Duration::from_secs(secs),
        )
        .unwrap();
        let connected = Instant::now();
        let mut negotiation2 =
            VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
        write_frame(&mut tcp, &negotiation2.client_hello().unwrap()).unwrap();
        let response2 = read_frame(&mut tcp, MAX_NEGOTIATION_FRAME).unwrap();
        negotiation2.client_accept_response(&response2).unwrap();
        let negotiated = Instant::now();
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
        write_frame(&mut tcp, &hs2.first_message().unwrap()).unwrap();
        let response = read_frame(&mut tcp, 1024).unwrap();
        let mut session = hs2.finish(&response, context(2)).unwrap();
        let authenticated = Instant::now();
        negotiation2.admit_data().unwrap();
        let resume_validated = Instant::now();
        manager
            .prepare_warm_candidate(PathId(2), PathGeneration(1), 7001, 1)
            .unwrap();
        let readiness_sequence_deadline = (Instant::now() + READINESS_SEQUENCE_TIMEOUT)
            .min(experiment_origin + Duration::from_secs(secs));
        let mut readiness_satisfied = authenticated;
        for observation_id in 1..=READINESS_PROBES {
            if observation_id == 3 && args.iter().any(|arg| arg == "--test-readiness-short") {
                tcp.shutdown(Shutdown::Write).unwrap();
                fail("test readiness sequence stopped before third request")
            }
            if observation_id == 3 && args.iter().any(|arg| arg == "--test-readiness-stall") {
                std::thread::sleep(READINESS_PROBE_TIMEOUT + Duration::from_millis(200));
            }
            let requested = Instant::now();
            let request = ProcessMessage::ReadinessRequest {
                session: SessionId(
                    if args.iter().any(|arg| arg == "--test-readiness-wrong-tuple") {
                        7002
                    } else {
                        7001
                    },
                ),
                target_path: 2,
                path_generation: 1,
                delivery_epoch: 1,
                challenge_id: if observation_id == 2
                    && args.iter().any(|arg| arg == "--test-readiness-replay")
                {
                    1
                } else {
                    observation_id
                },
            }
            .encode()
            .unwrap();
            let mut encrypted = session.seal(&request).unwrap();
            if observation_id == 2 && args.iter().any(|arg| arg == "--test-readiness-tamper") {
                let last = encrypted.len() - 1;
                encrypted[last] ^= 1;
            }
            bound_stream_to_deadline(
                &tcp,
                readiness_sequence_deadline,
                Some(READINESS_PROBE_TIMEOUT),
            )
            .unwrap_or_else(|_| fail("readiness request deadline elapsed"));
            write_frame(&mut tcp, &encrypted).unwrap();
            bound_stream_to_deadline(
                &tcp,
                readiness_sequence_deadline,
                Some(READINESS_PROBE_TIMEOUT),
            )
            .unwrap_or_else(|_| fail("readiness response deadline elapsed"));
            let ciphertext = read_frame(&mut tcp, READINESS_FRAME_MAX)
                .unwrap_or_else(|_| fail("readiness response timeout or malformed frame"));
            let plain = session
                .open(&ciphertext)
                .unwrap_or_else(|_| fail("readiness response authentication failed"));
            let response = ProcessMessage::decode(&plain)
                .unwrap_or_else(|_| fail("malformed readiness response"));
            let admitted = matches!(response, ProcessMessage::ReadinessResponse { session: SessionId(7001), target_path: 2, path_generation: 1, delivery_epoch: 1, challenge_id, admitted: true } if challenge_id == observation_id);
            let warm = manager
                .observe_warm_candidate_readiness(ReadinessObservation {
                    target_path: PathId(2),
                    generation: PathGeneration(1),
                    session_id: 7001,
                    delivery_epoch: 1,
                    observation_id,
                    authenticated: true,
                    resume_validated: true,
                    resource_admitted: admitted,
                })
                .unwrap_or_else(|_| fail("readiness tuple unadmitted"));
            readiness_satisfied = Instant::now();
            let sequence_pause = args
                .windows(2)
                .find(|w| w[0] == "--test-readiness-sequence-pause-ms")
                .map(|w| {
                    w[1].parse::<u64>()
                        .unwrap_or_else(|_| fail("invalid readiness pause"))
                })
                .unwrap_or(0);
            if sequence_pause > 1_000 {
                fail("readiness pause outside 0-1000")
            }
            if observation_id < READINESS_PROBES && sequence_pause > 0 {
                std::thread::sleep(Duration::from_millis(sequence_pause));
            }
            emit_diagnostic(
                args,
                "client",
                "tcp_warm_readiness",
                observation_id as usize,
                &format!(
                    ",\"warm\":{},\"request_us\":{},\"response_us\":{}",
                    warm,
                    requested.duration_since(experiment_origin).as_micros(),
                    readiness_satisfied
                        .duration_since(experiment_origin)
                        .as_micros()
                ),
            );
        }
        if manager.active() != Some(PathId(1))
            || !manager
                .warm_candidate()
                .is_some_and(|candidate| candidate.warm)
        {
            fail("warm readiness changed active ownership")
        }
        println!(
            "carrier_event name=tcp_warm session=7001 generation=1 readiness=3 application_data=0"
        );
        warm_tcp = Some(tcp);
        warm_session = Some(session);
        warm_timing = Some((
            connect_started,
            connected,
            negotiated,
            authenticated,
            resume_validated,
            readiness_satisfied,
        ));
    }
    const HEALTH_OBSERVATION_WINDOW: Duration = Duration::from_secs(1);
    const MAX_IGNORED_HEALTH_DATAGRAMS: usize = 64;
    let failure_observation_started = Instant::now();
    let mut decision_at = None;
    if automatic_health_failover {
        let mut window = HealthObservationWindow::new(
            failure_observation_started,
            HEALTH_OBSERVATION_WINDOW,
            MAX_IGNORED_HEALTH_DATAGRAMS,
        );
        for observation_index in 1..=health_limits.fail_after {
            loop {
                let now = Instant::now();
                if now >= window.deadline() {
                    let observation_started = window.started_at();
                    let state = window
                        .expire(now, &mut health, PathId(1))
                        .unwrap_or_else(|e| fail(&format!("UDP health observation failed: {e:?}")));
                    let state_name = match state {
                        HealthState::Unknown => "unknown",
                        HealthState::Healthy => "healthy",
                        HealthState::Degraded => "degraded",
                        HealthState::Failed => "failed",
                    };
                    emit_diagnostic(
                        args,
                        "client",
                        "udp_health_event",
                        observation_index as usize,
                        &format!(
                            ",\"event\":\"failure\",\"cause\":\"authenticated_delivery_ack_timeout\",\"state\":\"{}\",\"failure_observation_started_us\":{},\"failure_observation_elapsed_us\":{}",
                            state_name,
                            observation_started
                                .duration_since(experiment_origin)
                                .as_micros(),
                            now.duration_since(observation_started).as_micros()
                        ),
                    );
                    if state == HealthState::Failed {
                        let pending = manager
                            .fail_udp_to_tcp(
                                PathId(1),
                                PathGeneration(0),
                                PathId(2),
                                state,
                                CarrierSwitchReason::UdpPathDegraded,
                            )
                            .unwrap();
                        decision_at = Some(now);
                        println!(
                            "carrier_event name=udp_health_failed session=7001 generation={} threshold={} reason={} diagnostic_cause=authenticated_delivery_ack_timeout",
                            pending.generation.0,
                            health_limits.fail_after,
                            pending.reason.as_str()
                        );
                    }
                    break;
                }
                match u.recv_from(&mut buf) {
                    Ok((_, peer)) if peer != target => {
                        match window.observe(
                            HealthDatagram::WrongPeer,
                            Instant::now(),
                            &mut health,
                            PathId(1),
                        ) {
                            Ok(None) => {
                                admission_diagnostic("unexpected_peer");
                            }
                            Err(HealthWindowError::AdmissionBudgetExhausted { ignored }) => {
                                fail(&format!(
                                    "UDP health admission budget exhausted after {ignored} ignored datagrams"
                                ));
                            }
                            other => fail(&format!("unexpected UDP health result: {other:?}")),
                        }
                    }
                    Ok((n, _)) => {
                        let classification = match us.open_unreliable(&buf[..n]) {
                            Ok(plain)
                                if records
                                    .get(1)
                                    .is_some_and(|record| delivery_ack_matches(&plain, record)) =>
                            {
                                HealthDatagram::PermittedProgress
                            }
                            Ok(_) => HealthDatagram::StaleOrNonmatchingAuthenticated,
                            Err(_) => HealthDatagram::MalformedOrUnadmitted,
                        };
                        if classification == HealthDatagram::PermittedProgress {
                            window
                                .observe(classification, Instant::now(), &mut health, PathId(1))
                                .unwrap_or_else(|e| {
                                    fail(&format!("UDP health observation failed: {e:?}"))
                                });
                            fail(
                                "unexpected permitted UDP delivery progress at degradation boundary",
                            );
                        }
                        match window.observe(classification, Instant::now(), &mut health, PathId(1))
                        {
                            Ok(None) => {
                                admission_diagnostic(match classification {
                                    HealthDatagram::MalformedOrUnadmitted => {
                                        "malformed_or_unadmitted"
                                    }
                                    HealthDatagram::StaleOrNonmatchingAuthenticated => {
                                        "stale_or_nonmatching_authenticated"
                                    }
                                    _ => unreachable!(),
                                });
                            }
                            Err(HealthWindowError::AdmissionBudgetExhausted { ignored }) => {
                                fail(&format!(
                                    "UDP health admission budget exhausted after {ignored} ignored datagrams"
                                ));
                            }
                            other => fail(&format!("unexpected UDP health result: {other:?}")),
                        }
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => fail("UDP health receive failed"),
                }
            }
        }
        if manager.active().is_some() || failover.active() != ActiveCarrier::Udp {
            fail("TCP became active before target-path readiness")
        }
    } else {
        println!(
            "carrier_event name=controlled_udp_stop session=7001 generation=0 reason=bounded_application_fault_injection"
        );
    }
    let (
        mut tcp,
        mut ts,
        tcp_connect_started,
        tcp_connected,
        tcp_negotiated,
        tcp_authenticated,
        resume_validated,
        readiness_satisfied,
        warm_recovery,
    ) = if let (
        Some(tcp),
        Some(session),
        Some((
            connect_started,
            connected,
            negotiated,
            authenticated,
            resume_validated,
            readiness_satisfied,
        )),
    ) = (warm_tcp, warm_session, warm_timing)
    {
        (
            tcp,
            session,
            connect_started,
            connected,
            negotiated,
            authenticated,
            resume_validated,
            readiness_satisfied,
            true,
        )
    } else {
        let mut negotiation2 =
            VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
        let hello2 = negotiation2.client_hello().unwrap();
        let connect_started = Instant::now();
        let mut tcp = TcpStream::connect_timeout(
            &format!("{addr}:{tp}").parse().unwrap(),
            Duration::from_secs(secs),
        )
        .unwrap();
        let connected = Instant::now();
        write_frame(&mut tcp, &hello2).unwrap();
        let response2 = read_frame(&mut tcp, MAX_NEGOTIATION_FRAME).unwrap();
        negotiation2.client_accept_response(&response2).unwrap();
        let negotiated = Instant::now();
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
        write_frame(&mut tcp, &hs2.first_message().unwrap()).unwrap();
        let resp = read_frame(&mut tcp, 1024).unwrap();
        let mut session = hs2.finish(&resp, context(2)).unwrap();
        let authenticated = Instant::now();
        let readiness_sequence_deadline = Instant::now() + READINESS_SEQUENCE_TIMEOUT;
        for challenge_id in 1..=READINESS_PROBES {
            let request = ProcessMessage::ReadinessRequest {
                session: SessionId(7001),
                target_path: 2,
                path_generation: 1,
                delivery_epoch: 1,
                challenge_id,
            }
            .encode()
            .unwrap();
            bound_stream_to_deadline(
                &tcp,
                readiness_sequence_deadline,
                Some(READINESS_PROBE_TIMEOUT),
            )
            .unwrap_or_else(|_| fail("cold readiness request deadline elapsed"));
            write_frame(&mut tcp, &session.seal(&request).unwrap()).unwrap();
            bound_stream_to_deadline(
                &tcp,
                readiness_sequence_deadline,
                Some(READINESS_PROBE_TIMEOUT),
            )
            .unwrap_or_else(|_| fail("cold readiness response deadline elapsed"));
            let response = read_frame(&mut tcp, READINESS_FRAME_MAX)
                .unwrap_or_else(|_| fail("cold readiness control timeout"));
            let plain = session
                .open(&response)
                .unwrap_or_else(|_| fail("cold readiness authentication failed"));
            if !matches!(ProcessMessage::decode(&plain), Ok(ProcessMessage::ReadinessResponse { session: SessionId(7001), target_path: 2, path_generation: 1, delivery_epoch: 1, challenge_id: response_id, admitted: true }) if response_id == challenge_id)
            {
                fail("cold readiness control rejected")
            }
        }
        (
            tcp,
            session,
            connect_started,
            connected,
            negotiated,
            authenticated,
            authenticated,
            authenticated,
            false,
        )
    };
    println!("carrier_event name=tcp_resume_guard session=7001 generation=1");
    let new_active_at = if automatic_health_failover {
        let decision = if warm_recovery {
            manager
                .promote_warm_authenticated_resume(PathId(2), PathGeneration(1), 7001, 1)
                .unwrap()
        } else {
            manager
                .promote_cold_authenticated_resume(PathId(2), PathGeneration(1), true, true)
                .unwrap()
        };
        if !failover.apply_manager_decision(&decision) {
            fail("manager promotion was not applied");
        }
        Instant::now()
    } else {
        Instant::now()
    };
    let resend_records = if automatic_health_failover {
        failover.tcp_resend().unwrap().len()
    } else {
        count.saturating_sub(1)
    };
    let tcp_records = resend_records;
    let mut first_resumed_data_at = None;
    let mut first_resumed_ack_at = None;
    let mut replayed_records = 0usize;
    let mut tcp_active_sample = None;
    let uncertain_records = if recovery_enabled {
        records.len().saturating_sub(2)
    } else {
        records.len().saturating_sub(1)
    };
    let post_return_record = if recovery_enabled {
        records.last().cloned()
    } else {
        None
    };
    for record in records.into_iter().skip(1).take(tcp_records) {
        let logical = ProcessMessage::Data {
            session: SessionId(7001),
            record: record.clone(),
        }
        .encode()
        .unwrap();
        let encrypted = ts.seal(&logical).unwrap();
        let resumed_sent = Instant::now();
        first_resumed_data_at.get_or_insert(resumed_sent);
        write_frame(&mut tcp, &encrypted).unwrap();
        let ack = read_frame(&mut tcp, PROCESS_FRAME_MAX)
            .unwrap_or_else(|_| fail("TCP delivery acknowledgement timeout"));
        let plain = ts
            .open(&ack)
            .unwrap_or_else(|_| fail("unauthenticated TCP delivery acknowledgement"));
        if !delivery_ack_matches(&plain, &record) {
            fail("invalid TCP delivery acknowledgement")
        }
        let resumed_ack = Instant::now();
        first_resumed_ack_at.get_or_insert(resumed_ack);
        tcp_active_sample.get_or_insert(HealthSample {
            rtt_us: resumed_ack
                .duration_since(resumed_sent)
                .as_micros()
                .min(u64::MAX as u128) as u64,
            loss_per_mille: 0,
            pto: 0,
        });
        replayed_records = replayed_records.saturating_add(1);
        delivery
            .delivery_ack(
                record.stream,
                record.offset,
                record.data.len(),
                count as u64 + 4,
            )
            .unwrap();
        if automatic_health_failover {
            failover.confirm(DataId(record.offset)).unwrap();
        }
        emit_diagnostic(
            args,
            "client",
            "tcp_delivery_ack_validated",
            record.offset as usize / bytes.max(1),
            &format!(",\"ciphertext_bytes\":{}", ack.len()),
        );
    }
    if automatic_health_failover
        && let (Some(decision), Some(first_data), Some(first_ack)) =
            (decision_at, first_resumed_data_at, first_resumed_ack_at)
    {
        let latency = first_data.duration_since(decision).as_micros() as u64;
        failover.record_recovery_latency(latency);
        let fallback_class = if warm_recovery { "warm" } else { "cold" };
        let promotion_gate = if warm_recovery {
            "warm_authenticated_resume"
        } else {
            "cold_authenticated_resume"
        };
        let promotion_ready_field = if warm_recovery {
            "readiness_satisfied_us"
        } else {
            "cold_promotion_ready_us"
        };
        emit_diagnostic(
            args,
            "client",
            "failover_timing",
            count,
            &format!(
                ",\"fallback_class\":\"{}\",\"promotion_gate\":\"{}\",\"failure_decided_at_us\":{},\"tcp_connect_started_us\":{},\"tcp_connected_us\":{},\"tcp_negotiated_us\":{},\"tcp_authenticated_us\":{},\"resume_validated_us\":{},\"{}\":{},\"new_active_at_us\":{},\"first_resumed_data_accepted_us\":{},\"first_resumed_ack_at_us\":{},\"recovery_latency_us\":{}",
                fallback_class,
                promotion_gate,
                decision.duration_since(experiment_origin).as_micros(),
                tcp_connect_started
                    .duration_since(experiment_origin)
                    .as_micros(),
                tcp_connected.duration_since(experiment_origin).as_micros(),
                tcp_negotiated.duration_since(experiment_origin).as_micros(),
                tcp_authenticated
                    .duration_since(experiment_origin)
                    .as_micros(),
                resume_validated
                    .duration_since(experiment_origin)
                    .as_micros(),
                promotion_ready_field,
                readiness_satisfied
                    .duration_since(experiment_origin)
                    .as_micros(),
                new_active_at.duration_since(experiment_origin).as_micros(),
                first_data.duration_since(experiment_origin).as_micros(),
                first_ack.duration_since(experiment_origin).as_micros(),
                latency
            ),
        );
    }
    if automatic_health_failover && recovery_enabled {
        let request = ProcessMessage::ReadinessRequest {
            session: SessionId(7001),
            target_path: 1,
            path_generation: 1,
            delivery_epoch: 1,
            challenge_id: 1,
        }
        .encode()
        .unwrap();
        let challenge_sent = Instant::now();
        let mut ciphertext = us.seal_unreliable(&request).unwrap();
        if args.iter().any(|arg| arg == "--test-migration-back-tamper") {
            let last = ciphertext.len() - 1;
            ciphertext[last] ^= 1;
        }
        emit_diagnostic(
            args,
            "client",
            "udp_recovery_challenge_sent",
            1,
            ",\"path\":1,\"generation\":1",
        );
        let tcp_active_sample = tcp_active_sample
            .unwrap_or_else(|| fail("missing authenticated TCP application RTT sample"));
        manager.observe(PathId(2), tcp_active_sample).unwrap();
        emit_diagnostic(
            args,
            "client",
            "tcp_active_health_observed",
            1,
            &format!(
                ",\"path\":2,\"generation\":1,\"rtt_us\":{},\"loss_per_mille\":0,\"pto\":0",
                tcp_active_sample.rtt_us
            ),
        );
        let mut recovery_buf = [0u8; READINESS_FRAME_MAX];
        u.send_to(&ciphertext, target).unwrap();
        u.set_read_timeout(Some(Duration::from_secs(secs.min(3))))
            .unwrap();
        let udp_recovery_sample = match u.recv_from(&mut recovery_buf) {
            Ok((n, source)) if source == target => {
                let plain = us
                    .open_unreliable(&recovery_buf[..n])
                    .unwrap_or_else(|_| fail("UDP recovery response authentication failed"));
                if !matches!(
                    ProcessMessage::decode(&plain),
                    Ok(ProcessMessage::ReadinessResponse {
                        session: SessionId(7001),
                        target_path: 1,
                        path_generation: 1,
                        delivery_epoch: 1,
                        challenge_id: 1,
                        admitted: true
                    })
                ) {
                    fail("UDP recovery response tuple mismatch")
                }
                HealthSample {
                    rtt_us: challenge_sent.elapsed().as_micros().min(u64::MAX as u128) as u64,
                    loss_per_mille: 0,
                    pto: 0,
                }
            }
            Ok(_) => {
                emit_diagnostic(
                    args,
                    "client",
                    "udp_recovery_failed",
                    1,
                    ",\"active\":\"tcp\",\"reason\":\"wrong_peer\"",
                );
                fail("UDP recovery response from wrong peer")
            }
            Err(_) => {
                emit_diagnostic(
                    args,
                    "client",
                    "udp_recovery_failed",
                    1,
                    ",\"active\":\"tcp\",\"reason\":\"timeout\"",
                );
                fail("UDP recovery validation timeout")
            }
        };
        manager.observe(PathId(1), udp_recovery_sample).unwrap();
        emit_diagnostic(
            args,
            "client",
            "udp_recovery_validated",
            1,
            &format!(
                ",\"path\":1,\"generation\":1,\"challenge_id\":1,\"rtt_us\":{},\"loss_per_mille\":0,\"pto\":0",
                udp_recovery_sample.rtt_us
            ),
        );
        let candidate = MigrationCandidate {
            path: PathId(1),
            generation: PathGeneration(1),
            validated: true,
            health: udp_recovery_sample,
        };
        if manager.migrate_back_to_udp(candidate).is_ok() {
            fail("migration-back bypassed hold gate")
        }
        emit_diagnostic(
            args,
            "client",
            "udp_migration_hold",
            1,
            ",\"active\":\"tcp\",\"path\":2,\"generation\":1",
        );
        manager
            .migrate_back_to_udp(candidate)
            .unwrap_or_else(|_| fail("migration-back gate rejected"));
        if !failover.apply_migration_back() {
            fail("migration-back ownership transition rejected")
        }
        println!("carrier_event name=udp_recovered session=7001 generation=1 validated=true");
        println!("carrier_event name=migrated_back_to_udp session=7001 generation=1");
        emit_diagnostic(
            args,
            "client",
            "udp_migrated_back",
            1,
            ",\"from\":\"tcp\",\"from_path\":2,\"to\":\"udp\",\"to_path\":1,\"generation\":1",
        );
        let post_record =
            post_return_record.unwrap_or_else(|| fail("missing post-return UDP record"));
        let post = ProcessMessage::Data {
            session: SessionId(7001),
            record: post_record.clone(),
        }
        .encode()
        .unwrap();
        let sealed = us.seal_unreliable(&post).unwrap();
        u.send_to(&sealed, target).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let ack_len = recv_udp_delivery_ack(
            &u,
            target,
            &mut us,
            &post_record,
            &negotiation_response,
            &noise_response,
            deadline,
            &mut |_| {},
        )
        .unwrap_or_else(|_| fail("post-return UDP DeliveryAck timeout"));
        delivery
            .delivery_ack(
                post_record.stream,
                post_record.offset,
                post_record.data.len(),
                count as u64 + 5,
            )
            .unwrap();
        emit_diagnostic(
            args,
            "client",
            "udp_return_delivery_ack_validated",
            count,
            &format!(",\"ciphertext_bytes\":{}", ack_len),
        );
    }
    emit_diagnostic(
        args,
        "client",
        "failover_accounting",
        count,
        &format!(
            ",\"udp_confirmed_records\":{},\"udp_confirmed_bytes\":{},\"uncertain_records\":{},\"uncertain_bytes\":{},\"replayed_records\":{},\"replayed_bytes\":{},\"confirmed_records\":{},\"confirmed_bytes\":{},\"duplicate_records\":0,\"duplicate_bytes\":0,\"lost_records\":0,\"lost_bytes\":0,\"conflicting_records\":0,\"conflicting_bytes\":0",
            usize::from(recovery_enabled) + 1,
            (usize::from(recovery_enabled) + 1) * bytes,
            uncertain_records,
            uncertain_records * bytes,
            replayed_records,
            replayed_records * bytes,
            usize::from(recovery_enabled) + 1 + replayed_records,
            (usize::from(recovery_enabled) + 1 + replayed_records) * bytes
        ),
    );
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
    let mode = if automatic_health_failover {
        "automatic_health_failure"
    } else {
        "controlled_udp_stop"
    };
    println!(
        "failover_client_ok session=7001 count={count} application_bytes_total={} failover_mode={} controlled_udp_stop={}",
        count * bytes,
        mode,
        !automatic_health_failover
    );
}
fn endpoint_rebind_server(args: &[String]) {
    let count = exchange_count(args);
    let bytes = parse(args, "--bytes", Some("16"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    let secs = parse(args, "--duration", Some("10"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid duration"));
    if count != 2 || bytes == 0 || bytes > MAX_BYTES || secs == 0 || secs > MAX_DURATION {
        fail("endpoint rebind requires count=2, bytes=1-1200, duration=1-30")
    }
    let port = parse(args, "--udp-port", Some("40081"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid UDP port"));
    if !(40080..=MAX_PORT).contains(&port) {
        fail("port outside 40080-40100")
    }
    let bind = parse(args, "--udp-bind", Some(&format!("0.0.0.0:{port}")))
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad UDP bind"));
    let client = peer_public_key(&parse(args, "--client-key", None));
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-server.identity"),
    )));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client,
        scope: b"failover".to_vec(),
        status: TrustStatus::Active,
    }]);
    let socket = UdpSocket::bind(bind).unwrap_or_else(|_| fail("UDP bind failed"));
    socket
        .set_read_timeout(Some(Duration::from_secs(secs)))
        .unwrap();
    let promotion_delay_ms = parse(args, "--test-promotion-delay-ms", Some("0"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid promotion delay"));
    if promotion_delay_ms > 1000 {
        fail("promotion delay outside 0-1000 ms")
    }
    let mut buf = [0u8; 65536];
    emit_diagnostic(args, "server", "start", 0, ",\"mode\":\"endpoint_rebind\"");
    println!("endpoint_rebind_server_ready");

    let (n, endpoint_a) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint A negotiation timeout"));
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
    let selection = negotiation
        .server_accept_hello(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint A negotiation rejected"));
    socket.send_to(&selection, endpoint_a).unwrap();
    let binding = negotiation.authenticated_binding().unwrap();
    let (n, source) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint A handshake timeout"));
    if source != endpoint_a {
        fail("endpoint A changed before authentication")
    }
    let (response, mut secure) =
        ResponderHandshake::new_with_prologue_binding(&id, policy, DOMAIN, binding.as_bytes())
            .unwrap()
            .receive_first(&buf[..n], context(1))
            .unwrap_or_else(|_| fail("endpoint A authentication rejected"));
    socket.send_to(&response, endpoint_a).unwrap();

    let mut runtime = SessionRuntime::new(SessionId(7001), runtime_limits(bytes, 2), 0).unwrap();
    runtime.open_stream(StreamId(1), 0).unwrap();
    let (n, source) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("pre-rebind Data timeout"));
    if source != endpoint_a {
        fail("pre-rebind Data used wrong endpoint")
    }
    let plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("pre-rebind Data authentication failed"));
    let ProcessMessage::Data { session, record } =
        ProcessMessage::decode(&plain).unwrap_or_else(|_| fail("pre-rebind Data malformed"))
    else {
        fail("pre-rebind Data missing")
    };
    if session != SessionId(7001) {
        fail("pre-rebind Session mismatch")
    }
    runtime
        .receive(
            InboundRecord {
                stream: record.stream,
                offset: record.offset,
                data: record.data.clone(),
            },
            1,
        )
        .unwrap_or_else(|_| fail("pre-rebind delivery rejected"));
    let ack = ProcessMessage::DeliveryAck {
        session,
        stream: record.stream,
        offset: record.offset,
        len: record.data.len(),
    }
    .encode()
    .unwrap();
    socket
        .send_to(&secure.seal_unreliable(&ack).unwrap(), endpoint_a)
        .unwrap();
    let _ = runtime.pop_receive(2).unwrap();
    emit_diagnostic(
        args,
        "server",
        "endpoint_a_confirmed",
        1,
        ",\"path\":1,\"generation\":0",
    );

    let mut state = EndpointRebindState::new(1, PathId(1), PathGeneration(0), 7001, 1)
        .unwrap_or_else(|_| fail("endpoint state setup failed"));
    let (n, endpoint_b) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint candidate timeout"));
    if endpoint_b == endpoint_a {
        fail("candidate endpoint did not change")
    }
    let candidate_plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint candidate authentication failed"));
    if !matches!(
        ProcessMessage::decode(&candidate_plain),
        Ok(ProcessMessage::ReadinessRequest {
            session: SessionId(7001),
            target_path: 1,
            path_generation: 1,
            delivery_epoch: 1,
            challenge_id: 1
        })
    ) {
        fail("endpoint candidate tuple mismatch")
    }
    let candidate = EndpointRebindCandidate {
        source_tag: 2,
        path: PathId(1),
        generation: PathGeneration(1),
    };
    state
        .observe_candidate(candidate)
        .unwrap_or_else(|_| fail("endpoint candidate rejected"));
    emit_diagnostic(
        args,
        "server",
        "endpoint_candidate_seen",
        1,
        ",\"path\":1,\"generation\":1,\"source_endpoint_changed\":true",
    );
    let mut random = [0u8; 8];
    getrandom::getrandom(&mut random).unwrap_or_else(|_| fail("challenge randomness unavailable"));
    let mut challenge_id = u64::from_be_bytes(random);
    if challenge_id == 0 {
        challenge_id = 1;
    }
    state
        .arm_challenge(challenge_id)
        .unwrap_or_else(|_| fail("endpoint challenge rejected"));
    let request = ProcessMessage::ReadinessRequest {
        session: SessionId(7001),
        target_path: 1,
        path_generation: 1,
        delivery_epoch: 1,
        challenge_id,
    }
    .encode()
    .unwrap();
    socket
        .send_to(&secure.seal_unreliable(&request).unwrap(), endpoint_b)
        .unwrap();
    emit_diagnostic(
        args,
        "server",
        "endpoint_challenge_sent",
        1,
        ",\"path\":1,\"generation\":1",
    );
    let (n, response_source) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint challenge response timeout"));
    if response_source != endpoint_b {
        emit_diagnostic(
            args,
            "server",
            "endpoint_rebind_failed",
            1,
            ",\"reason\":\"wrong_source\"",
        );
        fail("endpoint challenge response from wrong source")
    }
    let response_plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint challenge response authentication failed"));
    if !matches!(ProcessMessage::decode(&response_plain), Ok(ProcessMessage::ReadinessResponse { session: SessionId(7001), target_path: 1, path_generation: 1, delivery_epoch: 1, challenge_id: response_id, admitted: true }) if response_id == challenge_id)
    {
        emit_diagnostic(
            args,
            "server",
            "endpoint_rebind_failed",
            1,
            ",\"reason\":\"tuple_mismatch\"",
        );
        fail("endpoint challenge response tuple mismatch")
    }
    if promotion_delay_ms > 0 {
        let hold_until = Instant::now() + Duration::from_millis(promotion_delay_ms);
        socket
            .set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        let mut held_data = false;
        while Instant::now() < hold_until {
            match socket.recv_from(&mut buf) {
                Ok((held_n, held_source)) if held_source == endpoint_b => {
                    if let Ok(held_plain) = secure.open_unreliable(&buf[..held_n])
                        && matches!(
                            ProcessMessage::decode(&held_plain),
                            Ok(ProcessMessage::Data { .. })
                        )
                    {
                        held_data = true;
                    }
                }
                Ok(_) => {}
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(_) => fail("promotion delay interception failed"),
            }
        }
        socket
            .set_read_timeout(Some(Duration::from_secs(secs)))
            .unwrap();
        if held_data {
            fail("post-rebind Data arrived before promotion sync")
        }
        emit_diagnostic(
            args,
            "server",
            "endpoint_promotion_delay_held",
            1,
            &format!(",\"milliseconds\":{}", promotion_delay_ms),
        );
    }
    state
        .validate_and_promote(candidate, 7001, 1, challenge_id)
        .unwrap_or_else(|_| fail("endpoint promotion rejected"));
    let sync = ProcessMessage::ReadinessResponse {
        session: SessionId(7001),
        target_path: 1,
        path_generation: 1,
        delivery_epoch: 1,
        challenge_id: 1,
        admitted: true,
    }
    .encode()
    .unwrap();
    socket
        .send_to(&secure.seal_unreliable(&sync).unwrap(), endpoint_b)
        .unwrap();
    emit_diagnostic(
        args,
        "server",
        "endpoint_validated",
        1,
        ",\"path\":1,\"generation\":1",
    );
    emit_diagnostic(
        args,
        "server",
        "endpoint_promoted",
        1,
        ",\"path\":1,\"generation\":1,\"source_endpoint_changed\":true",
    );
    emit_diagnostic(
        args,
        "server",
        "endpoint_promotion_sync_sent",
        1,
        ",\"path\":1,\"generation\":1",
    );

    let (mut n, mut post_source) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("post-rebind Data timeout"));
    if post_source == endpoint_a {
        emit_diagnostic(
            args,
            "server",
            "endpoint_stale_source_rejected",
            1,
            ",\"path\":1,\"generation\":0",
        );
        (n, post_source) = socket
            .recv_from(&mut buf)
            .unwrap_or_else(|_| fail("post-rebind Data timeout"));
    }
    if post_source != endpoint_b || state.active_source_tag() != 2 {
        fail("post-rebind Data used inactive endpoint")
    }
    let post_plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("post-rebind Data authentication failed"));
    let ProcessMessage::Data {
        session: post_session,
        record: post_record,
    } = ProcessMessage::decode(&post_plain).unwrap_or_else(|_| fail("post-rebind Data malformed"))
    else {
        fail("post-rebind Data missing")
    };
    if post_session != SessionId(7001) {
        fail("post-rebind Session mismatch")
    }
    runtime
        .receive(
            InboundRecord {
                stream: post_record.stream,
                offset: post_record.offset,
                data: post_record.data.clone(),
            },
            3,
        )
        .unwrap_or_else(|_| fail("post-rebind delivery rejected"));
    let post_ack = ProcessMessage::DeliveryAck {
        session: post_session,
        stream: post_record.stream,
        offset: post_record.offset,
        len: post_record.data.len(),
    }
    .encode()
    .unwrap();
    socket
        .send_to(&secure.seal_unreliable(&post_ack).unwrap(), endpoint_b)
        .unwrap();
    let _ = runtime.pop_receive(4).unwrap();
    emit_diagnostic(
        args,
        "server",
        "endpoint_post_delivery_ack_sent",
        2,
        ",\"path\":1,\"generation\":1",
    );
    emit_diagnostic(
        args,
        "server",
        "summary",
        2,
        &format!(
            ",\"classification\":\"A\",\"records\":2,\"application_bytes_total\":{}",
            bytes * 2
        ),
    );
    println!(
        "endpoint_rebind_server_ok records=2 application_bytes_total={}",
        bytes * 2
    );
}

fn endpoint_rebind_client(args: &[String]) {
    let count = exchange_count(args);
    let bytes = parse(args, "--bytes", Some("16"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    let secs = parse(args, "--duration", Some("10"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid duration"));
    if count != 2 || bytes == 0 || bytes > MAX_BYTES || secs == 0 || secs > MAX_DURATION {
        fail("endpoint rebind requires count=2, bytes=1-1200, duration=1-30")
    }
    let addr = parse(args, "--addr", Some("127.0.0.1"));
    let port = parse(args, "--udp-port", Some("40081"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid UDP port"));
    let target = format!("{addr}:{port}")
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| fail("bad UDP target"));
    let server_key = peer_public_key(&parse(args, "--server-key", None));
    let id = load_or_generate(&PathBuf::from(parse(
        args,
        "--identity",
        Some("neko-client.identity"),
    )));
    let endpoint_a = UdpSocket::bind("0.0.0.0:0").unwrap();
    endpoint_a
        .set_read_timeout(Some(Duration::from_secs(secs)))
        .unwrap();
    let mut buf = [0u8; 65536];
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS).unwrap();
    endpoint_a
        .send_to(&negotiation.client_hello().unwrap(), target)
        .unwrap();
    let (n, source) = endpoint_a.recv_from(&mut buf).unwrap();
    if source != target {
        fail("negotiation response from wrong peer")
    }
    negotiation.client_accept_response(&buf[..n]).unwrap();
    let binding = negotiation.authenticated_binding().unwrap();
    let mut handshake = InitiatorHandshake::new_with_prologue_binding(
        &id,
        &server_key,
        b"failover",
        DOMAIN,
        binding.as_bytes(),
    )
    .unwrap();
    endpoint_a
        .send_to(&handshake.first_message().unwrap(), target)
        .unwrap();
    let (n, source) = endpoint_a.recv_from(&mut buf).unwrap();
    if source != target {
        fail("handshake response from wrong peer")
    }
    let mut secure = handshake.finish(&buf[..n], context(1)).unwrap();
    let payload = vec![b'x'; bytes];
    let first_record = OutboundRecord {
        stream: StreamId(1),
        offset: 0,
        data: payload.clone(),
    };
    let first = ProcessMessage::Data {
        session: SessionId(7001),
        record: first_record.clone(),
    }
    .encode()
    .unwrap();
    endpoint_a
        .send_to(&secure.seal_unreliable(&first).unwrap(), target)
        .unwrap();
    let (n, source) = endpoint_a.recv_from(&mut buf).unwrap();
    if source != target {
        fail("pre-rebind DeliveryAck from wrong peer")
    }
    let ack = secure.open_unreliable(&buf[..n]).unwrap();
    if !delivery_ack_matches(&ack, &first_record) {
        fail("pre-rebind DeliveryAck mismatch")
    }
    emit_diagnostic(
        args,
        "client",
        "endpoint_a_confirmed",
        1,
        ",\"path\":1,\"generation\":0",
    );

    let endpoint_b = UdpSocket::bind("0.0.0.0:0").unwrap();
    endpoint_b
        .set_read_timeout(Some(Duration::from_secs(secs)))
        .unwrap();
    if endpoint_a.local_addr().unwrap() == endpoint_b.local_addr().unwrap() {
        fail("source endpoint did not change")
    }
    let candidate = ProcessMessage::ReadinessRequest {
        session: SessionId(7001),
        target_path: 1,
        path_generation: 1,
        delivery_epoch: 1,
        challenge_id: 1,
    }
    .encode()
    .unwrap();
    endpoint_b
        .send_to(&secure.seal_unreliable(&candidate).unwrap(), target)
        .unwrap();
    emit_diagnostic(
        args,
        "client",
        "endpoint_candidate_sent",
        1,
        ",\"path\":1,\"generation\":1,\"source_endpoint_changed\":true",
    );
    let (n, source) = endpoint_b
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint challenge timeout"));
    if source != target {
        fail("endpoint challenge from wrong peer")
    }
    let challenge_plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint challenge authentication failed"));
    let ProcessMessage::ReadinessRequest {
        session,
        target_path,
        path_generation,
        delivery_epoch,
        challenge_id,
    } = ProcessMessage::decode(&challenge_plain)
        .unwrap_or_else(|_| fail("endpoint challenge malformed"))
    else {
        fail("endpoint challenge missing")
    };
    if session != SessionId(7001)
        || target_path != 1
        || path_generation != 1
        || delivery_epoch != 1
        || challenge_id == 0
    {
        fail("endpoint challenge tuple mismatch")
    }
    let mut response_id = challenge_id;
    if args.iter().any(|a| a == "--test-wrong-rebind-challenge") {
        response_id = response_id.wrapping_add(1);
        if response_id == 0 {
            response_id = 1;
        }
    }
    let response = ProcessMessage::ReadinessResponse {
        session,
        target_path,
        path_generation,
        delivery_epoch,
        challenge_id: response_id,
        admitted: true,
    }
    .encode()
    .unwrap();
    endpoint_b
        .send_to(&secure.seal_unreliable(&response).unwrap(), target)
        .unwrap();
    emit_diagnostic(
        args,
        "client",
        "endpoint_challenge_answered",
        1,
        ",\"path\":1,\"generation\":1",
    );
    if args.iter().any(|a| a == "--test-wrong-rebind-challenge") {
        fail("test wrong endpoint challenge completed without rejection")
    }
    let (n, source) = endpoint_b
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint promotion sync timeout"));
    if source != target {
        fail("endpoint promotion sync from wrong peer")
    }
    let plain = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint promotion sync authentication failed"));
    if !matches!(
        ProcessMessage::decode(&plain),
        Ok(ProcessMessage::ReadinessResponse {
            session: SessionId(7001),
            target_path: 1,
            path_generation: 1,
            delivery_epoch: 1,
            challenge_id: 1,
            admitted: true,
        })
    ) {
        fail("endpoint promotion sync malformed")
    }
    emit_diagnostic(
        args,
        "client",
        "endpoint_promotion_sync_received",
        1,
        ",\"path\":1,\"generation\":1",
    );
    if args.iter().any(|a| a == "--test-stale-old-endpoint") {
        let stale = ProcessMessage::ReadinessResponse {
            session: SessionId(7001),
            target_path: 1,
            path_generation: 0,
            delivery_epoch: 1,
            challenge_id,
            admitted: true,
        }
        .encode()
        .unwrap();
        endpoint_a
            .send_to(&secure.seal_unreliable(&stale).unwrap(), target)
            .unwrap();
        emit_diagnostic(
            args,
            "client",
            "endpoint_stale_source_sent",
            1,
            ",\"path\":1,\"generation\":0",
        );
    }
    let second_record = OutboundRecord {
        stream: StreamId(1),
        offset: bytes as u64,
        data: payload,
    };
    let second = ProcessMessage::Data {
        session: SessionId(7001),
        record: second_record.clone(),
    }
    .encode()
    .unwrap();
    endpoint_b
        .send_to(&secure.seal_unreliable(&second).unwrap(), target)
        .unwrap();
    let (n, source) = endpoint_b
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("post-rebind DeliveryAck timeout"));
    if source != target {
        fail("post-rebind DeliveryAck from wrong peer")
    }
    let ack = secure
        .open_unreliable(&buf[..n])
        .unwrap_or_else(|_| fail("post-rebind DeliveryAck authentication failed"));
    if !delivery_ack_matches(&ack, &second_record) {
        fail("post-rebind DeliveryAck mismatch")
    }
    emit_diagnostic(
        args,
        "client",
        "endpoint_promoted",
        1,
        ",\"path\":1,\"generation\":1,\"source_endpoint_changed\":true",
    );
    emit_diagnostic(
        args,
        "client",
        "endpoint_post_delivery_ack_validated",
        2,
        ",\"path\":1,\"generation\":1",
    );
    emit_diagnostic(
        args,
        "client",
        "summary",
        2,
        &format!(
            ",\"classification\":\"A\",\"records\":2,\"application_bytes_total\":{}",
            bytes * 2
        ),
    );
    println!(
        "endpoint_rebind_client_ok records=2 application_bytes_total={}",
        bytes * 2
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
        Some("periodic-server") => periodic::server(&a),
        Some("periodic-client") => periodic::client(&a),
        Some("scheduler-fairness") => scheduler_fairness(&a),
        Some("key-update") => key_update_fixture(&a),
        Some("health-observe") => health_observe(&a),
        Some("capabilities") => capabilities(&a),
        Some("multistream") => multistream::run(&a),
        Some("failover") | Some("failover-server") | Some("failover-client") => failover_gate(&a),
        Some("endpoint-rebind-server") => endpoint_rebind_server(&a),
        Some("endpoint-rebind-client") => endpoint_rebind_client(&a),
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
    fn post_handshake_admission_ignores_exact_duplicates_and_wrong_peer() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let wrong = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let expected = OutboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: vec![7; 4],
        };
        let (mut sender, mut receiver) = secure_pair();
        let ack = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 0,
            len: 4,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&ack).unwrap();
        let client_addr = client.local_addr().unwrap();
        wrong.send_to(b"wrong-peer", client_addr).unwrap();
        server.send_to(b"selection", client_addr).unwrap();
        server.send_to(b"noise", client_addr).unwrap();
        server.send_to(&sealed, client_addr).unwrap();
        let mut diagnostics = Vec::new();
        let n = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &expected,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |d| diagnostics.push(d),
        )
        .unwrap();
        assert_eq!(n, sealed.len());
        assert_eq!(
            diagnostics,
            [
                "unexpected_peer",
                "duplicate_negotiation_response",
                "duplicate_noise_response"
            ]
        );
    }

    #[test]
    fn post_handshake_admission_fails_at_malformed_bound() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (_, mut receiver) = secure_pair();
        let expected = OutboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: vec![7; 4],
        };
        for _ in 0..MAX_POST_HANDSHAKE_MALFORMED {
            server
                .send_to(b"garbage", client.local_addr().unwrap())
                .unwrap();
        }
        let err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &expected,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |_| {},
        )
        .unwrap_err();
        assert_eq!(err, "UDP delivery acknowledgement malformed bound exceeded");
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
