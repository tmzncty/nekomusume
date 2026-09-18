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
    thread,
    time::{Duration, Instant},
};
const USAGE: &str = "Usage: neko <server|client|probe|health-observe|failover|multistream|scheduler-fairness|key-update|periodic-server|periodic-client|lab|workload|endpoint-rebind-server|endpoint-rebind-client|keygen|capabilities> [bounded options]

  probe --matrix --target LOOPBACK:PORT --transport tcp|udp --ip-version ipv4|ipv6 [--timeout-ms 1-5000] [--bytes 1-1200] [--json]: local-loopback only
  lab --scenario reliable-udp [--rounds 4-64] [--drop-every 0|2-16] [--drop-ack-every 0-16] [--settle-ms 1-20000] [--json]: bounded loopback reliable-UDP fixture (controlled suppression is not natural loss; manager decision only, no TCP socket; not production)
  --count N: bounded authenticated exchanges (1-64; periodic 1-600)\n  periodic-*: one TCP Session, duration 1-600s, interval 100-5000ms, <=1MiB app data; reconnect unsupported\n  failover --role server|client: canonical bounded failover command\n  failover-server|failover-client: legacy aliases for failover\n  capabilities [--json]: secret-free build, command, default, and limit report\n\nBounded authenticated research probe only; no proxy/tunnel behavior.\n";
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
        println!(
            "fixture_scenarios command=lab scenario=reliable-udp maturity=fixture scope=bounded-loopback-controlled-suppression-manager-decision-only-no-tcp-socket"
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
// H-R9-048/049: project the complete typed `acked_packets` retirement set
// into per-packet positive evidence fields — one entry per retired packet,
// never a representative-only `.last()` collapse.
fn carrier_retired_fields(acked_packets: &[u64]) -> Vec<String> {
    acked_packets
        .iter()
        .map(|pn| {
            format!(
                ",\"applied\":true,\"packet_number\":{},\"retired\":true",
                pn
            )
        })
        .collect()
}
// H-R9-050: the single retransmit admission gate used by the executable
// callers — congestion admission and Recovery ownership accounting BOTH use
// the exact encoded/sealed wire byte count, never the retained plaintext
// length. Returns true only when the caller may send the datagram.
fn admit_retransmit(
    rt: &mut neko_carrier::ReliableUdpRuntime,
    pn: u64,
    now_us: u64,
    wire: &[u8],
    frame: neko_reliable::FrameId,
) -> bool {
    if !rt.can_send(wire.len() as u64) {
        return false;
    }
    rt.on_retransmit_sent(pn, now_us, wire.len() as u64, frame)
        .is_ok()
}
// H-R9-055: the single executable PTO-due decision owner — only fires the
// mutating probe when `now_us >= next_pto_deadline_us`. Returns the deadline
// and the probe frames; yields None (zero transition) before the deadline.
type PtoProbes = Vec<(neko_reliable::FrameId, Vec<u8>)>;
fn due_pto_probe(
    rt: &mut neko_carrier::ReliableUdpRuntime,
    now_us: u64,
) -> Option<(u64, PtoProbes)> {
    let deadline = rt.recovery_engine().next_pto_deadline_us(1_000, 0)?;
    if now_us < deadline {
        return None;
    }
    Some((deadline, rt.pto_probe()))
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
pub(crate) fn read_identity(path: &PathBuf) -> Option<LocalIdentity> {
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
/// Outcome of one authenticated datagram classified by the reliable-mode receive
/// owner. Every successfully authenticated plaintext is classified exactly once
/// and never silently discarded.
#[derive(Debug)]
enum UdpAcknowledgement {
    /// A Session `DeliveryAck` matched one outstanding reliable-owned logical
    /// record. The caller applies exactly this record's confirmation once.
    Session {
        record: OutboundRecord,
        bytes: usize,
    },
    /// A canonical Carrier packet ACK was classified; `applied` is the typed
    /// `apply_ack` outcome. A rejection never mutated recovery.
    Carrier {
        /// `true` iff the ACK retired at least one packet — not merely
        /// accepted-empty (stale/duplicate) or rejected.
        applied: bool,
        /// Packets actually retired by this ACK (empty for stale/duplicate).
        acked_packets: Vec<u64>,
        /// `true` iff apply_ack returned Err (e.g. future/never-sent ACK).
        rejected: bool,
    },
}

#[allow(clippy::too_many_arguments)]
fn recv_udp_delivery_ack(
    socket: &UdpSocket,
    peer: SocketAddr,
    secure: &mut neko_crypto::SecureSession,
    // R9-2E: the bounded set of outstanding reliable-owned logical records. An
    // authenticated Session DeliveryAck matching any of them is consumed in
    // arrival order, so a later record's confirmation is never swallowed while
    // an earlier one is still awaited.
    outstanding: &mut Vec<OutboundRecord>,
    negotiation_response: &[u8],
    noise_response: &[u8],
    deadline: Instant,
    diagnostic: &mut dyn FnMut(&'static str),
    // R9: optional reliable-UDP runtime. Authenticated canonical Carrier packet
    // ACKs are applied here and reported as a typed outcome.
    mut rt: Option<&mut neko_carrier::ReliableUdpRuntime>,
    // H-R9-006: the malformed/ignored budget is owned by the whole reliable
    // receive/settlement operation, not one helper invocation — caller keeps a
    // persistent count across every demux return (Session ACK or Carrier ACK).
    malformed: &mut usize,
    // R9-3: monotonic recovery clock origin for the cross-process reliable
    // operation. Sampled after authenticated Carrier ACK receipt, before
    // apply_ack — never a stale pre-receive scalar.
    recovery_epoch: Instant,
    // H-R9-056/058/059: operation-owned exact ACK witness — the bounded set of
    // (stream,offset,len) already exact-matched by this operation. A fresh
    // exact duplicate for one of them is accepted-empty; any other unmatched
    // ACK is the bounded unexpected negative. Cardinality derives from the
    // operation's admitted records, never Session lifetime.
    confirmed_acks: &mut std::collections::BTreeSet<(u64, u64, u64)>,
) -> Result<UdpAcknowledgement, &'static str> {
    let mut buf = [0u8; 65536];
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
                Ok(plain) => {
                    // Domain 1 — Session DeliveryAck for an outstanding logical
                    // record. Order-independent: whichever record's confirmation
                    // arrives first is consumed as a real match.
                    if let Ok(ProcessMessage::DeliveryAck {
                        session,
                        stream,
                        offset,
                        len,
                    }) = ProcessMessage::decode(&plain)
                        && session == SessionId(7001)
                    {
                        if let Some(pos) = outstanding.iter().position(|r| {
                            r.stream == stream && r.offset == offset && r.data.len() == len
                        }) {
                            let record = outstanding.remove(pos);
                            // Record the exact admitted range so a later exact
                            // duplicate ACK is classification-only.
                            confirmed_acks.insert((stream.0, offset, len as u64));
                            return Ok(UdpAcknowledgement::Session { record, bytes: n });
                        }
                        // H-R9-056/058/059: a freshly sealed EXACT duplicate
                        // for a range this operation already exact-matched is
                        // accepted-empty — idempotent zero-delta feedback, not
                        // malformed. Any other unmatched ACK is fail-closed.
                        if confirmed_acks.contains(&(stream.0, offset, len as u64)) {
                            diagnostic("accepted_empty_logical_ack");
                            continue;
                        }
                        // Authenticated but unadmitted logical ACK: a typed,
                        // bounded negative, never a match and never an
                        // unbounded wait.
                        diagnostic("unexpected_logical_ack");
                        *malformed += 1;
                        if *malformed >= MAX_POST_HANDSHAKE_MALFORMED {
                            return Err("UDP delivery acknowledgement malformed bound exceeded");
                        }
                        continue;
                    }
                    // Domain 2 — canonical Carrier packet ACK. Applied and
                    // rejected outcomes are both typed and observable.
                    if rt.is_some()
                        && let Ok(rec) = neko_wire::decode(&plain)
                        && rec.record_type == neko_wire::RecordType::Ack
                        && let Ok(ack) = neko_wire::decode_ack(&rec.payload)
                        && let Ok(ranges) = neko_reliable::AckRanges::from_ranges(
                            32,
                            &ack.ranges
                                .iter()
                                .map(|w| neko_reliable::AckRange {
                                    start: w.start,
                                    end: w.end,
                                })
                                .collect::<Vec<_>>(),
                        )
                    {
                        // H-R9-025: an OK result only means the canonical ACK
                        // was accepted — a stale/duplicate ACK may retire
                        // nothing. Surface the actual acked_packets so the
                        // caller can prove a real packet transition occurred.
                        // R9-3 ACK observation time: sample the monotonic
                        // recovery clock AFTER the authenticated ACK was
                        // received/decrypted/decoded — not before the blocking
                        // recv, so RTT/PTO math sees the true observation time.
                        let now_us = recovery_epoch.elapsed().as_micros() as u64;
                        let outcome = rt
                            .as_deref_mut()
                            .map(|r| r.apply_ack(&ranges, now_us, ack.ack_delay_us));
                        let (applied, acked_packets, rejected) = match outcome {
                            Some(Ok(o)) => (!o.acked_packets.is_empty(), o.acked_packets, false),
                            Some(Err(_)) => (false, Vec::new(), true),
                            None => (false, Vec::new(), false),
                        };
                        return Ok(UdpAcknowledgement::Carrier {
                            applied,
                            acked_packets,
                            rejected,
                        });
                    }
                    // Domain 3 — any other authenticated plaintext consumes the
                    // finite malformed budget in reliable mode too, so a peer
                    // cannot make this owner spin unboundedly.
                    *malformed += 1;
                    diagnostic("malformed_or_unadmitted");
                    if *malformed >= MAX_POST_HANDSHAKE_MALFORMED {
                        return Err("UDP delivery acknowledgement malformed bound exceeded");
                    }
                }
                Err(_) => {
                    *malformed += 1;
                    diagnostic("malformed_or_unadmitted");
                    if *malformed >= MAX_POST_HANDSHAKE_MALFORMED {
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
    let mut handshake_admission: Option<preauth::AdmissionTicket> = None;
    let mut delayed_noise_duplicate_sent = false;
    let mut guard = None;
    let mut app = Vec::new();
    let mut migration_back_complete = false;
    // R9-4: --delay-r9-ack-reorder withholds the FIRST post-return Carrier ACK
    // until a second post packet arrives, then releases the delayed original
    // before the fresh copy's ACK — a deterministic ACK-loss + reorder seam.
    let mut delayed_post_ack: Option<(Vec<u8>, u64)> = None;
    // H-R9-057: the delay/reorder challenge is exactly one-shot — after the
    // first withheld ACK is released, later post packets ACK normally.
    let mut delay_reorder_done = false;
    let udp_local_port = udp.local_addr().map(|a| a.port()).unwrap_or(up);
    let tcp_local_port = tcp.local_addr().map(|a| a.port()).unwrap_or(tp);
    let mut runtime =
        SessionRuntime::new(SessionId(7001), runtime_limits(bytes, count), 0).unwrap();
    // R9: bounded receiver-side reliable-UDP runtime for packet ACK tracking.
    // Session delivery/dedup stays in `runtime` (SessionRuntime) above Carrier.
    let mut server_rt = neko_carrier::ReliableUdpRuntime::new(1, 1200)
        .unwrap_or_else(|_| fail("server reliable udp runtime"));
    // M-R9-008: buffered first-record Session DeliveryAck for the
    // reversed-ack-order fault seam (released after a later record's ACK).
    let mut pending_reversed_ack: Option<Vec<u8>> = None;
    // P3: deferred Carrier packet ACK for --malformed-budget-test ordering.
    let mut pending_p3_carrier_ack: Option<Vec<u8>> = None;
    // H-R9-028: one-shot guards for the initial stale/future injection seams —
    // each fires exactly once per bounded operation, not once per packet ACK.
    let mut stale_ack_sent = false;
    let mut future_ack_sent = false;
    // H-R9-034: one-shot guard for the settlement-phase stale seam — the
    // duplicate is sent after the last reliable record's Session ACK so the
    // client's settlement continuation (not the initial loop) consumes it.
    // Retains the last reliable record's canonical ACK plaintext for re-seal.
    let mut stale_ack_late_sent = false;
    let mut stale_late_pack: Option<Vec<u8>> = None;
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
        if handshake_admission
            .as_ref()
            .is_some_and(|admission| admission.was_expired(&expired_states))
        {
            handshake_admission = None;
            handshake_cache = None;
            secure = None;
            guard = None;
            emit_diagnostic(
                args,
                "server",
                "udp_preprogress_expired",
                0,
                ",\"secure_retired\":true,\"resume_guard_retired\":true",
            );
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
                        preauth
                            .suppress_charged_response(response_permit)
                            .unwrap_or_else(|_| fail("pre-auth response suppression failed"));
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
                    handshake_admission = Some(authenticated.admission);
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
                            preauth
                                .suppress_charged_response(response_permit)
                                .unwrap_or_else(|_| fail("pre-auth response suppression failed"));
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
                if let Some(admission) = handshake_admission.as_mut() {
                    preauth
                        .charge_input(admission, n, 4096)
                        .unwrap_or_else(|_| fail("pre-auth retained-owner input rejected"));
                    emit_diagnostic(
                        args,
                        "server",
                        "udp_retained_input_charged",
                        0,
                        &format!(",\"ciphertext_bytes\":{}", n),
                    );
                }
                if let Some((first, response)) = handshake_cache.as_ref() {
                    if buf[..n] == first[..] {
                        let admission = handshake_admission
                            .as_mut()
                            .unwrap_or_else(|| fail("pre-auth cached retry owner missing"));
                        let permit = preauth
                            .charge_response(admission, response.len())
                            .unwrap_or_else(|_| fail("pre-auth cached retry response rejected"));
                        preauth
                            .send_udp_response(&udp, response, peer, permit)
                            .unwrap_or_else(|_| fail("pre-auth cached retry deadline elapsed"));
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
                let reliable_udp = args.iter().any(|a| a == "--reliable-udp");
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
                            // H-R9-003: only a valid authenticated Data packet
                            // that reached Session delivery is recorded in the
                            // recovery/ACK tracker — non-Data/malformed packets
                            // never enter the outgoing R9 ACK range.
                            let pn = u64::from_be_bytes(buf[..8].try_into().unwrap_or([0; 8]));
                            if reliable_udp {
                                server_rt
                                    .on_packet_received(pn, true)
                                    .unwrap_or_else(|_| fail("r9 server on_packet_received"));
                            }
                            let logical = ProcessMessage::DeliveryAck {
                                session,
                                stream,
                                offset,
                                len,
                            }
                            .encode()
                            .unwrap();
                            let ack = ss.seal_unreliable(&logical).unwrap();
                            // M-R9-008 reversed-order seam: with
                            // --reverse-ack-order the server buffers the FIRST
                            // logical record's Session DeliveryAck and only
                            // releases it after answering the SECOND record —
                            // record-1 confirmation arrives before record-0.
                            let reverse_ack_order = args.iter().any(|a| a == "--reverse-ack-order");
                            if !delayed_noise_duplicate_sent
                                && args
                                    .iter()
                                    .any(|a| a == "--delay-noise-duplicate-until-application")
                            {
                                delayed_noise_duplicate_sent = true;
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_noise_response_delayed_duplicate_suppressed",
                                    0,
                                    ",\"reason\":\"authenticated_progress\"",
                                );
                            }
                            if let Some(admission) = handshake_admission.take() {
                                preauth.release(admission);
                            }
                            handshake_cache = None;
                            // R9 packet ACK: authenticated Carrier feedback for
                            // the received packet number, before/separate from
                            // the Session DeliveryAck logical confirmation.
                            let malformed_budget_test =
                                args.iter().any(|a| a == "--malformed-budget-test");
                            if reliable_udp {
                                // Fault seam for the incomplete-settlement
                                // regression: --suppress-r9-ack withholds the
                                // Carrier packet ACK (Session DeliveryAck still
                                // sent) so the client cannot retire in-flight.
                                let suppress_ack = args.iter().any(|a| a == "--suppress-r9-ack");
                                if !suppress_ack
                                    && let Some(ranges) = server_rt.poll_outgoing_ack(0)
                                {
                                    let pack = neko_wire::encode(&neko_wire::Record {
                                        record_type: neko_wire::RecordType::Ack,
                                        flags: 0,
                                        payload: neko_wire::encode_ack(&ranges).unwrap(),
                                    })
                                    .unwrap();
                                    if let Ok(sealed_ack) = ss.seal_unreliable(&pack) {
                                        // H-R9-026/027 regression seams.
                                        // --send-stale-ack re-seals the same
                                        // canonical ACK plaintext under a
                                        // FRESH authenticated envelope so
                                        // crypto accepts it and Recovery sees
                                        // a stale/duplicate (accepted-empty).
                                        // H-R9-034: retain the FIRST reliable
                                        // record's canonical ACK plaintext so
                                        // the settlement-phase seam can re-seal
                                        // it as a stale duplicate after the
                                        // last reliable record's Session ACK.
                                        if args.iter().any(|a| a == "--send-stale-ack-late")
                                            && offset == 0
                                        {
                                            stale_late_pack = Some(pack.clone());
                                        }
                                        // H-R9-028: one-shot only — exactly one
                                        // fresh-envelope duplicate per operation.
                                        if !stale_ack_sent
                                            && args.iter().any(|a| a == "--send-stale-ack")
                                        {
                                            if let Ok(resealed) = ss.seal_unreliable(&pack) {
                                                let _ = udp.send_to(&resealed, peer);
                                                stale_ack_sent = true;
                                            }
                                        }
                                        // --send-future-ack sends a canonical
                                        // ACK whose largest_observed exceeds
                                        // the client's largest_sent — Recovery
                                        // rejects it before any mutation.
                                        // H-R9-028: one-shot only.
                                        if !future_ack_sent
                                            && args.iter().any(|a| a == "--send-future-ack")
                                        {
                                            let future = neko_wire::encode(&neko_wire::Record {
                                                record_type: neko_wire::RecordType::Ack,
                                                flags: 0,
                                                payload: neko_wire::encode_ack(
                                                    &neko_wire::AckPayload {
                                                        largest_observed: u64::MAX - 1,
                                                        ranges: vec![neko_wire::AckRangeWire {
                                                            start: u64::MAX - 1,
                                                            end: u64::MAX - 1,
                                                        }],
                                                        ack_delay_us: 0,
                                                    },
                                                )
                                                .unwrap_or_default(),
                                            })
                                            .unwrap();
                                            if let Ok(sealed_f) = ss.seal_unreliable(&future) {
                                                let _ = udp.send_to(&sealed_f, peer);
                                                future_ack_sent = true;
                                            }
                                        }
                                        if malformed_budget_test {
                                            // P3 seam: defer the Carrier ACK
                                            // until after malformed #1/#2 so
                                            // observed order is malformed,
                                            // malformed, Carrier ACK,
                                            // malformed.
                                            pending_p3_carrier_ack = Some(sealed_ack);
                                        } else {
                                            let _ = udp.send_to(&sealed_ack, peer);
                                            emit_diagnostic(
                                                args,
                                                "server",
                                                "udp_packet_ack_sent",
                                                0,
                                                "",
                                            );
                                        }
                                    }
                                }
                            }
                            if reverse_ack_order && offset == 0 {
                                // Buffer the first record's ACK; emit it only
                                // after a later record's ACK — reversed order.
                                pending_reversed_ack = Some(ack.clone());
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_delivery_ack_deferred",
                                    0,
                                    ",\"offset\":0",
                                );
                            } else if cease_udp_replies_after
                                .is_none_or(|point| udp_replies < point)
                            {
                                // M-R9-008 P3: malformed #1 -> malformed #2 ->
                                // Carrier packet ACK -> malformed #3. The
                                // Carrier ACK was deferred above; emit it here
                                // between the malformed pairs so the client's
                                // operation-wide budget must persist across
                                // valid Carrier feedback.
                                if malformed_budget_test {
                                    udp.send_to(b"malformed", peer).unwrap();
                                    udp.send_to(b"malformed", peer).unwrap();
                                    if let Some(pack) = pending_p3_carrier_ack.take() {
                                        let _ = udp.send_to(&pack, peer);
                                        emit_diagnostic(
                                            args,
                                            "server",
                                            "udp_packet_ack_sent",
                                            0,
                                            "",
                                        );
                                    }
                                }
                                udp.send_to(&ack, peer).unwrap();
                                udp_replies += 1;
                                emit_diagnostic(
                                    args,
                                    "server",
                                    "udp_delivery_ack_sent",
                                    offset as usize / bytes.max(1),
                                    &format!(",\"ciphertext_bytes\":{}", ack.len()),
                                );
                                // H-R9-034: --send-stale-ack-late re-seals the
                                // retained record-0 canonical ACK plaintext under
                                // a fresh envelope only after the LAST reliable
                                // record's Session ACK (offset == bytes), so the
                                // duplicate reaches the client's settlement
                                // continuation (not the initial loop).
                                if !stale_ack_late_sent
                                    && offset == bytes.max(1) as u64
                                    && let Some(pack) = stale_late_pack.take()
                                {
                                    if let Ok(resealed) = ss.seal_unreliable(&pack) {
                                        let _ = udp.send_to(&resealed, peer);
                                        stale_ack_late_sent = true;
                                    }
                                }
                                // Now release the deferred first-record ACK —
                                // record-1's confirmation arrives before
                                // record-0's (reversed order).
                                if let Some(deferred) = pending_reversed_ack.take() {
                                    udp.send_to(&deferred, peer).unwrap();
                                    udp_replies += 1;
                                    emit_diagnostic(
                                        args,
                                        "server",
                                        "udp_delivery_ack_sent",
                                        0,
                                        ",\"reversed\":true,\"offset\":0",
                                    );
                                }
                                if malformed_budget_test {
                                    udp.send_to(b"malformed", peer).unwrap();
                                }
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
                // H-R9-016: derive the expected TCP replay count from the same
                // ownership partition the client uses — reliable-UDP-owned
                // records (records[0..2] under --reliable-udp) plus the
                // reserved final record under recovery/migration-back are NOT
                // replayed over TCP. This reconciles the server's expected
                // TCP Data count with the client's actual send count.
                let reliable_udp = args.iter().any(|a| a == "--reliable-udp");
                let uncertain_start = if reliable_udp { 2 } else { 1 };
                let uncertain_end = if recovery_enabled {
                    count.saturating_sub(1)
                } else {
                    count
                };
                let tcp_records = uncertain_end.saturating_sub(uncertain_start);
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
                                &format!(
                                    ",\"ciphertext_bytes\":{},\"stream\":{},\"offset\":{}",
                                    encrypted.len(),
                                    stream_id.0,
                                    offset
                                ),
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
                            let reliable_udp = args.iter().any(|a| a == "--reliable-udp");
                            // R9-3: tolerate PTO retransmission — the client
                            // may suppress the first post-return Data and
                            // retransmit after a bounded PTO delay.
                            let post_deadline = Instant::now() + Duration::from_secs(5);
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
                                // H-R9-015: post-return Data is Carrier-ACK owned — record receipt on the
                                // server recovery owner and emit the canonical packet ACK alongside the
                                // Session DeliveryAck.
                                let mut post_pn: u64 = 0;
                                if reliable_udp {
                                    post_pn = u64::from_be_bytes(
                                        recovery_buf[..8].try_into().unwrap_or([0; 8]),
                                    );
                                    server_rt.on_packet_received(post_pn, true).unwrap_or_else(
                                        |_| fail("r9 post-return on_packet_received"),
                                    );
                                }
                                // H-R9-030 post-return seams: the pending ACK
                                // obligation created by on_packet_received is
                                // consumptive — poll it exactly once here and
                                // retain the canonical plaintext. The stale
                                // seam re-seals that retained plaintext (true
                                // semantic duplicate); the future seam sends a
                                // canonical never-sent ACK before the real one.
                                // Under a seam the deterministic order is
                                // future -> real -> duplicate -> DeliveryAck;
                                // without a seam the ordinary P2 order
                                // (DeliveryAck first, then Carrier ACK) is
                                // unchanged.
                                let seam_stale = args.iter().any(|a| a == "--send-stale-ack");
                                let seam_future = args.iter().any(|a| a == "--send-future-ack");
                                // READY_LOCAL 2 order-only seam: Carrier ACK
                                // before Session DeliveryAck with no injected
                                // stale/future feedback.
                                let seam_reverse =
                                    args.iter().any(|a| a == "--reverse-post-return-ack");
                                let seam_active = seam_stale || seam_future || seam_reverse;
                                let post_ack_pack = if reliable_udp {
                                    server_rt.poll_outgoing_ack(0).map(|ranges| {
                                        neko_wire::encode(&neko_wire::Record {
                                            record_type: neko_wire::RecordType::Ack,
                                            flags: 0,
                                            payload: neko_wire::encode_ack(&ranges).unwrap(),
                                        })
                                        .unwrap()
                                    })
                                } else {
                                    None
                                };
                                if reliable_udp && seam_active {
                                    // Future/never-sent injection arrives while
                                    // the client receive owner is still active,
                                    // before the legitimate current ACK.
                                    if seam_future {
                                        let future = neko_wire::encode(&neko_wire::Record {
                                            record_type: neko_wire::RecordType::Ack,
                                            flags: 0,
                                            payload: neko_wire::encode_ack(
                                                &neko_wire::AckPayload {
                                                    largest_observed: u64::MAX - 1,
                                                    ranges: vec![neko_wire::AckRangeWire {
                                                        start: u64::MAX - 1,
                                                        end: u64::MAX - 1,
                                                    }],
                                                    ack_delay_us: 0,
                                                },
                                            )
                                            .unwrap_or_default(),
                                        })
                                        .unwrap();
                                        if let Ok(sealed_f) = udp_session.seal_unreliable(&future) {
                                            let _ = udp.send_to(&sealed_f, post_source);
                                        }
                                    }
                                    // Legitimate current Carrier ACK first.
                                    if let Some(pack) = &post_ack_pack {
                                        if let Ok(sealed_pack) = udp_session.seal_unreliable(pack) {
                                            let _ = udp.send_to(&sealed_pack, post_source);
                                            emit_diagnostic(
                                                args,
                                                "server",
                                                "udp_return_packet_ack_sent",
                                                0,
                                                &format!(",\"packet_number\":{}", post_pn),
                                            );
                                        }
                                        // Stale case: re-seal the SAME retained
                                        // ACK plaintext under a second fresh
                                        // envelope — a true semantic duplicate
                                        // that must classify accepted-empty.
                                        if seam_stale
                                            && let Ok(resealed) = udp_session.seal_unreliable(pack)
                                        {
                                            let _ = udp.send_to(&resealed, post_source);
                                        }
                                    }
                                }
                                // P4 fault seam: --suppress-r9-dack withholds the post-return Session
                                // DeliveryAck; the Carrier packet ACK is still sent.
                                let suppress_dack = args.iter().any(|a| a == "--suppress-r9-dack");
                                if !suppress_dack {
                                    let sealed_ack = udp_session.seal_unreliable(&ack).unwrap();
                                    udp.send_to(&sealed_ack, post_source).unwrap();
                                    emit_diagnostic(
                                        args,
                                        "server",
                                        "udp_return_delivery_ack_sent",
                                        post_offset as usize / bytes.max(1),
                                        &format!(
                                            ",\"stream\":{},\"offset\":{},\"len\":{}",
                                            post_stream.0, post_offset, post_len
                                        ),
                                    );
                                }
                                // READY_LOCAL 1 P4 seam: suppress ONLY the
                                // post-return Carrier packet ACK — Session
                                // DeliveryAck is still sent. The pending ACK
                                // obligation is consumed but not sent.
                                let suppress_pack =
                                    args.iter().any(|a| a == "--suppress-r9-post-return-pack");
                                let delay_reorder =
                                    args.iter().any(|a| a == "--delay-r9-ack-reorder");
                                // R9-4: with --delay-r9-ack-reorder the FIRST
                                // post-return Carrier ACK is withheld; on the
                                // next post packet the delayed original is
                                // released BEFORE the fresh copy's ACK —
                                // deterministic ACK-loss + delayed/reorder.
                                if reliable_udp && delay_reorder && !delay_reorder_done {
                                    if delayed_post_ack.is_none() {
                                        if let Some(pack) = &post_ack_pack {
                                            if let Ok(sealed_pack) =
                                                udp_session.seal_unreliable(pack)
                                            {
                                                delayed_post_ack = Some((sealed_pack, post_pn));
                                            }
                                        }
                                    } else {
                                        delay_reorder_done = true;
                                        let (delayed, dpn) = delayed_post_ack.take().unwrap();
                                        let _ = udp.send_to(&delayed, post_source);
                                        emit_diagnostic(
                                            args,
                                            "server",
                                            "udp_return_packet_ack_sent",
                                            0,
                                            &format!(",\"packet_number\":{}", dpn),
                                        );
                                        if let Some(pack) = &post_ack_pack {
                                            if let Ok(sealed_pack) =
                                                udp_session.seal_unreliable(pack)
                                            {
                                                let _ = udp.send_to(&sealed_pack, post_source);
                                                emit_diagnostic(
                                                    args,
                                                    "server",
                                                    "udp_return_packet_ack_sent",
                                                    0,
                                                    &format!(",\"packet_number\":{}", post_pn),
                                                );
                                            }
                                        }
                                    }
                                } else if reliable_udp && !seam_active && !suppress_pack {
                                    if let Some(pack) = &post_ack_pack {
                                        if let Ok(sealed_pack) = udp_session.seal_unreliable(pack) {
                                            let _ = udp.send_to(&sealed_pack, post_source);
                                            emit_diagnostic(
                                                args,
                                                "server",
                                                "udp_return_packet_ack_sent",
                                                0,
                                                &format!(",\"packet_number\":{}", post_pn),
                                            );
                                        }
                                    }
                                }
                                if let Some(delivered) = runtime.pop_receive(5).unwrap() {
                                    app.extend_from_slice(&delivered.data);
                                }
                                migration_back_complete = true;
                                // R9-3: keep receiving — a suppressed original
                                // may be retransmitted after PTO; the server
                                // must still ACK the fresh copy.
                                continue;
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
    let first_data_delay_ms = parse(args, "--test-first-data-delay-ms", Some("0"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid first data delay"));
    if first_data_delay_ms > 2000 {
        fail("first data delay outside 0-2000 ms")
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
    if first_data_delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(first_data_delay_ms));
    }
    // R9 reliable-UDP mode: the post-auth application transport runs the
    // committed ReliableUdpRuntime — cwnd admission, packet->stable-FrameId
    // recording, socket send, authenticated packet ACKs, in-flight settlement.
    // Session DeliveryAck remains a separate logical confirmation path.
    // R9-3: one bounded monotonic relative recovery clock for the entire
    // cross-process reliable operation — initial sends and post-return share
    // the same origin.
    let recovery_epoch = Instant::now();
    let reliable_udp = args.iter().any(|a| a == "--reliable-udp");
    let mut rt = if reliable_udp {
        let mut r = neko_carrier::ReliableUdpRuntime::new(1, 1200)
            .unwrap_or_else(|_| fail("r9 reliable udp runtime"));
        r.ready_standby(0);
        Some(r)
    } else {
        None
    };
    if let Some(rt) = rt.as_mut() {
        // cwnd admission before committing Session byte offset/recovery owner.
        if !rt.can_send(encrypted.len() as u64) {
            fail("r9 cwnd refused initial send");
        }
        let pn = u64::from_be_bytes(encrypted[..8].try_into().unwrap());
        rt.on_packet_sent(
            pn,
            recovery_epoch.elapsed().as_micros() as u64,
            encrypted.len() as u64,
            neko_reliable::FrameId(udp_record.offset),
            &udp_record.data,
        )
        .unwrap_or_else(|_| fail("r9 record send"));
        // H-R9-061: first-send socket-outcome transaction — positive sent
        // evidence is committed only on real socket success; a socket error
        // rolls back exactly that packet's Recovery ownership.
        match u.send_to(&encrypted, target) {
            Ok(_) => {
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
            }
            Err(_) => {
                rt.abandon_sent(pn);
                emit_diagnostic(
                    args,
                    "client",
                    "udp_datagram_send_failed",
                    0,
                    &format!(",\"packet_number\":{}", pn),
                );
            }
        }
    } else {
        u.send_to(&encrypted, target).unwrap();
    }
    // R9: under --reliable-udp, a second offered record also rides the
    // reliable-UDP transport (cwnd admission + on_packet_sent + send) to prove
    // multi-record once-delivery; the remaining records keep their existing
    // uncertain/failover ownership so the no-loss path stays coherent.
    if let Some(rt) = rt.as_mut()
        && let Some(rec) = records.get(1)
    {
        let l2 = ProcessMessage::Data {
            session: SessionId(7001),
            record: rec.clone(),
        }
        .encode()
        .unwrap();
        let e2 = us.seal_unreliable(&l2).unwrap();
        if !rt.can_send(e2.len() as u64) {
            fail("r9 cwnd refused second send");
        }
        let pn = u64::from_be_bytes(e2[..8].try_into().unwrap());
        rt.on_packet_sent(
            pn,
            recovery_epoch.elapsed().as_micros() as u64,
            e2.len() as u64,
            neko_reliable::FrameId(rec.offset),
            &rec.data,
        )
        .unwrap_or_else(|_| fail("r9 record send"));
        // H-R9-061: first-send socket-outcome transaction — positive sent
        // evidence only on real socket success; a socket error rolls back
        // exactly that packet's Recovery ownership.
        match u.send_to(&e2, target) {
            Ok(_) => {
                emit_diagnostic(
                    args,
                    "client",
                    "r9_udp_record_sent",
                    0,
                    &format!(",\"offset\":{}", rec.offset),
                );
            }
            Err(_) => {
                rt.abandon_sent(pn);
                emit_diagnostic(
                    args,
                    "client",
                    "r9_udp_record_send_failed",
                    0,
                    &format!(",\"packet_number\":{}", pn),
                );
            }
        }
    }
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
    // R9-2E: one bounded authenticated receive/demux owner. Every authenticated
    // plaintext is classified exactly once — Session DeliveryAck for any
    // outstanding logical record (order-independent), canonical Carrier packet
    // ACK (typed applied/rejected), or a bounded malformed negative. Under
    // --reliable-udp both reliable-owned records are awaited together, so a
    // record-1 confirmation arriving before record-0 is consumed, not swallowed.
    let mut outstanding = vec![udp_record.clone()];
    if reliable_udp && let Some(rec1) = records.get(1) {
        outstanding.push(rec1.clone());
    }
    let mut logical_confirmations = 0u64;
    let mut packet_ack_applied = 0u64;
    let mut packet_ack_rejected = 0u64;
    // H-R9-006: persistent malformed budget across every demux return.
    let mut malformed = 0usize;
    // H-R9-011: pending logical-ACK buffer — a later record's exact ACK is
    // buffered until the Session confirmed watermark reaches its offset, so it
    // can never manufacture an earlier range's confirmation.
    let mut pending_acks: Vec<OutboundRecord> = Vec::new();
    // H-R9-059: operation-owned exact ACK witness — bounded by this operation's
    // admitted outstanding records, cleared at operation completion.
    let mut confirmed_acks = std::collections::BTreeSet::new();
    while !outstanding.is_empty() {
        let outcome = recv_udp_delivery_ack(
            &u,
            target,
            &mut us,
            &mut outstanding,
            &negotiation_response,
            &noise_response,
            application_deadline,
            &mut admission_diagnostic,
            rt.as_mut(),
            &mut malformed,
            recovery_epoch,
            &mut confirmed_acks,
        )
        .unwrap_or_else(|e| fail(&format!("UDP delivery acknowledgement failed: {e:?}")));
        match outcome {
            UdpAcknowledgement::Session { record, bytes } => {
                // H-R9-011: only the exact pending ACK whose offset equals the
                // Session confirmed watermark may advance delivery. A later
                // record's ACK is buffered, never allowed to over-promote an
                // earlier unconfirmed range via the cumulative watermark.
                let wm = delivery.confirmed_watermark(record.stream);
                if record.offset == wm {
                    delivery
                        .delivery_ack(
                            record.stream,
                            record.offset,
                            record.data.len(),
                            count as u64 + 3,
                        )
                        .unwrap_or_else(|e| fail(&format!("r9 delivery_ack failed: {e:?}")));
                    logical_confirmations += 1;
                    // H-R9-012: emit the current ACK's validated diagnostic
                    // IMMEDIATELY after its Session application — before any
                    // buffered pending drain — so structured evidence order
                    // matches the actual mutation order.
                    if logical_confirmations == 1 {
                        // Legacy event retained for compatibility.
                        emit_diagnostic(
                            args,
                            "client",
                            "udp_delivery_ack_validated",
                            1,
                            &format!(",\"ciphertext_bytes\":{bytes}"),
                        );
                    }
                    // H-R9-013: EVERY reliable Session mutation — including the
                    // first — emits an exact offset-bearing applied event so
                    // process evidence proves which logical range confirmed.
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_delivery_ack_validated",
                        1,
                        &format!(
                            ",\"ciphertext_bytes\":{},\"stream\":{},\"offset\":{}",
                            bytes, record.stream.0, record.offset
                        ),
                    );
                    // Drain any buffered pending ACKs that are now in-order —
                    // each emits its applied event in actual mutation order.
                    while let Some(pos) = pending_acks
                        .iter()
                        .position(|p| p.offset == delivery.confirmed_watermark(p.stream))
                    {
                        let rec = pending_acks.remove(pos);
                        delivery
                            .delivery_ack(rec.stream, rec.offset, rec.data.len(), count as u64 + 3)
                            .unwrap_or_else(|e| {
                                fail(&format!("r9 pending delivery_ack failed: {e:?}"))
                            });
                        logical_confirmations += 1;
                        emit_diagnostic(
                            args,
                            "client",
                            "r9_udp_delivery_ack_validated",
                            1,
                            &format!(
                                ",\"ciphertext_bytes\":0,\"stream\":{},\"offset\":{},\"buffered\":true",
                                rec.stream.0, rec.offset
                            ),
                        );
                    }
                    continue;
                } else if record.offset > wm {
                    // Out-of-order ACK — buffer it; it cannot confirm an
                    // earlier unconfirmed range.
                    pending_acks.push(record);
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_delivery_ack_buffered",
                        0,
                        &format!(
                            ",\"offset\":{},\"watermark\":{}",
                            pending_acks.last().unwrap().offset,
                            wm
                        ),
                    );
                    continue;
                } else {
                    // offset < wm — duplicate/already-covered ACK; consume as
                    // a typed negative, do not re-apply.
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_delivery_ack_covered",
                        0,
                        &format!(",\"offset\":{},\"watermark\":{}", record.offset, wm),
                    );
                    continue;
                }
            }
            UdpAcknowledgement::Carrier {
                applied, rejected, ..
            } => {
                if applied {
                    packet_ack_applied += 1;
                    // H-R9-022: prove a valid Carrier ACK was actually applied
                    // while the operation-wide malformed counter is at its
                    // current level — the P3 discriminator.
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_packet_ack_applied",
                        0,
                        &format!(",\"malformed\":{malformed}"),
                    );
                } else if rejected {
                    packet_ack_rejected += 1;
                    emit_diagnostic(args, "client", "r9_udp_packet_ack_rejected", 0, "");
                } else {
                    // H-R9-033: classification-only evidence — accepted-empty
                    // stale/duplicate is neither a packet transition nor a
                    // rejection; it increments no state and emits no counter.
                    emit_diagnostic(args, "client", "r9_udp_packet_ack_accepted_empty", 0, "");
                }
            }
        }
    }
    // R9 settlement: drain Carrier packet ACKs through the SAME bounded
    // receive/demux owner (H-R9-007) — no second untyped receive loop.
    // Pre-settlement invariant (M-R9-008): every logical expectation must be
    // retired before Carrier settlement is treated as complete — no pending
    // logical evidence may remain.
    if reliable_udp && !(outstanding.is_empty() && pending_acks.is_empty()) {
        fail("r9 logical confirmation incomplete before settlement");
    }
    if let Some(rt) = rt.as_mut() {
        // M-R9-009: settlement uses the SAME absolute application deadline —
        // no fresh time budget just because Session confirmations landed first.
        let settle_deadline = application_deadline;
        // H-R9-034: after in_flight reaches zero, perform one final bounded
        // drain so a late-arriving stale/duplicate is still classified
        // accepted-empty rather than silently dropped by process truth.
        // Only for the non-migration-back initial path — post-return has its
        // own receive owner and must not have its ACKs consumed here.
        let mut drained_after_zero = migration_back;
        while (rt.in_flight() > 0 || !drained_after_zero) && Instant::now() < settle_deadline {
            match recv_udp_delivery_ack(
                &u,
                target,
                &mut us,
                &mut outstanding,
                &negotiation_response,
                &noise_response,
                settle_deadline,
                &mut admission_diagnostic,
                Some(rt),
                &mut malformed,
                recovery_epoch,
                &mut confirmed_acks,
            ) {
                Ok(UdpAcknowledgement::Carrier {
                    applied, rejected, ..
                }) => {
                    if applied {
                        packet_ack_applied += 1;
                    } else if rejected {
                        packet_ack_rejected += 1;
                        emit_diagnostic(args, "client", "r9_udp_packet_ack_rejected", 0, "");
                    } else {
                        // H-R9-034: settlement continuation must also classify
                        // accepted-empty — the same typed non-event evidence as
                        // the initial loop, so a late duplicate is observable.
                        emit_diagnostic(args, "client", "r9_udp_packet_ack_accepted_empty", 0, "");
                    }
                }
                Ok(UdpAcknowledgement::Session { .. }) => {}
                Err(_) => break, // timeout or bounded malformed bound hit
            }
            if rt.in_flight() == 0 {
                drained_after_zero = true;
            }
        }
        emit_diagnostic(
            args,
            "client",
            "r9_udp_packet_ack_outcomes",
            0,
            &format!(
                ",\"applied\":{},\"rejected\":{},\"remaining_in_flight\":{}",
                packet_ack_applied,
                packet_ack_rejected,
                rt.in_flight()
            ),
        );
        // H-R9-010: only emit the settled marker when in-flight is actually
        // zero. An incomplete settlement is a typed terminal negative — emit
        // it and FAIL the operation so downstream health/failover never
        // consumes a false settlement premise.
        if rt.in_flight() == 0 {
            emit_diagnostic(
                args,
                "client",
                "r9_udp_in_flight_settled",
                0,
                ",\"remaining_in_flight\":0",
            );
        } else {
            emit_diagnostic(
                args,
                "client",
                "r9_udp_settlement_incomplete",
                0,
                &format!(",\"remaining_in_flight\":{}", rt.in_flight()),
            );
            fail("r9 reliable-UDP settlement incomplete");
        }
    }
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
    // H-R9-002 + H-R9-005: under --reliable-udp, records[0]/[1] are reliable-
    // owned, so the legacy uncertain subset starts at index 2. The start index
    // must ALSO respect `uncertain_end` — under migration-back/recovery the
    // final record (index == uncertain_end) is reserved/unassigned and must be
    // neither tracked nor wire-sent before the post-promotion return-to-UDP.
    let uncertain_start = if reliable_udp { 2 } else { 1 };
    let uncertain_count = uncertain_end.saturating_sub(uncertain_start);
    for uncertain in records.iter().skip(uncertain_start).take(uncertain_count) {
        failover
            .track_uncertain(DataId(uncertain.offset), &uncertain.data)
            .unwrap();
    }
    // Direct uncertain send only when the start index is strictly below the
    // reserved `uncertain_end` boundary — a reserved final record is never
    // consumed early.
    if uncertain_start < uncertain_end
        && let Some(uncertain) = records.get(uncertain_start)
    {
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
        // H-R9-017: controlled fallback must use the same ownership partition
        // as the server — reliable-UDP-owned records and the reserved final
        // record are NOT replayed over TCP. Replaying an already reliable-
        // owned/confirmed offset manufactures a BrokenPipe and an ownership
        // contradiction.
        uncertain_end.saturating_sub(uncertain_start)
    };
    let tcp_records = resend_records;
    let mut first_resumed_data_at = None;
    let mut first_resumed_ack_at = None;
    let mut replayed_records = 0usize;
    let mut tcp_active_sample = None;
    // H-R9-020: uncertain record count is the ownership-partition width —
    // uncertain_end - uncertain_start — not records.len()-2. Under
    // --reliable-udp + migration-back the uncertain set is exactly one record.
    let uncertain_records = uncertain_end.saturating_sub(uncertain_start);
    let post_return_record = if recovery_enabled {
        records.last().cloned()
    } else {
        None
    };
    // H-R9-020: track whether the post-return reliable record actually ran so
    // the final accounting counts it once (Option is consumed by unwrap below).
    let mut post_return_ran = false;
    // H-R9-019: TCP replay identity follows the same ownership partition — the
    // replayed records start at `uncertain_start`, not positionally at index 1.
    // Under --reliable-udp the uncertain set begins after the reliable-owned
    // records, so an equal count must not replay an already reliable-owned
    // offset across the Carrier boundary.
    for record in records.into_iter().skip(uncertain_start).take(tcp_records) {
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
            &format!(
                ",\"ciphertext_bytes\":{},\"stream\":{},\"offset\":{}",
                ack.len(),
                record.stream.0,
                record.offset
            ),
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
        // H-R9-015: under --reliable-udp the post-return Data is reliable-owned
        // — congestion admission + on_packet_sent + socket send, and its ACK
        // drain reuses the same bounded demux owner (not a None rt).
        // R9-3: one bounded monotonic relative recovery clock for this
        // post-return operation — sampled at send time, shared by PTO and ACK.
        let post_recovery_epoch = Instant::now();
        let mut post_pn_client: u64 = 0;
        #[allow(unused_assignments)]
        let mut post_sent_at_us: u64 = 0;
        if let Some(rt) = rt.as_mut() {
            if !rt.can_send(sealed.len() as u64) {
                fail("r9 post-return cwnd refused send");
            }
            let pn = u64::from_be_bytes(sealed[..8].try_into().unwrap());
            post_pn_client = pn;
            post_sent_at_us = post_recovery_epoch.elapsed().as_micros() as u64;
            rt.on_packet_sent(
                pn,
                post_sent_at_us,
                sealed.len() as u64,
                neko_reliable::FrameId(post_record.offset),
                &post_record.data,
            )
            .unwrap_or_else(|_| fail("r9 post-return record send"));
        }
        // R9-3: suppress exactly one reliable-owned Data after congestion
        // admission and Recovery ownership commit. The packet was sent to
        // Recovery (on_packet_sent above) but the wire send is dropped — this
        // tests data-loss recovery, not an unowned unsent record.
        let drop_r9_data = args.iter().any(|a| a == "--drop-r9-data");
        if drop_r9_data {
            // H-R9-061: the dropped packet is recovery-owned (positive sent
            // evidence reflects the committed ownership); the wire send is
            // controlled-suppressed, not a socket error.
            emit_diagnostic(
                args,
                "client",
                "r9_udp_post_return_sent",
                0,
                &format!(
                    ",\"stream\":{},\"offset\":{},\"packet_number\":{}",
                    post_record.stream.0, post_record.offset, post_pn_client
                ),
            );
            emit_diagnostic(args, "client", "r9_udp_post_return_data_dropped", 0, "");
        }
        if !drop_r9_data && rt.is_some() {
            // H-R9-061: first-send socket-outcome transaction — the positive
            // sent diagnostic is committed ONLY on real socket success; a
            // socket error rolls back exactly that packet's Recovery ownership
            // and emits no sent evidence.
            // H-R9-062: --fail-r9-first-send injects a socket error on the
            // production first-send owner — the same code path a real Err
            // takes, proving rollback and no positive sent evidence.
            let fail_first_send = args.iter().any(|a| a == "--fail-r9-first-send");
            match if fail_first_send {
                Err(std::io::Error::other("injected first-send failure"))
            } else {
                u.send_to(&sealed, target)
            } {
                Ok(_) => {
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_post_return_sent",
                        0,
                        &format!(
                            ",\"stream\":{},\"offset\":{},\"packet_number\":{}",
                            post_record.stream.0, post_record.offset, post_pn_client
                        ),
                    );
                }
                Err(_) => {
                    if let Some(r) = rt.as_mut() {
                        r.abandon_sent(post_pn_client);
                    }
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_post_return_send_failed",
                        0,
                        &format!(",\"packet_number\":{}", post_pn_client),
                    );
                }
            }
        } else if !drop_r9_data {
            u.send_to(&sealed, target).unwrap();
        }
        // H-R9-015: under --reliable-udp the post-return receive owner waits for
        // BOTH the Session DeliveryAck (logical confirmation) AND the Carrier
        // packet ACK to drain rt.in_flight to zero — a Carrier ACK arriving
        // first is applied, never a protocol error; success requires both
        // logical ownership retired and post-return recovery settled.
        // R9-3: under the data-drop seam the settle deadline must cover at
        // least one PTO fire + retransmission round-trip, not just the
        // ordinary immediate-ACK window.
        let drop_r9_data_settle = args.iter().any(|a| a == "--drop-r9-data");
        let deadline =
            Instant::now() + Duration::from_secs(if drop_r9_data_settle { 8 } else { 2 });
        let mut post_outstanding = vec![post_record.clone()];
        // H-R9-059: post-return operation-owned exact ACK witness.
        let mut post_confirmed_acks = std::collections::BTreeSet::new();
        let mut malformed = 0usize;
        let mut ack_len = 0usize;
        loop {
            let session_done = post_outstanding.is_empty();
            let carrier_done = rt.as_ref().is_none_or(|r| r.in_flight() == 0);
            if session_done && carrier_done {
                break;
            }
            // R9-3: fire PTO/retransmission BEFORE the blocking receive so a
            // due probe is never deferred until the settle deadline. The
            // receive deadline below is then clamped to the next PTO deadline
            // so the loop wakes up to retransmit on time.
            if let Some(rt) = rt.as_mut() {
                let now_us = post_recovery_epoch.elapsed().as_micros() as u64;
                if let Some((pto_deadline, probes)) = due_pto_probe(rt, now_us) {
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_pto_fired",
                        0,
                        &format!(
                            ",\"deadline_us\":{},\"fired_at_us\":{},\"pto_count\":{}",
                            pto_deadline,
                            now_us,
                            rt.recovery_engine().pto_count
                        ),
                    );
                    for (frame, plaintext) in probes {
                        // H-R9-041: congestion admission is checked on the
                        // exact encoded wire bytes, not the plaintext — encode
                        // and seal first, then admit re_sealed.len() before
                        // Recovery ownership commit.
                        let re_msg = ProcessMessage::Data {
                            session: SessionId(7001),
                            record: OutboundRecord {
                                stream: post_record.stream,
                                offset: post_record.offset,
                                data: plaintext.clone(),
                            },
                        }
                        .encode();
                        let Ok(re_msg) = re_msg else { continue };
                        let Ok(re_sealed) = us.seal_unreliable(&re_msg) else {
                            continue;
                        };
                        let rpn = u64::from_be_bytes(re_sealed[..8].try_into().unwrap_or([0u8; 8]));
                        if !admit_retransmit(rt, rpn, now_us, &re_sealed, frame) {
                            continue;
                        }
                        // H-R9-051: socket send is part of the transaction —
                        // a failed send rolls back Recovery ownership for this
                        // packet copy and must not emit r9_udp_retransmit_sent
                        // or advance the active packet identity.
                        if u.send_to(&re_sealed, target).is_err() {
                            let _ = rt.abandon_retransmit(rpn);
                            continue;
                        }
                        emit_diagnostic(
                            args,
                            "client",
                            "r9_udp_retransmit_sent",
                            0,
                            &format!(
                                ",\"packet_number\":{},\"original_packet_number\":{},\"frame\":{}",
                                rpn, post_pn_client, frame.0
                            ),
                        );
                        // R9-3: the active post-return packet identity moves to
                        // the fresh retransmission — its ACK is the positive
                        // retirement that settles this frame's ownership.
                        post_pn_client = rpn;
                    }
                }
            }
            if Instant::now() >= deadline {
                // H-R9-038: classification-only residual-domain evidence —
                // which of the two domains is still outstanding when the
                // bounded post-return settlement fails. Mutates nothing.
                emit_diagnostic(
                    args,
                    "client",
                    "r9_udp_post_return_residual",
                    0,
                    &format!(
                        ",\"session_outstanding\":{},\"remaining_in_flight\":{}",
                        post_outstanding.len(),
                        rt.as_ref().map(|r| r.in_flight()).unwrap_or(0)
                    ),
                );
                fail("post-return reliable-UDP dual settlement timeout");
            }
            // R9-3: clamp the blocking receive deadline to the next PTO
            // deadline so the loop wakes to retransmit instead of sleeping
            // through the entire settle window.
            let recv_deadline = rt
                .as_ref()
                .and_then(|r| r.recovery_engine().next_pto_deadline_us(1_000, 0))
                .map(|d| post_recovery_epoch + Duration::from_micros(d))
                .map(|i| i.min(deadline))
                .unwrap_or(deadline);
            match recv_udp_delivery_ack(
                &u,
                target,
                &mut us,
                &mut post_outstanding,
                &negotiation_response,
                &noise_response,
                recv_deadline,
                &mut |d| {
                    // H-R9-060: the demux's typed classification (accepted-empty
                    // exact duplicate / unexpected logical ACK) is emitted as
                    // observable evidence, not discarded.
                    emit_diagnostic(args, "client", d, 0, "");
                },
                rt.as_mut(),
                &mut malformed,
                post_recovery_epoch,
                &mut post_confirmed_acks,
            ) {
                Ok(UdpAcknowledgement::Session { record, bytes }) => {
                    delivery
                        .delivery_ack(
                            record.stream,
                            record.offset,
                            record.data.len(),
                            count as u64 + 5,
                        )
                        .unwrap();
                    ack_len = bytes;
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_return_delivery_ack",
                        0,
                        &format!(
                            ",\"stream\":{},\"offset\":{},\"len\":{}",
                            record.stream.0,
                            record.offset,
                            record.data.len()
                        ),
                    );
                }
                Ok(UdpAcknowledgement::Carrier {
                    applied,
                    acked_packets,
                    rejected,
                }) => {
                    // H-R9-025/026/029: three distinct Carrier outcome classes.
                    // H-R9-047: a positive ACK that retires ANY packet copy —
                    // including an older sibling under repeated PTO — is a real
                    // retirement, not accepted-empty. `applied` is set from
                    // `acked_packets` non-empty; emit the actually-retired pn.
                    let retired = !acked_packets.is_empty();
                    if applied && retired {
                        // H-R9-048/049: one ACK range may retire several packet
                        // identities — emit positive evidence for EVERY
                        // retired packet so the projection is complete.
                        for fields in carrier_retired_fields(&acked_packets) {
                            emit_diagnostic(args, "client", "r9_udp_return_packet_ack", 0, &fields);
                        }
                    } else if rejected {
                        // Typed rejection (e.g. future/never-sent) — never
                        // claims the current packet was ACKed, never mutates
                        // recovery state.
                        emit_diagnostic(args, "client", "r9_udp_return_packet_ack_rejected", 0, "");
                    } else {
                        // Accepted-empty stale/duplicate — classification-only
                        // diagnostic; no applied/rejected/state change.
                        emit_diagnostic(
                            args,
                            "client",
                            "r9_udp_return_packet_ack_accepted_empty",
                            0,
                            "",
                        );
                    }
                }
                Err(_) if drop_r9_data => {
                    // R9-3: under the data-drop seam a receive timeout is not
                    // terminal — fall through to the PTO/retransmission block
                    // below before deciding bounded failure.
                    emit_diagnostic(args, "client", "r9_udp_post_return_recv_timeout", 0, "");
                }
                Err(_) => {
                    // R9-4: a receive timeout under ACK-delay/reorder is not
                    // terminal — fall through to the PTO/retransmission block
                    // on the next iteration before deciding bounded failure.
                    // H-R9-038 residual evidence remains classification-only.
                    emit_diagnostic(
                        args,
                        "client",
                        "r9_udp_post_return_residual",
                        0,
                        &format!(
                            ",\"session_outstanding\":{},\"remaining_in_flight\":{}",
                            post_outstanding.len(),
                            rt.as_ref().map(|r| r.in_flight()).unwrap_or(0)
                        ),
                    );
                }
            }
        }
        emit_diagnostic(
            args,
            "client",
            "udp_return_delivery_ack_validated",
            count,
            &format!(",\"ciphertext_bytes\":{}", ack_len),
        );
        post_return_ran = true;
        // M-R9-008 P2: dedicated post-return terminal evidence — emitted only
        // after both the exact Session DeliveryAck and the Carrier packet ACK
        // have drained the post-return recovery ownership to zero. Only emit
        // the reliable-settlement event when a recovery owner actually exists —
        // map_or(0) would falsely imply settlement with no runtime present.
        if let Some(r) = rt.as_ref() {
            emit_diagnostic(
                args,
                "client",
                "r9_udp_post_return_settled",
                0,
                &format!(
                    ",\"stream\":{},\"offset\":{},\"remaining_in_flight\":{}",
                    post_record.stream.0,
                    post_record.offset,
                    r.in_flight()
                ),
            );
        }
    }
    emit_diagnostic(
        args,
        "client",
        "failover_accounting",
        count,
        &format!(
            ",\"udp_confirmed_records\":{},\"udp_confirmed_bytes\":{},\"uncertain_records\":{},\"uncertain_bytes\":{},\"replayed_records\":{},\"replayed_bytes\":{},\"confirmed_records\":{},\"confirmed_bytes\":{},\"duplicate_records\":0,\"duplicate_bytes\":0,\"lost_records\":0,\"lost_bytes\":0,\"conflicting_records\":0,\"conflicting_bytes\":0",
            // H-R9-020: UDP-confirmed = reliable-owned partition start
            // (records[0..uncertain_start]); a completed post-return record is
            // separately tracked, not double-counted here.
            uncertain_start,
            uncertain_start * bytes,
            uncertain_records,
            uncertain_records * bytes,
            replayed_records,
            replayed_records * bytes,
            uncertain_start + replayed_records + usize::from(post_return_ran),
            (uncertain_start + replayed_records + usize::from(post_return_ran)) * bytes
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
    let mut preauth = preauth::ListenerAdmission::new();
    emit_diagnostic(args, "server", "start", 0, ",\"mode\":\"endpoint_rebind\"");
    println!("endpoint_rebind_server_ready");

    let (n, endpoint_a) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint A negotiation timeout"));
    let mut admission = preauth
        .admit_carrier(preauth::CarrierKind::Udp, endpoint_a)
        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
    preauth
        .charge_input(&mut admission, n, 64)
        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
    let mut negotiation =
        VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS).unwrap();
    let selection = negotiation
        .server_accept_hello(&buf[..n])
        .unwrap_or_else(|_| fail("endpoint A negotiation rejected"));
    let selection_permit = preauth
        .charge_response(&mut admission, selection.len())
        .unwrap_or_else(|_| fail("pre-auth response rejected"));
    preauth
        .send_udp_response(&socket, &selection, endpoint_a, selection_permit)
        .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
    let binding = negotiation.authenticated_binding().unwrap();
    let (n, source) = socket
        .recv_from(&mut buf)
        .unwrap_or_else(|_| fail("endpoint A handshake timeout"));
    if source != endpoint_a {
        fail("endpoint A changed before authentication")
    }
    preauth
        .charge_input(&mut admission, n, 4096)
        .unwrap_or_else(|_| fail("pre-auth admission rejected"));
    let (response, mut secure) =
        ResponderHandshake::new_with_prologue_binding(&id, policy, DOMAIN, binding.as_bytes())
            .unwrap()
            .receive_first(&buf[..n], context(1))
            .unwrap_or_else(|_| fail("endpoint A authentication rejected"));
    let response_permit = preauth
        .charge_response(&mut admission, response.len())
        .unwrap_or_else(|_| fail("pre-auth response rejected"));
    preauth
        .send_udp_response(&socket, &response, endpoint_a, response_permit)
        .unwrap_or_else(|_| fail("pre-auth response deadline elapsed"));
    preauth.release(admission);

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
        Some("failover") => match parse(args, "--role", None).as_str() {
            "server" => failover_server(args),
            "client" => failover_client(args),
            _ => fail("failover --role must be server or client"),
        },
        _ => fail("use failover --role server|client"),
    }
}

fn lab(args: &[String]) {
    let json = json_mode(args);
    // R8: bounded executable reliable-UDP lab — a real loopback UDP socket pair
    // drives the committed ReliableUdpRuntime with deterministic sender-side
    // packet suppression, receiver ACKs, PTO retransmit, fresh-evidence health
    // and automatic warm-TCP fallback. Emits structured counters; --json for a
    // machine-readable run. Controlled suppression is not natural loss.
    // R8: `neko lab --scenario reliable-udp` — a bounded executable reliable-UDP
    // local path (real loopback sockets + ReliableUdpRuntime + SessionRuntime
    // dedup + manager-level warm-TCP decision). Omitting `--scenario` keeps the
    // legacy socket-free deterministic failover demo; an unknown, duplicated or
    // value-less scenario fails before any socket or identity side effect.
    match parse_scenario(args).as_deref() {
        None => {}
        Some("reliable-udp") => {
            lab_reliable_udp(args, json);
            return;
        }
        Some(other) => fail(&format!("unknown lab scenario: {other}")),
    }
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

/// Bounded counters for the R8 reliable-UDP lab. A success counter is
/// incremented only after the corresponding operation actually succeeded, so no
/// fabricated success evidence can be emitted from a discarded result.
#[derive(Default)]
struct LabCounters {
    delivered: u64,
    duplicates: u64,
    conflicts: u64,
    application_bytes: u64,
    initial_admitted: u64,
    initial_wire_sent: u64,
    initial_suppressed: u64,
    cwnd_refusals: u64,
    acks_emitted: u64,
    acks_wire_sent: u64,
    acks_suppressed: u64,
    acks_failed: u64,
    acks_applied: u64,
    acks_rejected: u64,
    pto_due: u64,
    pto_fired: u64,
    retransmit_attempts: u64,
    retransmit_wire_sent: u64,
    retransmit_refused: u64,
    newly_lost: u64,
    unauthenticated: u64,
    recv_rejected: u64,
}

/// Mutable state of one bounded R8 lab run. The lab runs on a deterministic
/// logical clock, so a PTO is fired only once the recovery deadline derived from
/// the current RTT estimate and PTO count has actually been reached.
#[derive(Default)]
struct LabState {
    now_us: u64,
    next_admit_us: u64,
    outstanding: std::collections::BTreeMap<u64, u64>,
    c: LabCounters,
    manager_switch: bool,
    partial: bool,
}

const LAB_STEP_US: u64 = 25_000;
const LAB_PTO_GRANULARITY_US: u64 = 1_000;
const LAB_MAX_ACK_DELAY_US: u64 = 0;
const LAB_MAX_SEND_BYTES: u64 = 1200;

/// Optional-value parser used only for `--scenario`. Returns `None` when the key
/// is absent, so the repository required-value `parse` semantics elsewhere are
/// untouched. A missing value or a duplicate key fails deterministically before
/// any socket or identity side effect.
fn parse_scenario(args: &[String]) -> Option<String> {
    let mut found: Option<String> = None;
    let mut i = 1usize;
    while i < args.len() {
        if args[i] == "--scenario" {
            if found.is_some() {
                fail("duplicate --scenario");
            }
            let value = match args.get(i + 1) {
                Some(v) if !v.starts_with("--") => v.clone(),
                _ => fail("missing --scenario value"),
            };
            found = Some(value);
            i += 1;
        }
        i += 1;
    }
    found
}

/// One bounded lab step: drain every currently available authenticated ACK, then
/// fire a PTO only if the current recovery deadline has been reached. Returns
/// `true` when no recovery work remains outstanding.
fn lab_pump(
    rt: &mut neko_carrier::ReliableUdpRuntime,
    sess: &mut neko_crypto::SecureSession,
    sock: &UdpSocket,
    buf: &mut [u8],
    st: &mut LabState,
) -> bool {
    while let Ok(n) = sock.recv(buf) {
        let plain = match sess.open(&buf[..n]) {
            Ok(p) => p,
            Err(_) => {
                st.c.unauthenticated += 1;
                continue;
            }
        };
        let rec = match neko_wire::decode(&plain) {
            Ok(r) => r,
            Err(_) => {
                st.c.acks_rejected += 1;
                continue;
            }
        };
        if rec.record_type != neko_wire::RecordType::Ack {
            continue;
        }
        let ack = match neko_wire::decode_ack(&rec.payload) {
            Ok(a) => a,
            Err(_) => {
                st.c.acks_rejected += 1;
                continue;
            }
        };
        let wire_ranges: Vec<neko_reliable::AckRange> = ack
            .ranges
            .iter()
            .map(|w| neko_reliable::AckRange {
                start: w.start,
                end: w.end,
            })
            .collect();
        let ranges = match neko_reliable::AckRanges::from_ranges(32, &wire_ranges) {
            Ok(r) => r,
            Err(_) => {
                st.c.acks_rejected += 1;
                continue;
            }
        };
        match rt.apply_ack(&ranges, st.now_us, ack.ack_delay_us) {
            Ok(out) => {
                st.c.acks_applied += 1;
                st.c.newly_lost += out.lost_packets.len() as u64;
                for n in out.acked_packets.iter().chain(out.lost_packets.iter()) {
                    st.outstanding.remove(n);
                }
            }
            Err(_) => st.c.acks_rejected += 1,
        }
    }
    let rtt = rt.recovery_engine().rtt;
    let pto_count = rt.recovery_engine().pto_count;
    if let Some(oldest) = st.outstanding.values().next().copied() {
        let deadline = oldest.saturating_add(rtt.pto_us(
            LAB_PTO_GRANULARITY_US,
            LAB_MAX_ACK_DELAY_US,
            pto_count,
        ));
        if st.now_us >= deadline {
            st.c.pto_due += 1;
            let probes = rt.pto_probe();
            if !probes.is_empty() {
                st.c.pto_fired += 1;
            }
            for (frame, plaintext) in probes {
                st.c.retransmit_attempts += 1;
                let msg = ProcessMessage::Data {
                    session: SessionId(1),
                    record: OutboundRecord {
                        stream: StreamId(1),
                        offset: frame.0,
                        data: plaintext.clone(),
                    },
                };
                let wire = neko_wire::encode(&neko_wire::Record {
                    record_type: neko_wire::RecordType::Data,
                    flags: 0,
                    payload: msg.encode().expect("bounded process message"),
                })
                .expect("bounded record");
                let sealed = match sess.seal(&wire) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                // H-R9-041: admission on exact encoded wire bytes.
                let rpn = u64::from_be_bytes(sealed[..8].try_into().expect("sequence prefix"));
                if !admit_retransmit(rt, rpn, st.now_us, &sealed, frame) {
                    st.c.retransmit_refused += 1;
                    continue;
                }
                // H-R9-051: socket send is part of the transaction — a failed
                // send rolls back Recovery ownership and must not count as
                // wire-sent or caller outstanding.
                if sock.send(&sealed).is_err() {
                    let _ = rt.abandon_retransmit(rpn);
                    continue;
                }
                st.c.retransmit_wire_sent += 1;
                st.outstanding.insert(rpn, st.now_us);
            }
        }
    }
    if matches!(
        rt.poll_health(st.now_us / 1000),
        neko_carrier::RuntimeEvent::WarmFallback(_)
    ) {
        st.manager_switch = true;
    }
    rt.in_flight() == 0
}

/// R8 bounded executable reliable-UDP lab. A real loopback UDP socket pair and a
/// `ReliableUdpRuntime` drive authenticated packet send/recv, receiver ACKs,
/// deterministic sender-side packet suppression, deadline-driven PTO retransmit,
/// fresh-evidence health and the manager warm-TCP decision, then emit structured
/// counters. The manager decision is NOT a real TCP transport switch: this
/// command opens no TCP socket. Controlled suppression is not natural Internet
/// loss; lab-only, never a proxy or tunnel.
fn lab_reliable_udp(args: &[String], json: bool) {
    let rounds = parse(args, "--rounds", Some("12"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid rounds"));
    let drop_every = parse(args, "--drop-every", Some("5"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid drop-every"));
    let drop_ack_every = parse(args, "--drop-ack-every", Some("0"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid drop-ack-every"));
    let settle_ms = parse(args, "--settle-ms", Some("3000"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid settle-ms"));
    if !(4..=64).contains(&rounds) {
        fail("rounds outside 4-64");
    }
    if drop_every != 0 && !(2..=16).contains(&drop_every) {
        fail("drop-every outside 0 or 2-16");
    }
    if drop_ack_every > 16 {
        fail("drop-ack-every outside 0-16");
    }
    if !(1..=20_000).contains(&settle_ms) {
        fail("settle-ms outside 1-20000");
    }

    let client_sock = UdpSocket::bind("127.0.0.1:0").unwrap_or_else(|_| fail("client bind"));
    let server_sock = UdpSocket::bind("127.0.0.1:0").unwrap_or_else(|_| fail("server bind"));
    client_sock
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap_or_else(|_| fail("client timeout"));
    server_sock
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap_or_else(|_| fail("server timeout"));
    let server_addr = server_sock
        .local_addr()
        .unwrap_or_else(|_| fail("server addr"));
    let client_addr = client_sock
        .local_addr()
        .unwrap_or_else(|_| fail("client addr"));
    client_sock
        .connect(server_addr)
        .unwrap_or_else(|_| fail("client connect"));
    server_sock
        .connect(client_addr)
        .unwrap_or_else(|_| fail("server connect"));

    let ci = LocalIdentity::generate().unwrap_or_else(|_| fail("client identity"));
    let si = LocalIdentity::generate().unwrap_or_else(|_| fail("server identity"));
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"lab".to_vec(),
        status: TrustStatus::Active,
    }]);
    let ctx = RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 0,
        direction: 0,
    };
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"lab", b"rel")
        .unwrap_or_else(|_| fail("initiator"));
    client_sock
        .send(&ih.first_message().unwrap_or_else(|_| fail("first msg")))
        .unwrap_or_else(|_| fail("send first"));
    let rh = ResponderHandshake::new(&si, policy, b"rel").unwrap_or_else(|_| fail("responder"));
    let mut sbuf = [0u8; 8192];
    let n1 = loop {
        match server_sock.recv(&mut sbuf) {
            Ok(v) => break v,
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => fail("server handshake recv"),
        }
    };
    let (resp, mut server_sess) = rh
        .receive_first(&sbuf[..n1], ctx)
        .unwrap_or_else(|_| fail("receive_first"));
    server_sock
        .send(&resp)
        .unwrap_or_else(|_| fail("send resp"));
    let mut cbuf = [0u8; 8192];
    let n2 = loop {
        match client_sock.recv(&mut cbuf) {
            Ok(v) => break v,
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => fail("client handshake recv"),
        }
    };
    let mut client_sess = ih
        .finish(&cbuf[..n2], ctx)
        .unwrap_or_else(|_| fail("client finish"));

    let server_done = Arc::new(AtomicBool::new(false));
    let sd = Arc::clone(&server_done);
    let mut server_session_rt = SessionRuntime::new(SessionId(1), RuntimeLimits::default(), 0)
        .unwrap_or_else(|_| fail("server session rt"));
    server_session_rt
        .open_stream(StreamId(1), 0)
        .unwrap_or_else(|_| fail("open stream"));
    let server_join = thread::spawn(move || {
        let mut rt =
            neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap_or_else(|_| fail("server rt"));
        let mut c = LabCounters::default();
        let mut ack_seq = 0u64;
        while !sd.load(Ordering::Acquire) {
            let n = match server_sock.recv(&mut sbuf) {
                Ok(n) => n,
                Err(_) => continue,
            };
            if n < 8 {
                c.unauthenticated += 1;
                continue;
            }
            let seq = u64::from_be_bytes(sbuf[..8].try_into().expect("sequence prefix"));
            let plain = match server_sess.open(&sbuf[..n]) {
                Ok(p) => p,
                Err(_) => {
                    c.unauthenticated += 1;
                    continue;
                }
            };
            let rec = match neko_wire::decode(&plain) {
                Ok(r) => r,
                Err(_) => {
                    c.unauthenticated += 1;
                    continue;
                }
            };
            if rec.record_type != neko_wire::RecordType::Data {
                continue;
            }
            match rt.on_packet_received(seq, true) {
                Ok(()) => {}
                Err(_) => {
                    c.recv_rejected += 1;
                    continue;
                }
            }
            if let Ok(ProcessMessage::Data { record, .. }) = ProcessMessage::decode(&rec.payload) {
                match server_session_rt.receive(
                    InboundRecord {
                        stream: record.stream,
                        offset: record.offset,
                        data: record.data,
                    },
                    0,
                ) {
                    Ok(()) => match server_session_rt.pop_receive(0) {
                        Ok(Some(r)) => {
                            c.delivered += 1;
                            c.application_bytes += r.data.len() as u64;
                        }
                        Ok(None) => c.duplicates += 1,
                        Err(_) => c.conflicts += 1,
                    },
                    Err(_) => c.conflicts += 1,
                }
            }
            if let Some(ack) = rt.poll_outgoing_ack(0) {
                ack_seq += 1;
                let fault = drop_ack_every != 0 && ack_seq.is_multiple_of(drop_ack_every as u64);
                let arec = neko_wire::encode(&neko_wire::Record {
                    record_type: neko_wire::RecordType::Ack,
                    flags: 0,
                    payload: neko_wire::encode_ack(&ack).expect("bounded ack"),
                })
                .expect("bounded ack record");
                match server_sess.seal(&arec) {
                    Ok(sealed) => {
                        c.acks_emitted += 1;
                        if fault {
                            c.acks_suppressed += 1;
                        } else if server_sock.send(&sealed).is_ok() {
                            c.acks_wire_sent += 1;
                        } else {
                            c.acks_failed += 1;
                        }
                    }
                    Err(_) => c.acks_failed += 1,
                }
            }
        }
        c
    });

    let mut rt =
        neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap_or_else(|_| fail("client rt"));
    rt.ready_standby(0);
    let udp_key = neko_carrier::ConcurrentPathKey {
        path: PathId(1),
        generation: PathGeneration(1),
    };
    let tcp_key = neko_carrier::ConcurrentPathKey {
        path: PathId(2),
        generation: PathGeneration(1),
    };
    for i in 0..3u64 {
        let _ = rt.manager_mut().observe_readiness(udp_key, true, true, i);
    }
    rt.activate_udp(2);

    let mut st = LabState::default();
    let mut offset = 0u64;

    for r in 0..rounds as u64 {
        if !rt.can_send(LAB_MAX_SEND_BYTES) {
            for _ in 0..64 {
                st.now_us = st.now_us.saturating_add(LAB_STEP_US);
                if lab_pump(&mut rt, &mut client_sess, &client_sock, &mut cbuf, &mut st) {
                    break;
                }
            }
        }
        if !rt.can_send(LAB_MAX_SEND_BYTES) {
            st.c.cwnd_refusals += 1;
            st.partial = true;
            break;
        }
        let interval = rt.pacing_interval_us(LAB_MAX_SEND_BYTES);
        if st.now_us < st.next_admit_us {
            st.now_us = st.next_admit_us;
        }
        st.next_admit_us = st.now_us.saturating_add(interval);

        let payload = format!("lab-{r}").into_bytes();
        let suppress = drop_every != 0 && (r + 1).is_multiple_of(drop_every as u64);
        let frame_offset = offset;
        let msg = ProcessMessage::Data {
            session: SessionId(1),
            record: OutboundRecord {
                stream: StreamId(1),
                offset: frame_offset,
                data: payload.clone(),
            },
        };
        let wire = neko_wire::encode(&neko_wire::Record {
            record_type: neko_wire::RecordType::Data,
            flags: 0,
            payload: msg.encode().expect("bounded process message"),
        })
        .expect("bounded record");
        let sealed = client_sess.seal(&wire).unwrap_or_else(|_| fail("seal"));
        let pn = u64::from_be_bytes(sealed[..8].try_into().expect("sequence prefix"));
        rt.on_packet_sent(
            pn,
            st.now_us,
            sealed.len() as u64,
            neko_reliable::FrameId(frame_offset),
            &payload,
        )
        .unwrap_or_else(|_| fail("record send"));
        st.c.initial_admitted += 1;
        st.outstanding.insert(pn, st.now_us);
        offset = offset
            .checked_add(payload.len() as u64)
            .expect("lab Session byte offset overflowed u64");
        if suppress {
            st.c.initial_suppressed += 1;
        } else if client_sock.send(&sealed).is_ok() {
            st.c.initial_wire_sent += 1;
        }
        for _ in 0..8 {
            st.now_us = st.now_us.saturating_add(LAB_STEP_US);
            if lab_pump(&mut rt, &mut client_sess, &client_sock, &mut cbuf, &mut st) {
                break;
            }
        }
    }

    let settle_deadline_us = st.now_us.saturating_add(settle_ms.saturating_mul(1000));
    while !st.outstanding.is_empty() && st.now_us < settle_deadline_us {
        st.now_us = st.now_us.saturating_add(LAB_STEP_US);
        lab_pump(&mut rt, &mut client_sess, &client_sock, &mut cbuf, &mut st);
    }
    server_done.store(true, Ordering::Release);
    let server_c = match server_join.join() {
        Ok(c) => c,
        Err(_) => fail("server thread panicked"),
    };
    st.c.delivered = server_c.delivered;
    st.c.duplicates = server_c.duplicates;
    st.c.conflicts = server_c.conflicts;
    st.c.application_bytes = server_c.application_bytes;
    st.c.acks_emitted = server_c.acks_emitted;
    st.c.acks_wire_sent = server_c.acks_wire_sent;
    st.c.acks_suppressed = server_c.acks_suppressed;
    st.c.acks_failed = server_c.acks_failed;
    st.c.unauthenticated += server_c.unauthenticated;
    st.c.recv_rejected += server_c.recv_rejected;

    let c = &st.c;
    let remaining_in_flight = rt.in_flight();
    let active_path = match rt.manager().active() {
        Some(k) if k == tcp_key => "tcp",
        Some(k) if k == udp_key => "udp",
        _ => "none",
    };
    drop(client_sock);
    let cleanup_ok = UdpSocket::bind(client_addr).is_ok() && UdpSocket::bind(server_addr).is_ok();

    // Settlement is judged against the runtime's authoritative in-flight view as
    // well as the lab's own ownership map, so `ok` cannot rest on lab-local
    // bookkeeping that has drifted from actual recovery state.
    let settled = st.outstanding.is_empty() && remaining_in_flight == 0;
    let partial = st.partial || !settled;

    let ok = settled
        && !partial
        && c.conflicts == 0
        && c.acks_rejected == 0
        && c.recv_rejected == 0
        && c.delivered == rounds as u64
        && cleanup_ok;

    if json {
        let mut j = String::new();
        j.push_str(r#"{"ok":"#);
        j.push_str(if ok { "true" } else { "false" });
        j.push_str(r#","schema":"nekomusume.reliable-udp-lab.v1","scenario":"reliable-udp","mode":"loopback-controlled","rounds":"#);
        j.push_str(&rounds.to_string());
        j.push_str(r#","drop_every":"#);
        j.push_str(&drop_every.to_string());
        j.push_str(r#","drop_ack_every":"#);
        j.push_str(&drop_ack_every.to_string());
        j.push_str(r#","offered":"#);
        j.push_str(&rounds.to_string());
        j.push_str(r#","settled":"#);
        j.push_str(if settled { "true" } else { "false" });
        j.push_str(r#","partial":"#);
        j.push_str(if partial { "true" } else { "false" });
        j.push_str(r#","session":{"delivered":"#);
        j.push_str(&c.delivered.to_string());
        j.push_str(r#","duplicates":"#);
        j.push_str(&c.duplicates.to_string());
        j.push_str(r#","conflicts":"#);
        j.push_str(&c.conflicts.to_string());
        j.push_str(r#","application_bytes":"#);
        j.push_str(&c.application_bytes.to_string());
        j.push_str(r#"},"initial":{"admitted":"#);
        j.push_str(&c.initial_admitted.to_string());
        j.push_str(r#","wire_sent":"#);
        j.push_str(&c.initial_wire_sent.to_string());
        j.push_str(r#","suppressed":"#);
        j.push_str(&c.initial_suppressed.to_string());
        j.push_str(r#","cwnd_refusals":"#);
        j.push_str(&c.cwnd_refusals.to_string());
        j.push_str(r#"},"acks":{"emitted":"#);
        j.push_str(&c.acks_emitted.to_string());
        j.push_str(r#","wire_sent":"#);
        j.push_str(&c.acks_wire_sent.to_string());
        j.push_str(r#","suppressed":"#);
        j.push_str(&c.acks_suppressed.to_string());
        j.push_str(r#","failed":"#);
        j.push_str(&c.acks_failed.to_string());
        j.push_str(r#","applied":"#);
        j.push_str(&c.acks_applied.to_string());
        j.push_str(r#","rejected":"#);
        j.push_str(&c.acks_rejected.to_string());
        j.push_str(r#"},"recovery":{"pto_due":"#);
        j.push_str(&c.pto_due.to_string());
        j.push_str(r#","pto_fired":"#);
        j.push_str(&c.pto_fired.to_string());
        j.push_str(r#","retransmit_attempts":"#);
        j.push_str(&c.retransmit_attempts.to_string());
        j.push_str(r#","retransmit_wire_sent":"#);
        j.push_str(&c.retransmit_wire_sent.to_string());
        j.push_str(r#","retransmit_refused":"#);
        j.push_str(&c.retransmit_refused.to_string());
        j.push_str(r#","newly_lost":"#);
        j.push_str(&c.newly_lost.to_string());
        j.push_str(r#","remaining_in_flight":"#);
        j.push_str(&remaining_in_flight.to_string());
        j.push_str(r#"},"unauthenticated":"#);
        j.push_str(&c.unauthenticated.to_string());
        j.push_str(r#","recv_rejected":"#);
        j.push_str(&c.recv_rejected.to_string());
        j.push_str(r#","manager_switch":"#);
        j.push_str(if st.manager_switch { "true" } else { "false" });
        j.push_str(r#","manager_active_path":""#);
        j.push_str(active_path);
        j.push_str(r#"","manager_switch_scope":"manager decision only; this command opens no TCP socket","cleanup":""#);
        j.push_str(if cleanup_ok { "verified" } else { "failed" });
        j.push_str(r#""}"#);
        println!("{j}");
    } else {
        println!(
            "scenario=reliable-udp ok={ok} rounds={rounds} settled={settled} partial={partial}"
        );
        println!(
            "session delivered={} duplicates={} conflicts={} application_bytes={}",
            c.delivered, c.duplicates, c.conflicts, c.application_bytes
        );
        println!(
            "initial admitted={} wire_sent={} suppressed={} cwnd_refusals={}",
            c.initial_admitted, c.initial_wire_sent, c.initial_suppressed, c.cwnd_refusals
        );
        println!(
            "acks emitted={} wire_sent={} suppressed={} failed={} applied={} rejected={}",
            c.acks_emitted,
            c.acks_wire_sent,
            c.acks_suppressed,
            c.acks_failed,
            c.acks_applied,
            c.acks_rejected
        );
        println!(
            "recovery pto_due={} pto_fired={} retransmit_attempts={} retransmit_wire_sent={} retransmit_refused={} newly_lost={} remaining_in_flight={}",
            c.pto_due,
            c.pto_fired,
            c.retransmit_attempts,
            c.retransmit_wire_sent,
            c.retransmit_refused,
            c.newly_lost,
            remaining_in_flight
        );
        println!(
            "manager_switch={} manager_active_path={active_path} manager_switch_scope=manager-decision-only-no-tcp-socket cleanup={}",
            st.manager_switch,
            if cleanup_ok { "verified" } else { "failed" }
        );
    }
    if !ok {
        std::process::exit(1);
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

#[derive(Debug)]
struct MatrixProbeArgs {
    target: SocketAddr,
    transport: reachability::Transport,
    version: reachability::IpVersion,
    timeout_ms: u64,
    bytes: usize,
    json: bool,
}

fn parse_matrix_probe_args(args: &[String]) -> MatrixProbeArgs {
    let mut matrix = false;
    let mut target = None;
    let mut transport = None;
    let mut version = None;
    let mut timeout_ms = None;
    let mut bytes = None;
    let mut json = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index].as_str();
        match option {
            "--matrix" => {
                if matrix {
                    fail("duplicate --matrix");
                }
                matrix = true;
                index += 1;
            }
            "--json" => {
                if json {
                    fail("duplicate --json");
                }
                json = true;
                index += 1;
            }
            "--target" | "--transport" | "--ip-version" | "--timeout-ms" | "--bytes" => {
                let value = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .unwrap_or_else(|| fail(&format!("missing value for {option}")))
                    .clone();
                let slot = match option {
                    "--target" => &mut target,
                    "--transport" => &mut transport,
                    "--ip-version" => &mut version,
                    "--timeout-ms" => &mut timeout_ms,
                    "--bytes" => &mut bytes,
                    _ => unreachable!(),
                };
                if slot.replace(value).is_some() {
                    fail(&format!("duplicate {option}"));
                }
                index += 2;
            }
            _ => fail(&format!("unknown matrix argument: {option}")),
        }
    }
    if !matrix {
        fail("missing --matrix");
    }
    let target = target
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| fail("missing or invalid --target"));
    let transport = match transport.as_deref() {
        Some("tcp") => reachability::Transport::Tcp,
        Some("udp") => reachability::Transport::Udp,
        _ => fail("--transport must be tcp or udp"),
    };
    let version = match version.as_deref() {
        Some("ipv4") => reachability::IpVersion::V4,
        Some("ipv6") => reachability::IpVersion::V6,
        _ => fail("--ip-version must be ipv4 or ipv6"),
    };
    let timeout_ms = timeout_ms
        .map(|value| {
            value
                .parse::<u64>()
                .unwrap_or_else(|_| fail("invalid --timeout-ms"))
        })
        .unwrap_or(500);
    let bytes = bytes
        .map(|value| {
            value
                .parse::<usize>()
                .unwrap_or_else(|_| fail("invalid --bytes"))
        })
        .unwrap_or(32);
    reachability::validate(target, version, timeout_ms, bytes).unwrap_or_else(|error| fail(error));
    MatrixProbeArgs {
        target,
        transport,
        version,
        timeout_ms,
        bytes,
        json,
    }
}

fn matrix_probe(args: &[String]) -> ! {
    let parsed = parse_matrix_probe_args(args);
    let artifact = reachability::run(
        parsed.transport,
        parsed.version,
        parsed.target,
        parsed.timeout_ms,
        parsed.bytes,
    );
    if parsed.json {
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
    fn admit_retransmit_uses_encoded_wire_bytes_not_plaintext() {
        // H-R9-050: the admission gate must be driven by the exact encoded
        // wire length — a budget that admits the plaintext length but not the
        // larger encoded length must refuse, and commit no ownership.
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
        // Outstanding frame whose plaintext is retained for retransmit.
        rt.on_packet_sent(0, 0, 400, neko_reliable::FrameId(9), b"p")
            .unwrap();
        // Fill cwnd to leave a budget smaller than the encoded wire size but
        // larger than the retained plaintext: plaintext_len=400 fits in the
        // remaining 800B window, encoded_len=1200 does not.
        let mut n = 1u64;
        while rt.can_send(400) && rt.in_flight() < 28 {
            rt.on_packet_sent(n, n * 1000, 400, neko_reliable::FrameId(3000 + n), b"x")
                .unwrap();
            n += 1;
        }
        // in_flight bytes = 29*400 = 11600; remaining 400 — encoded 1200B
        // retransmit exceeds it even though the 400B plaintext would fit.
        assert!(rt.can_send(400));
        assert!(!rt.can_send(1200));
        let before = rt.in_flight();
        let wire = vec![0u8; 1200]; // simulated encoded datagram, len > plaintext
        let frame = neko_reliable::FrameId(9);
        assert!(
            !admit_retransmit(&mut rt, 100, 1_000_000, &wire, frame),
            "encoded wire bytes must be refused at the budget boundary"
        );
        assert_eq!(rt.in_flight(), before, "refusal commits no ownership");
        // Paired control: same helper admits a wire size that fits.
        let small = vec![0u8; 400];
        assert!(admit_retransmit(&mut rt, 101, 1_000_000, &small, frame));
    }
    #[test]
    fn abandoned_retransmit_restores_committed_sent_watermark() {
        // H-R9-052: a socket-failed retransmit reservation must not become
        // ACK-valid future/never-sent history. After abandon_retransmit, an
        // ACK whose largest == the aborted packet number must be rejected
        // atomically (largest > restored committed largest_sent).
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
        // Send a real packet that stays in flight — frame 1 retains plaintext.
        rt.on_packet_sent(0, 0, 400, neko_reliable::FrameId(1), b"real")
            .unwrap();
        // Admit a retransmit reservation for a new packet number carrying
        // the same stable frame.
        let wire = vec![0u8; 400];
        assert!(admit_retransmit(
            &mut rt,
            100,
            1_000_000,
            &wire,
            neko_reliable::FrameId(1)
        ));
        assert_eq!(rt.in_flight(), 2);
        // Socket send fails — roll back the reservation.
        assert!(rt.abandon_retransmit(100));
        assert_eq!(rt.in_flight(), 1);
        // The aborted number must now be rejected as never-sent: largest=100
        // exceeds the restored committed watermark (largest_sent=0).
        let mut ack = neko_reliable::AckRanges::new(4).unwrap();
        ack.insert(100).unwrap();
        assert!(
            rt.apply_ack(&ack, 1_000_000, 0).is_err(),
            "ACK of aborted packet must be rejected atomically"
        );
        // Real packet remains in flight; no loss/RTT mutation.
        assert_eq!(rt.in_flight(), 1);
    }
    #[test]
    fn abandoned_retransmit_preserves_retired_committed_watermark() {
        // H-R9-053: the committed high-water must survive even when the
        // previous committed packet was already ACKed/retired out of `sent`.
        // After aborting a higher reservation, a late ACK for the retired
        // committed packet must still be accepted-empty, while an ACK for
        // the aborted number is rejected atomically.
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
        // Send and immediately ACK/retire committed packet N=0 (frame 1).
        rt.on_packet_sent(0, 0, 400, neko_reliable::FrameId(1), b"real")
            .unwrap();
        let mut ack_n = neko_reliable::AckRanges::new(4).unwrap();
        ack_n.insert(0).unwrap();
        rt.apply_ack(&ack_n, 1_000_000, 0).unwrap();
        assert_eq!(rt.in_flight(), 0);
        // Reserve higher packet M=100 for a retransmit of a different frame
        // whose plaintext is still retained; socket send fails.
        rt.on_packet_sent(1, 1_000_000, 400, neko_reliable::FrameId(2), b"other")
            .unwrap();
        let wire = vec![0u8; 400];
        assert!(admit_retransmit(
            &mut rt,
            100,
            1_000_000,
            &wire,
            neko_reliable::FrameId(2)
        ));
        assert!(rt.abandon_retransmit(100));
        // A late ACK for retired N=0 is still accepted-empty (largest_sent=1).
        let mut ack_dup = neko_reliable::AckRanges::new(4).unwrap();
        ack_dup.insert(0).unwrap();
        assert!(
            rt.apply_ack(&ack_dup, 2_000_000, 0).is_ok(),
            "late ACK for retired committed packet must remain accepted"
        );
        // ACK for aborted M=100 is rejected atomically (largest_sent=1).
        let mut ack_m = neko_reliable::AckRanges::new(4).unwrap();
        ack_m.insert(100).unwrap();
        assert!(
            rt.apply_ack(&ack_m, 2_000_000, 0).is_err(),
            "ACK of aborted packet must be rejected atomically"
        );
        // H-R9-054: packet-number reuse at or below the committed watermark
        // must be refused by the send owner — reuse of committed N=0 or
        // current watermark=1 must not commit new Recovery/Reno ownership.
        let before_flight = rt.in_flight();
        let before_bytes = rt.recovery_bytes_in_flight();
        let wire_reuse = vec![0u8; 400];
        assert!(
            !admit_retransmit(
                &mut rt,
                0,
                2_000_000,
                &wire_reuse,
                neko_reliable::FrameId(2)
            ),
            "reuse of committed N=0 must be refused"
        );
        assert!(
            !admit_retransmit(
                &mut rt,
                1,
                2_000_000,
                &wire_reuse,
                neko_reliable::FrameId(2)
            ),
            "reuse of committed watermark=1 must be refused"
        );
        assert_eq!(
            rt.in_flight(),
            before_flight,
            "reuse must not commit ownership"
        );
        assert_eq!(
            rt.recovery_bytes_in_flight(),
            before_bytes,
            "reuse must not charge Reno"
        );
        // Paired control: a fresh packet number > committed watermark succeeds.
        let wire2 = vec![0u8; 400];
        assert!(admit_retransmit(
            &mut rt,
            101,
            2_000_000,
            &wire2,
            neko_reliable::FrameId(2)
        ));
        assert_eq!(rt.in_flight(), 2);
    }
    #[test]
    fn socket_send_failure_rolls_back_retransmit_ownership() {
        // H-R9-051: after exact-wire admission succeeds, a socket Err must
        // reverse Recovery ownership, Reno charge, and packet->frame map —
        // no positive sent evidence, no in-flight packet, no caller
        // outstanding identity.
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
        // Outstanding frame whose plaintext is retained for retransmit.
        rt.on_packet_sent(0, 0, 400, neko_reliable::FrameId(9), b"p")
            .unwrap();
        let before_flight = rt.in_flight();
        let before_bytes = rt.recovery_bytes_in_flight();
        // Admit a retransmit reservation.
        let wire = vec![0u8; 400];
        assert!(admit_retransmit(
            &mut rt,
            100,
            1_000_000,
            &wire,
            neko_reliable::FrameId(9)
        ));
        assert_eq!(rt.in_flight(), before_flight + 1);
        assert!(rt.recovery_bytes_in_flight() > before_bytes);
        // Socket send fails — roll back.
        assert!(rt.abandon_retransmit(100));
        assert_eq!(
            rt.in_flight(),
            before_flight,
            "aborted packet must not remain in flight"
        );
        assert_eq!(
            rt.recovery_bytes_in_flight(),
            before_bytes,
            "Reno charge must be reversed"
        );
        // Paired control: a successful send path commits exactly once.
        let wire2 = vec![0u8; 400];
        assert!(admit_retransmit(
            &mut rt,
            101,
            1_000_000,
            &wire2,
            neko_reliable::FrameId(9)
        ));
        assert_eq!(rt.in_flight(), before_flight + 1);
    }
    #[test]
    fn due_pto_probe_is_quiet_before_deadline_and_fires_at_it() {
        // H-R9-055: exercise the real executable decision owner — at
        // deadline_us - 1 the helper performs zero PTO transition (no
        // pto_probe/on_pto, no pto_count increment, no retransmit work); at
        // deadline it performs the legitimate transition.
        let mut rt = neko_carrier::ReliableUdpRuntime::new(1, 1200).unwrap();
        rt.on_packet_sent(0, 0, 400, neko_reliable::FrameId(9), b"p")
            .unwrap();
        let deadline = rt.recovery_engine().next_pto_deadline_us(1_000, 0).unwrap();
        let pto_before = rt.recovery_engine().pto_count;
        let inflight_before = rt.in_flight();
        // Pre-deadline challenge: the same decision owner must do nothing.
        assert!(
            due_pto_probe(&mut rt, deadline - 1).is_none(),
            "pre-deadline must produce zero PTO transition"
        );
        assert_eq!(rt.recovery_engine().pto_count, pto_before);
        assert_eq!(rt.in_flight(), inflight_before);
        // At the deadline the same owner performs the legitimate transition.
        let fired = due_pto_probe(&mut rt, deadline);
        assert!(fired.is_some(), "at-deadline must probe");
        assert!(rt.recovery_engine().pto_count > pto_before);
    }
    #[test]
    fn carrier_retired_fields_projects_every_retired_packet() {
        // H-R9-049 projection discriminator: a multi-retirement typed result
        // must produce one evidence field per packet — never a representative
        // `.last()` collapse.
        let fields = carrier_retired_fields(&[4, 5, 6]);
        assert_eq!(fields.len(), 3);
        assert!(fields[0].contains("\"packet_number\":4"));
        assert!(fields[1].contains("\"packet_number\":5"));
        assert!(fields[2].contains("\"packet_number\":6"));
        assert!(fields.iter().all(|f| f.contains("\"retired\":true")));
        // Singleton stays a singleton.
        let one = carrier_retired_fields(&[7]);
        assert_eq!(one.len(), 1);
    }
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
        let expected_len = expected.data.len();
        let mut outstanding = vec![expected.clone()];
        let mut malformed = 0usize;
        let mut confirmed_acks = std::collections::BTreeSet::new();
        let _delivery = neko_session::SessionRuntime::new(
            SessionId(7001),
            neko_session::RuntimeLimits::default(),
            0,
        )
        .unwrap();
        let outcome = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |d| diagnostics.push(d),
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        )
        .unwrap();
        match outcome {
            UdpAcknowledgement::Session { record, bytes } => {
                assert_eq!(bytes, sealed.len());
                assert_eq!(record.data.len(), expected_len);
            }
            UdpAcknowledgement::Carrier { .. } => panic!("expected a Session DeliveryAck"),
        }
        assert!(outstanding.is_empty(), "matched record must be retired");
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
        let mut outstanding = vec![expected.clone()];
        let mut malformed = 0usize;
        let mut confirmed_acks = std::collections::BTreeSet::new();
        let _delivery = neko_session::SessionRuntime::new(
            SessionId(7001),
            neko_session::RuntimeLimits::default(),
            0,
        )
        .unwrap();
        let err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |_| {},
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        )
        .unwrap_err();
        assert_eq!(err, "UDP delivery acknowledgement malformed bound exceeded");
    }

    #[test]
    fn reliable_demux_matches_a_later_record_ack_while_an_earlier_one_is_outstanding() {
        // H-R9-001 discriminating regression: with two outstanding reliable-owned
        // logical records, a Session DeliveryAck for the LATER record must be
        // consumed as that record's confirmation even though the earlier one has
        // not been acknowledged. The previous serial single-`expected` receiver
        // did not match it, so the confirmation was swallowed and the wait timed
        // out — this test fails under that shape.
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (mut sender, mut receiver) = secure_pair();
        let rec0 = OutboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: vec![7; 16],
        };
        let rec1 = OutboundRecord {
            stream: StreamId(1),
            offset: 16,
            data: vec![7; 16],
        };
        let ack1 = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 16,
            len: 16,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&ack1).unwrap();
        server
            .send_to(&sealed, client.local_addr().unwrap())
            .unwrap();
        let mut outstanding = vec![rec0.clone(), rec1.clone()];
        let mut malformed = 0usize;
        let mut confirmed_acks = std::collections::BTreeSet::new();
        let _delivery = neko_session::SessionRuntime::new(
            SessionId(7001),
            neko_session::RuntimeLimits::default(),
            0,
        )
        .unwrap();
        let outcome = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |_| {},
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        )
        .unwrap();
        match outcome {
            UdpAcknowledgement::Session { record, bytes } => {
                assert_eq!(
                    record.offset, 16,
                    "the later record ACK must be matched, not swallowed"
                );
                assert_eq!(bytes, sealed.len());
            }
            UdpAcknowledgement::Carrier { .. } => panic!("expected a Session DeliveryAck"),
        }
        assert_eq!(
            outstanding.len(),
            1,
            "exactly the matched record is retired"
        );
        assert_eq!(
            outstanding[0].offset, 0,
            "the still-unacknowledged earlier record stays outstanding"
        );
    }

    #[test]
    fn reliable_demux_bounds_an_authenticated_unexpected_logical_ack() {
        // A stale/duplicate/unexpected logical ACK is a typed bounded negative,
        // never a match and never an unbounded wait — in reliable mode too.
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (mut sender, mut receiver) = secure_pair();
        let rec0 = OutboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: vec![7; 16],
        };
        let unexpected = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 4096,
            len: 16,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&unexpected).unwrap();
        for _ in 0..MAX_POST_HANDSHAKE_MALFORMED {
            server
                .send_to(&sealed, client.local_addr().unwrap())
                .unwrap();
        }
        let mut outstanding = vec![rec0.clone()];
        let mut reasons = Vec::new();
        let mut malformed = 0usize;
        let mut confirmed_acks = std::collections::BTreeSet::new();
        let _delivery = neko_session::SessionRuntime::new(
            SessionId(7001),
            neko_session::RuntimeLimits::default(),
            0,
        )
        .unwrap();
        let err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |d| reasons.push(d),
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        )
        .unwrap_err();
        assert_eq!(err, "UDP delivery acknowledgement malformed bound exceeded");
        assert!(reasons.contains(&"unexpected_logical_ack"), "{reasons:?}");
        assert_eq!(outstanding.len(), 1, "no unmatched ACK retires a record");
    }

    #[test]
    fn reliable_demux_accepts_exact_duplicate_ack_for_confirmed_range() {
        // H-R9-056/059: a freshly sealed Session DeliveryAck naming a range
        // this operation already exact-matched is an idempotent zero-delta
        // duplicate — classification-only accepted-empty, never malformed.
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (mut sender, mut receiver) = secure_pair();
        // The operation already exact-matched (stream=1, offset=0, len=16) —
        // the caller-owned witness records it, bounded by admitted records.
        let mut confirmed_acks = std::collections::BTreeSet::new();
        confirmed_acks.insert((1u64, 0u64, 16u64));
        assert!(confirmed_acks.contains(&(1u64, 0u64, 16u64)));
        // Fresh exact duplicate ACK for that confirmed range.
        let dup = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 0,
            len: 16,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&dup).unwrap();
        server
            .send_to(&sealed, client.local_addr().unwrap())
            .unwrap();
        let mut outstanding = vec![OutboundRecord {
            stream: StreamId(1),
            offset: 64,
            data: vec![9; 16],
        }];
        let mut reasons = Vec::new();
        let mut malformed = 0usize;
        let _err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_millis(300),
            &mut |d| reasons.push(d),
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        );
        // The duplicate is classification-only — no malformed charge.
        assert_eq!(malformed, 0, "{reasons:?}");
        assert!(
            reasons.contains(&"accepted_empty_logical_ack"),
            "{reasons:?}"
        );
        assert!(!reasons.contains(&"unexpected_logical_ack"), "{reasons:?}");
    }

    #[test]
    fn reliable_demux_rejects_below_watermark_subrange_ack() {
        // H-R9-058 negative: an authenticated ACK covering a SUBRANGE of a
        // confirmed range (end <= watermark but not an exact confirmed range)
        // is unadmitted logical feedback — bounded unexpected/malformed, never
        // silently accepted-empty.
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (mut sender, mut receiver) = secure_pair();
        let mut confirmed_acks = std::collections::BTreeSet::new();
        confirmed_acks.insert((1u64, 0u64, 16u64));
        // Subrange: offset 0 len 8 — below the confirmed (0,16) but not exact.
        let sub = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 0,
            len: 8,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&sub).unwrap();
        for _ in 0..MAX_POST_HANDSHAKE_MALFORMED {
            server
                .send_to(&sealed, client.local_addr().unwrap())
                .unwrap();
        }
        let mut outstanding = vec![OutboundRecord {
            stream: StreamId(1),
            offset: 64,
            data: vec![9; 16],
        }];
        let mut reasons = Vec::new();
        let mut malformed = 0usize;
        let _err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |d| reasons.push(d),
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        );
        assert!(reasons.contains(&"unexpected_logical_ack"), "{reasons:?}");
        assert!(malformed > 0, "{reasons:?}");
        assert!(
            !reasons.contains(&"accepted_empty_logical_ack"),
            "{reasons:?}"
        );
    }

    #[test]
    fn reliable_demux_rejects_zero_length_ack_even_at_watermark() {
        // H-R9-058 negative: a fresh authenticated len==0 DeliveryAck is
        // malformed, never accepted-empty — even when the operation witness
        // already holds a confirmed range at the same stream.
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = server.local_addr().unwrap();
        let (mut sender, mut receiver) = secure_pair();
        let mut confirmed_acks = std::collections::BTreeSet::new();
        confirmed_acks.insert((1u64, 0u64, 16u64));
        // Zero-length ACK at offset 0 — never an admitted record.
        let zero = ProcessMessage::DeliveryAck {
            session: SessionId(7001),
            stream: StreamId(1),
            offset: 0,
            len: 0,
        }
        .encode()
        .unwrap();
        let sealed = sender.seal_unreliable(&zero).unwrap();
        for _ in 0..MAX_POST_HANDSHAKE_MALFORMED {
            server
                .send_to(&sealed, client.local_addr().unwrap())
                .unwrap();
        }
        let mut outstanding = vec![OutboundRecord {
            stream: StreamId(1),
            offset: 64,
            data: vec![9; 16],
        }];
        let mut reasons = Vec::new();
        let mut malformed = 0usize;
        let _err = recv_udp_delivery_ack(
            &client,
            peer,
            &mut receiver,
            &mut outstanding,
            b"selection",
            b"noise",
            Instant::now() + Duration::from_secs(1),
            &mut |d| reasons.push(d),
            None,
            &mut malformed,
            Instant::now(),
            &mut confirmed_acks,
        );
        assert!(malformed > 0, "{reasons:?}");
        assert!(
            !reasons.contains(&"accepted_empty_logical_ack"),
            "{reasons:?}"
        );
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
