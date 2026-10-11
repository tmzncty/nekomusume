//! Research-scope SOCKS5 explicit-proxy entry (D071/D072; supersedes the
//! D051 no-proxy clause inside this boundary).
//!
//! TCP-only CONNECT subset of RFC 1928; the no-auth greeting segment is used
//! because authentication is the neko Noise handshake, which precedes any
//! SOCKS parsing. One SOCKS connection maps to one neko TCP session carrying
//! one stream (D071 §2). The server dials out only against an explicit
//! `--allow IP:PORT` whitelist — no entries, no start (fail-closed, D067
//! precedent) — and internal ranges are refused even when listed. Target
//! addresses never reach logs; only allow/deny counters do.
//!
//! Application records inside the session carry a one-byte tag:
//! `0x00` data · `0x01` half-close (FIN) · `0x02` CONNECT request
//! (`[ip bytes][be port]`) · `0x03` CONNECT reply (`[code]`).
//!
//! Pumping is stop-and-wait per record and per direction: one record in
//! flight each way, so the per-connection footprint is two chunk-sized
//! records plus acks — inside the D072 64 KiB budget by construction (the
//! declared runtime queue byte ceiling is itself 64 KiB).
//!
//! Transient socket conditions (`WouldBlock`, `Interrupted`) are retried,
//! and the neko read side tolerates `IDLE_CAP` of silence before declaring
//! death: carrier-level jitter — including failover-shaped reconnect
//! windows under the r9 contract — must not be misjudged as session death
//! by the pump (D071 decision 5). Half-close is propagated as a FIN record
//! and never tears down the reverse direction.

use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, SecureSession,
    TrustPolicy, TrustRecord, TrustStatus,
};
use neko_session::{
    InboundRecord, ProcessMessage, RuntimeLimits, SessionId, SessionRuntime, StreamId,
};
use neko_wire::{NEGOTIATION_VERSION, NegotiationRole, VersionNegotiator};
use std::collections::HashSet;
use std::io::{ErrorKind, Read, Write};
use std::net::{
    IpAddr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener, TcpStream,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const DOMAIN: &[u8] = b"nekomusume/neko-cli/socks/v1";
const SCOPE: &[u8] = b"neko-cli/socks";
const SUPPORTED_VERSIONS: &[u16] = &[NEGOTIATION_VERSION];
const SESSION: SessionId = SessionId(0x4e454b4f);
const STREAM: StreamId = StreamId(1);
/// D072: global concurrent SOCKS connections (multistream MAX_STREAMS=16
/// precedent). The 17th concurrent connection is refused, never queued.
const MAX_CONNS: usize = 16;
/// Per-record application payload chunk. Stop-and-wait keeps exactly one of
/// these in flight per direction.
const PUMP_CHUNK: usize = 1024;
/// D071 decision 3: SOCKS5 parser input cap (greeting ≤ 257 B, CONNECT
/// ≤ 262 B; 512 B is ample headroom).
const SOCKS_CAP: usize = 512;
const MAX_FRAME: usize = neko_session::PROCESS_FRAME_MAX + 8 + neko_crypto::RECORD_CONTEXT_LEN + 16;
/// D071 decision 4: socks rides the D068 measurement exception without
/// widening it — session total bytes hard-capped at 2 GiB.
const SESSION_TOTAL_BYTES: usize = 2 * 1024 * 1024 * 1024;
const EXCHANGE_DEADLINE: Duration = Duration::from_secs(5);
/// Hard idle cap: a neko socket silent this long is a dead session. Shorter
/// silences (carrier migration windows included) are tolerated.
const IDLE_CAP: Duration = Duration::from_secs(600);
const DIAL_TIMEOUT: Duration = Duration::from_secs(5);

const TAG_DATA: u8 = 0x00;
const TAG_FIN: u8 = 0x01;
const TAG_CONNECT: u8 = 0x02;
const TAG_CONNECT_REPLY: u8 = 0x03;

fn fail(e: &str) -> ! {
    eprintln!("neko: {e}");
    std::process::exit(2)
}

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

fn identity_path(args: &[String]) -> PathBuf {
    let path = option(args, "--identity").unwrap_or_else(|| fail("--identity is required"));
    PathBuf::from(path)
}

fn read_identity_at(path: &PathBuf) -> LocalIdentity {
    crate::read_identity(path).unwrap_or_else(|| fail("identity is required"))
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

fn limits() -> RuntimeLimits {
    RuntimeLimits {
        max_streams: 2,
        max_queue_records: 8,
        // D072: the declared queue byte ceiling IS the per-connection pump
        // buffer cap.
        max_queue_bytes: 64 * 1024,
        max_total_bytes: SESSION_TOTAL_BYTES,
        max_record_bytes: 4096,
        max_session_window: 1 << 20,
        max_stream_window: 1 << 20,
        ..RuntimeLimits::default()
    }
}

// ---------------------------------------------------------------- framing

fn frame_write(s: &mut TcpStream, b: &[u8]) -> Result<(), String> {
    if b.len() > MAX_FRAME {
        return Err("frame exceeds bound".into());
    }
    // Single staged write (P1-pre precedent): NODELAY must not split the
    // length header from the payload.
    let mut staged = Vec::with_capacity(4 + b.len());
    staged.extend_from_slice(&(b.len() as u32).to_be_bytes());
    staged.extend_from_slice(b);
    s.write_all(&staged).map_err(|e| e.to_string())?;
    s.flush().map_err(|e| e.to_string())
}

fn frame_read(s: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len = [0_u8; 4];
    read_exact(s, &mut len)?;
    let n = u32::from_be_bytes(len) as usize;
    if n > MAX_FRAME {
        return Err("frame exceeds bound".into());
    }
    let mut buf = vec![0_u8; n];
    read_exact(s, &mut buf)?;
    Ok(buf)
}

fn read_exact(s: &mut TcpStream, buf: &mut [u8]) -> Result<(), String> {
    let mut filled = 0;
    while filled < buf.len() {
        match s.read(&mut buf[filled..]) {
            Ok(0) => return Err("connection closed".into()),
            Ok(n) => filled += n,
            Err(e) if is_transient(&e) => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

fn is_transient(e: &std::io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted)
}

// ------------------------------------------------------------ SOCKS5 parse

#[derive(Debug, PartialEq, Eq)]
enum SocksError {
    Malformed,
    NotNoAuth,
    CommandUnsupported,
    AddressUnsupported,
}

/// Validate a client greeting `[VER=5][NMETHODS][METHODS..]`: at least one
/// offered method must be `0x00` (no-auth; the neko handshake did the real
/// auth).
fn parse_greeting(buf: &[u8]) -> Result<(), SocksError> {
    if buf.len() < 2 || buf[0] != 5 {
        return Err(SocksError::Malformed);
    }
    let n = buf[1] as usize;
    if buf.len() != 2 + n || n == 0 {
        return Err(SocksError::Malformed);
    }
    if buf[2..2 + n].contains(&0x00) {
        Ok(())
    } else {
        Err(SocksError::NotNoAuth)
    }
}

/// Parse a CONNECT request. IPv4/IPv6 literals only — domain targets answer
/// `0x08` (address type unsupported; DNS deferred by D071).
fn parse_connect(buf: &[u8]) -> Result<SocketAddr, SocksError> {
    if buf.len() < 4 || buf[0] != 5 || buf[2] != 0 {
        return Err(SocksError::Malformed);
    }
    if buf[1] != 1 {
        return Err(SocksError::CommandUnsupported);
    }
    match buf[3] {
        1 => {
            if buf.len() != 4 + 4 + 2 {
                return Err(SocksError::Malformed);
            }
            let mut ip = [0_u8; 4];
            ip.copy_from_slice(&buf[4..8]);
            let port = u16::from_be_bytes([buf[8], buf[9]]);
            Ok(SocketAddr::V4(SocketAddrV4::new(ip.into(), port)))
        }
        4 => {
            if buf.len() != 4 + 16 + 2 {
                return Err(SocksError::Malformed);
            }
            let mut ip = [0_u8; 16];
            ip.copy_from_slice(&buf[4..20]);
            let port = u16::from_be_bytes([buf[20], buf[21]]);
            Ok(SocketAddr::V6(SocketAddrV6::new(ip.into(), port, 0, 0)))
        }
        3 => Err(SocksError::AddressUnsupported),
        _ => Err(SocksError::Malformed),
    }
}

fn read_socks_greeting(s: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut head = [0_u8; 2];
    read_exact(s, &mut head)?;
    if head[0] != 5 || head[1] == 0 {
        return Err("bad socks greeting".into());
    }
    let mut msg = vec![0_u8; 2 + head[1] as usize];
    msg[0] = head[0];
    msg[1] = head[1];
    read_exact(s, &mut msg[2..])?;
    if msg.len() > SOCKS_CAP {
        return Err("socks message exceeds bound".into());
    }
    Ok(msg)
}

fn read_socks_connect(s: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut head = [0_u8; 4];
    read_exact(s, &mut head)?;
    if head[0] != 5 {
        return Err("bad socks version".into());
    }
    let mut msg = head.to_vec();
    let rest = match head[3] {
        1 => 4 + 2,
        4 => 16 + 2,
        3 => {
            let mut n = [0_u8; 1];
            read_exact(s, &mut n)?;
            msg.extend_from_slice(&n);
            n[0] as usize + 2
        }
        _ => return Err("bad socks address type".into()),
    };
    if msg.len() + rest > SOCKS_CAP {
        return Err("socks message exceeds bound".into());
    }
    let mut tail = vec![0_u8; rest];
    read_exact(s, &mut tail)?;
    msg.extend_from_slice(&tail);
    Ok(msg)
}

/// Minimal SOCKS5 reply with an unspecified bound address (RFC 1928 allows
/// `0.0.0.0:0` when the server does not report one).
fn socks_reply(code: u8) -> [u8; 10] {
    [5, code, 0, 1, 0, 0, 0, 0, 0, 0]
}

// --------------------------------------------------------------- whitelist

/// True for ranges the outbound dialer refuses even when whitelisted (D071
/// decision 3 double insurance; SSRF-shaped destinations).
fn is_internal(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 0
                || o[0] == 10
                || o[0] == 127
                || (o[0] == 169 && o[1] == 254)
                || (o[0] == 172 && (o[1] & 0xf0) == 16)
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 100 && (o[1] & 0xc0) == 64)
        }
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            v6 == Ipv6Addr::LOCALHOST || (seg[0] & 0xfe00) == 0xfc00 || (seg[0] & 0xffc0) == 0xfe80
        }
    }
}

fn build_whitelist(args: &[String]) -> HashSet<SocketAddr> {
    let mut allowed = HashSet::new();
    for i in 0..args.len().saturating_sub(1) {
        if args[i] != "--allow" {
            continue;
        }
        let addr: SocketAddr = args[i + 1]
            .parse()
            .unwrap_or_else(|_| fail("--allow entry is not IP:PORT: refused"));
        if is_internal(addr.ip()) {
            fail("--allow refuses internal-range targets (fail-closed)");
        }
        allowed.insert(addr);
    }
    if allowed.is_empty() {
        fail("socks-server requires at least one --allow IP:PORT (default deny, fail-closed)");
    }
    allowed
}

fn permits(whitelist: &HashSet<SocketAddr>, addr: SocketAddr) -> bool {
    !is_internal(addr.ip()) && whitelist.contains(&addr)
}

// ----------------------------------------------------------- neko handshake

fn server_handshake(
    socket: &mut TcpStream,
    id: &LocalIdentity,
    client_key: &[u8],
    admission: &mut crate::preauth::ListenerAdmission,
    ticket: &mut crate::preauth::AdmissionTicket,
) -> Result<SecureSession, &'static str> {
    let mut negotiation = VersionNegotiator::new(NegotiationRole::Server, SUPPORTED_VERSIONS)
        .map_err(|_| "handshake setup failed")?;
    let mut reader = crate::framed::FramedReader::new(MAX_FRAME);
    let deadline = Instant::now() + EXCHANGE_DEADLINE;
    let hello = crate::preauth::read_staged_frame(
        &mut reader,
        socket,
        MAX_FRAME,
        deadline,
        admission,
        ticket,
        64,
        64,
    )
    .map_err(|_| "handshake rejected")?;
    let response = negotiation
        .server_accept_hello(&hello)
        .map_err(|_| "handshake rejected")?;
    let permit = admission
        .charge_response(ticket, response.len() + 4)
        .map_err(|_| "pre-auth response rejected")?;
    admission
        .send_tcp_response(socket, &response, permit)
        .map_err(|_| "pre-auth response deadline elapsed")?;
    let binding = negotiation
        .authenticated_binding()
        .map_err(|_| "handshake rejected")?;
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client_key.to_vec(),
        scope: SCOPE.to_vec(),
        status: TrustStatus::Active,
    }]);
    let hs = ResponderHandshake::new_with_prologue_binding(id, policy, DOMAIN, binding.as_bytes())
        .map_err(|_| "handshake setup failed")?;
    let first = crate::preauth::read_staged_frame(
        &mut reader,
        socket,
        1024,
        deadline,
        admission,
        ticket,
        4096,
        4096,
    )
    .map_err(|_| "handshake rejected")?;
    let (response, session) = hs
        .receive_first(&first, context())
        .map_err(|_| "handshake rejected")?;
    let permit = admission
        .charge_response(ticket, response.len() + 4)
        .map_err(|_| "pre-auth response rejected")?;
    admission
        .send_tcp_response(socket, &response, permit)
        .map_err(|_| "pre-auth response deadline elapsed")?;
    Ok(session)
}

fn client_handshake(
    socket: &mut TcpStream,
    id: &LocalIdentity,
    server_key: &[u8],
) -> Result<SecureSession, &'static str> {
    let mut negotiation = VersionNegotiator::new(NegotiationRole::Client, SUPPORTED_VERSIONS)
        .map_err(|_| "handshake setup failed")?;
    let hello = negotiation
        .client_hello()
        .map_err(|_| "handshake rejected")?;
    frame_write(socket, &hello).map_err(|_| "handshake rejected")?;
    let response = frame_read(socket).map_err(|_| "handshake rejected")?;
    negotiation
        .client_accept_response(&response)
        .map_err(|_| "handshake rejected")?;
    let binding = negotiation
        .authenticated_binding()
        .map_err(|_| "handshake rejected")?;
    let mut hs = InitiatorHandshake::new_with_prologue_binding(
        id,
        server_key,
        SCOPE,
        DOMAIN,
        binding.as_bytes(),
    )
    .map_err(|_| "handshake setup failed")?;
    frame_write(
        socket,
        &hs.first_message().map_err(|_| "handshake rejected")?,
    )
    .map_err(|_| "handshake rejected")?;
    let response = frame_read(socket).map_err(|_| "handshake rejected")?;
    let session = hs
        .finish(&response, context())
        .map_err(|_| "handshake rejected")?;
    negotiation.admit_data().map_err(|_| "handshake rejected")?;
    Ok(session)
}

// ------------------------------------------------------------- pump shared

/// One SecureSession serves both directions; the upload and download threads
/// share it (plus the neko socket and runtime) behind a mutex. Neither
/// thread holds the lock while blocking on a socket.
struct Shared {
    neko: TcpStream,
    session: SecureSession,
    runtime: SessionRuntime,
    now_ms: u64,
}

impl Shared {
    fn open_stream(&mut self) -> Result<(), String> {
        self.runtime
            .open_stream(STREAM, self.now_ms)
            .map_err(|_| "stream open rejected".to_string())
    }

    /// Queue one tagged payload, seal, write. Returns the (offset, len) the
    /// peer's DeliveryAck will carry.
    fn send_record(&mut self, payload: &[u8]) -> Result<(u64, usize), String> {
        self.now_ms += 1;
        self.runtime
            .queue_send(STREAM, payload, self.now_ms)
            .map_err(|_| "queue rejected".to_string())?;
        let record = self
            .runtime
            .pop_send(self.now_ms)
            .map_err(|_| "queue starved".to_string())?
            .ok_or("queue starved")?;
        let (offset, len) = (record.offset, record.data.len());
        let wire = self
            .session
            .seal(
                &ProcessMessage::Data {
                    session: SESSION,
                    record,
                }
                .encode()
                .map_err(|_| "message encode failed".to_string())?,
            )
            .map_err(|_| "encryption failed".to_string())?;
        frame_write(&mut self.neko, &wire)?;
        Ok((offset, len))
    }

    /// Release a ledger slot after the peer acknowledged (offset, len).
    fn release(&mut self, offset: u64, len: usize) -> Result<(), String> {
        self.now_ms += 1;
        self.runtime
            .delivery_ack(STREAM, offset, len, self.now_ms)
            .map_err(|_| "delivery ack rejected".to_string())
    }
}

enum Inbound {
    /// Application payload with its tag byte stripped; `fin` marks a
    /// half-close record.
    Payload { data: Vec<u8>, fin: bool },
    /// A DeliveryAck for the upload direction.
    Ack { offset: u64, len: usize },
}

/// Read one sealed frame from `reader` (a private clone of the neko socket),
/// then process it against the shared session/runtime: receive + ack inbound
/// records, surface acks, never block while holding the lock.
fn read_inbound(reader: &mut TcpStream, shared: &Arc<Mutex<Shared>>) -> Result<Inbound, String> {
    let wire = frame_read(reader)?;
    let mut g = shared.lock().unwrap();
    let plain = g
        .session
        .open(&wire)
        .map_err(|_| "unauthenticated transport frame".to_string())?;
    let msg =
        ProcessMessage::decode(&plain).map_err(|_| "malformed process message".to_string())?;
    match msg {
        ProcessMessage::Data { session, record } => {
            if session != SESSION || record.stream != STREAM {
                return Err("record bound mismatch".into());
            }
            let (offset, len) = (record.offset, record.data.len());
            g.now_ms += 1;
            let now = g.now_ms;
            g.runtime
                .receive(
                    InboundRecord {
                        stream: record.stream,
                        offset: record.offset,
                        data: record.data,
                    },
                    now,
                )
                .map_err(|_| "out-of-order or over-limit record".to_string())?;
            // Ack before popping so the peer's stop-and-wait never waits on
            // our local consumption.
            let ack = ProcessMessage::DeliveryAck {
                session: SESSION,
                stream: STREAM,
                offset,
                len,
            };
            let ack_wire = g
                .session
                .seal(
                    &ack.encode()
                        .map_err(|_| "message encode failed".to_string())?,
                )
                .map_err(|_| "encryption failed".to_string())?;
            frame_write(&mut g.neko, &ack_wire)?;
            g.now_ms += 1;
            let now = g.now_ms;
            let inbound = g
                .runtime
                .pop_receive(now)
                .map_err(|_| "receive release failed".to_string())?
                .ok_or("receive starved")?;
            // The application tag byte stays in the payload; every consumer
            // (connect-phase both sides, pump downloader) parses it itself.
            let fin = !inbound.data.is_empty() && inbound.data[0] == TAG_FIN;
            Ok(Inbound::Payload {
                data: inbound.data,
                fin,
            })
        }
        ProcessMessage::DeliveryAck { offset, len, .. } => Ok(Inbound::Ack { offset, len }),
        _ => Err("unexpected process message".into()),
    }
}

/// Acks flow downloader→uploader through a condvar; `dead` latches fatal
/// downloader conditions so a parked uploader unblocks immediately.
struct AckGate {
    state: Mutex<AckState>,
    signal: Condvar,
}

#[derive(Default)]
struct AckState {
    acked: Option<(u64, usize)>,
    dead: bool,
}

impl AckGate {
    fn new() -> Arc<AckGate> {
        Arc::new(AckGate {
            state: Mutex::new(AckState::default()),
            signal: Condvar::new(),
        })
    }

    fn notify_ack(&self, offset: u64, len: usize) {
        let mut st = self.state.lock().unwrap();
        st.acked = Some((offset, len));
        self.signal.notify_all();
    }

    fn mark_dead(&self) {
        let mut st = self.state.lock().unwrap();
        st.dead = true;
        self.signal.notify_all();
    }

    fn wait_ack(&self, offset: u64, len: usize, deadline: Instant) -> bool {
        let mut st = self.state.lock().unwrap();
        while !st.dead && st.acked.is_none_or(|(a, l)| a != offset || l != len) {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let (guard, _) = self
                .signal
                .wait_timeout(st, deadline.saturating_duration_since(now))
                .unwrap();
            st = guard;
        }
        !st.dead
    }

    /// Park until death (peer EOF/fatal) — used by an uploader whose local
    /// side half-closed while the reverse direction still lives.
    fn park_until_dead(&self) {
        let mut st = self.state.lock().unwrap();
        while !st.dead {
            let (guard, _) = self
                .signal
                .wait_timeout(st, Duration::from_secs(60))
                .unwrap();
            st = guard;
        }
    }
}

/// The bidirectional pump. `local` is the SOCKS-local socket (client) or the
/// whitelisted outbound socket (server). Runs until both directions close.
fn pump(
    local: TcpStream,
    neko: TcpStream,
    session: SecureSession,
    runtime: SessionRuntime,
) -> Result<(), String> {
    let mut shared_core = Shared {
        neko,
        session,
        runtime,
        now_ms: 0,
    };
    // The connect-phase owner already opened STREAM before handing the
    // session to the pump; re-opening would be an InvalidTransition.
    let _ = &mut shared_core;
    let shared = Arc::new(Mutex::new(shared_core));

    let mut reader = shared
        .lock()
        .unwrap()
        .neko
        .try_clone()
        .map_err(|e| e.to_string())?;
    reader
        .set_read_timeout(Some(EXCHANGE_DEADLINE))
        .map_err(|e| e.to_string())?;
    let mut local_down = local.try_clone().map_err(|e| e.to_string())?;
    let mut local_up = local;

    let gate = AckGate::new();
    let gate_up = Arc::clone(&gate);
    let shared_up = Arc::clone(&shared);

    // Downloader: neko → local. Owns the reader clone; never holds the
    // shared lock across a blocking read.
    let down = thread::spawn(move || {
        let mut last_activity = Instant::now();
        let result = loop {
            match read_inbound(&mut reader, &shared) {
                Ok(Inbound::Payload { data, fin }) => {
                    last_activity = Instant::now();
                    if fin {
                        // Propagate the remote half-close; the local side
                        // may still send (half-close never kills the pump).
                        let _ = local_down.shutdown(Shutdown::Write);
                    } else if data.len() > 1 {
                        // data[0] is TAG_DATA; the payload follows.
                        if let Err(e) = local_down.write_all(&data[1..]) {
                            break Err(format!("local write: {e}"));
                        }
                    }
                }
                Ok(Inbound::Ack { offset, len }) => {
                    last_activity = Instant::now();
                    gate.notify_ack(offset, len);
                }
                Err(e) => {
                    if e == "connection closed" {
                        break Ok(());
                    }
                    // Timeouts surface as WouldBlock-shaped read errors from
                    // frame_read; only the idle cap decides death.
                    if last_activity.elapsed() < IDLE_CAP {
                        continue;
                    }
                    break Err(e);
                }
            }
        };
        gate.mark_dead();
        let _ = local_down.shutdown(Shutdown::Both);
        result
    });

    // Uploader: local → neko. Stop-and-wait: one record in flight, its ack
    // arrives via the gate before the next read.
    let mut buf = [0_u8; PUMP_CHUNK];
    let mut pending: Option<(u64, usize)> = None;
    let mut fin_sent = false;
    let up: Result<(), String> = loop {
        if let Some((offset, len)) = pending {
            if !gate_up.wait_ack(offset, len, Instant::now() + EXCHANGE_DEADLINE) {
                break Err("ack deadline elapsed".into());
            }
            let mut g = shared_up.lock().unwrap();
            if let Err(e) = g.release(offset, len) {
                break Err(e);
            }
            pending = None;
            if fin_sent {
                // Local EOF delivered and acknowledged. The reverse
                // direction may still be streaming: park until it resolves.
                gate_up.park_until_dead();
                break Ok(());
            }
        }
        let chunk = match local_up.read(&mut buf) {
            Ok(0) => {
                let frame = vec![TAG_FIN];
                frame
            }
            Ok(n) => {
                let mut frame = Vec::with_capacity(1 + n);
                frame.push(TAG_DATA);
                frame.extend_from_slice(&buf[..n]);
                frame
            }
            Err(e) if is_transient(&e) => continue,
            Err(e) => break Err(format!("local read: {e}")),
        };
        let fin = chunk[0] == TAG_FIN && chunk.len() == 1;
        let mut g = shared_up.lock().unwrap();
        let sent = match g.send_record(&chunk) {
            Ok(v) => v,
            Err(e) => break Err(e),
        };
        drop(g);
        if fin {
            fin_sent = true;
        }
        pending = Some(sent);
    };
    let _ = up;
    down.join().map_err(|_| "downloader panicked".to_string())?
}

// ------------------------------------------------------------- entrypoints

pub fn server(args: &[String]) {
    crate::measurement::emit_marker();
    if !crate::measurement::enabled() {
        fail("socks-server requires NEKO_MEASUREMENT=1 (research scope, D071)");
    }
    let port: u16 = option(args, "--port")
        .and_then(|v| v.parse().ok())
        .unwrap_or(40086);
    if !(40080..=40100).contains(&port) {
        fail("port outside 40080-40100");
    }
    let bind = option(args, "--bind").unwrap_or_else(|| format!("0.0.0.0:{port}"));
    let bind: SocketAddr = bind.parse().unwrap_or_else(|_| fail("invalid --bind"));
    let peer_key = hex_decode(
        &option(args, "--client-key").unwrap_or_else(|| fail("--client-key is required")),
    )
    .unwrap_or_else(|e| fail(&e));
    let id_path = identity_path(args);
    let whitelist = build_whitelist(args);

    let listener = TcpListener::bind(bind).unwrap_or_else(|e| fail(&format!("bind: {e}")));
    let live = Arc::new((Mutex::new(0_usize), Condvar::new()));
    let allowed = Arc::new(AtomicU64::new(0));
    let denied = Arc::new(AtomicU64::new(0));

    println!(
        "{{\"role\":\"socks-server\",\"listen\":\"{}\",\"allow_entries\":{},\"measurement_mode\":true}}",
        bind,
        whitelist.len()
    );

    loop {
        let (mut socket, peer) = match listener.accept() {
            Ok(v) => v,
            Err(_) => continue,
        };
        crate::bulk::apply_f1_nodelay(&socket);
        // D072 cap: at capacity the connection is refused outright
        // (reject-and-continue, D019 F1 form).
        {
            let mut guard = live.0.lock().unwrap();
            if *guard >= MAX_CONNS {
                denied.fetch_add(1, Ordering::Relaxed);
                drop(guard);
                let _ = socket.shutdown(Shutdown::Both);
                continue;
            }
            *guard += 1;
        }
        let live = Arc::clone(&live);
        let allowed = Arc::clone(&allowed);
        let denied = Arc::clone(&denied);
        let whitelist = whitelist.clone();
        let peer_key = peer_key.clone();
        let id_path = id_path.clone();
        thread::spawn(move || {
            let id = read_identity_at(&id_path);
            let mut preauth = crate::preauth::ListenerAdmission::new();
            let mut admission = match preauth.admit_carrier(crate::preauth::CarrierKind::Tcp, peer)
            {
                Ok(ticket) => ticket,
                Err(_) => {
                    eprintln!("neko: pre-auth admission rejected (connection closed)");
                    let mut guard = live.0.lock().unwrap();
                    *guard -= 1;
                    live.1.notify_all();
                    return;
                }
            };
            let session =
                match server_handshake(&mut socket, &id, &peer_key, &mut preauth, &mut admission) {
                    Ok(s) => s,
                    Err(reason) => {
                        preauth.release(admission);
                        eprintln!("neko: {reason} (connection closed)");
                        let mut guard = live.0.lock().unwrap();
                        *guard -= 1;
                        live.1.notify_all();
                        return;
                    }
                };
            preauth.release(admission);
            let outcome = serve_connection(socket, session, &whitelist, &allowed, &denied);
            if let Err(reason) = &outcome {
                eprintln!("neko: socks serve error: {reason}");
            }
            let mut guard = live.0.lock().unwrap();
            *guard -= 1;
            live.1.notify_all();
        });
    }
}

fn serve_connection(
    neko: TcpStream,
    session: SecureSession,
    whitelist: &HashSet<SocketAddr>,
    allowed: &AtomicU64,
    denied: &AtomicU64,
) -> Result<(), String> {
    let runtime =
        // D068 exception constructor: the 2 GiB total ceiling is legal only
        // inside the measurement axis (NEKO_MEASUREMENT=1 gates this whole
        // command).
        SessionRuntime::new_measurement(SESSION, limits(), 0)
            .map_err(|_| "runtime rejected".to_string())?;
    let mut shared = Shared {
        neko,
        session,
        runtime,
        now_ms: 0,
    };
    shared.open_stream()?;
    let shared = Arc::new(Mutex::new(shared));
    let mut reader = shared
        .lock()
        .unwrap()
        .neko
        .try_clone()
        .map_err(|e| e.to_string())?;
    reader
        .set_read_timeout(Some(EXCHANGE_DEADLINE))
        .map_err(|e| e.to_string())?;

    // CONNECT request: [TAG_CONNECT][ip][be port]. Ack it, then decide.
    let target = loop {
        match read_inbound(&mut reader, &shared)? {
            Inbound::Payload { data, fin: false } => {
                if data.len() < 1 + 4 + 2 || data[0] != TAG_CONNECT {
                    return Err("malformed connect request".into());
                }
                match decode_connect(&data[1..]) {
                    Some(t) => break t,
                    None => return Err("malformed connect target".into()),
                }
            }
            Inbound::Ack { .. } => continue,
            _ => return Err("unexpected inbound during connect".into()),
        }
    };

    // Whitelist gate. The target itself never reaches any log line — only
    // the counters do.
    if !permits(whitelist, target) {
        denied.fetch_add(1, Ordering::Relaxed);
        let mut g = shared.lock().unwrap();
        let (offset, len) = g.send_record(&[TAG_CONNECT_REPLY, 0x02])?;
        drop(g);
        wait_ack_local(&shared, offset, len)?;
        return Ok(());
    }
    allowed.fetch_add(1, Ordering::Relaxed);

    let (code, dialed) = match TcpStream::connect_timeout(&target, DIAL_TIMEOUT) {
        Ok(stream) => (0x00_u8, Some(stream)),
        Err(_) => (0x01, None),
    };
    {
        let mut g = shared.lock().unwrap();
        let (offset, len) = g.send_record(&[TAG_CONNECT_REPLY, code])?;
        drop(g);
        wait_ack_local(&shared, offset, len)?;
    }
    let outbound = dialed;
    let Some(outbound) = outbound else {
        return Ok(());
    };
    crate::bulk::apply_f1_nodelay(&outbound);

    // Reuse the established session state for the pump.
    let Shared {
        neko,
        session,
        runtime,
        now_ms,
    } = Arc::try_unwrap(shared)
        .map_err(|_| "shared session leaked".to_string())?
        .into_inner()
        .unwrap();
    let _ = now_ms;
    pump(outbound, neko, session, runtime)
}

fn wait_ack_local(shared: &Arc<Mutex<Shared>>, offset: u64, len: usize) -> Result<(), String> {
    let deadline = Instant::now() + EXCHANGE_DEADLINE;
    // The downloader half does not exist yet in the connect phase — drain
    // the ack inline from the reader clone.
    let mut reader = shared
        .lock()
        .unwrap()
        .neko
        .try_clone()
        .map_err(|e| e.to_string())?;
    reader
        .set_read_timeout(Some(EXCHANGE_DEADLINE))
        .map_err(|e| e.to_string())?;
    loop {
        if Instant::now() >= deadline {
            return Err("ack deadline elapsed".into());
        }
        match read_inbound(&mut reader, shared)? {
            Inbound::Ack { offset: a, len: l } if a == offset && l == len => {
                let mut g = shared.lock().unwrap();
                return g.release(offset, len);
            }
            Inbound::Ack { .. } => continue,
            Inbound::Payload { .. } => continue,
        }
    }
}

fn decode_connect(buf: &[u8]) -> Option<SocketAddr> {
    match buf.len() {
        6 => {
            let mut ip = [0_u8; 4];
            ip.copy_from_slice(&buf[..4]);
            Some(SocketAddr::V4(SocketAddrV4::new(
                ip.into(),
                u16::from_be_bytes([buf[4], buf[5]]),
            )))
        }
        18 => {
            let mut ip = [0_u8; 16];
            ip.copy_from_slice(&buf[..16]);
            Some(SocketAddr::V6(SocketAddrV6::new(
                ip.into(),
                u16::from_be_bytes([buf[16], buf[17]]),
                0,
                0,
            )))
        }
        _ => None,
    }
}

pub fn client(args: &[String]) {
    crate::measurement::emit_marker();
    if !crate::measurement::enabled() {
        fail("socks-client requires NEKO_MEASUREMENT=1 (research scope, D071)");
    }
    let listen_port: u16 = option(args, "--listen")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1080);
    // 127.0.0.1-only entry (D071): the bind call itself refuses anything
    // non-loopback — there is no flag to widen it.
    let listener = TcpListener::bind(("127.0.0.1", listen_port))
        .unwrap_or_else(|e| fail(&format!("bind: {e}")));
    let server_addr: SocketAddr = option(args, "--addr")
        .unwrap_or_else(|| fail("--addr (neko socks-server address) is required"))
        .parse()
        .unwrap_or_else(|_| fail("invalid --addr"));
    let server_key = hex_decode(
        &option(args, "--server-key").unwrap_or_else(|| fail("--server-key is required")),
    )
    .unwrap_or_else(|e| fail(&e));
    let id_path = identity_path(args);

    println!(
        "{{\"role\":\"socks-client\",\"listen\":\"127.0.0.1:{listen_port}\",\"server\":\"redacted\",\"measurement_mode\":true}}"
    );

    loop {
        let (local, _) = match listener.accept() {
            Ok(v) => v,
            Err(_) => continue,
        };
        crate::bulk::apply_f1_nodelay(&local);
        let id_path = id_path.clone();
        let server_key = server_key.clone();
        thread::spawn(move || {
            let id = read_identity_at(&id_path);
            let _ = client_connection(local, server_addr, &id, &server_key);
        });
    }
}

fn client_connection(
    mut local: TcpStream,
    server_addr: SocketAddr,
    id: &LocalIdentity,
    server_key: &[u8],
) -> Result<(), String> {
    local
        .set_read_timeout(Some(EXCHANGE_DEADLINE))
        .map_err(|e| e.to_string())?;

    // 1. SOCKS5 greeting — method 0x00 only.
    let greeting = read_socks_greeting(&mut local)?;
    match parse_greeting(&greeting) {
        Ok(()) => local.write_all(&[5, 0]).map_err(|e| e.to_string())?,
        Err(SocksError::NotNoAuth) => {
            let _ = local.write_all(&[5, 0xFF]);
            return Err("client demands auth the entry does not offer".into());
        }
        Err(_) => return Err("malformed greeting".into()),
    }

    // 2. SOCKS5 CONNECT — IPv4/IPv6 literals only (domain → 0x08).
    let request = read_socks_connect(&mut local)?;
    let target = match parse_connect(&request) {
        Ok(t) => t,
        Err(SocksError::AddressUnsupported) => {
            let _ = local.write_all(&socks_reply(0x08));
            return Err("domain targets are deferred (D071)".into());
        }
        Err(SocksError::CommandUnsupported) => {
            let _ = local.write_all(&socks_reply(0x07));
            return Err("only CONNECT is implemented".into());
        }
        Err(_) => {
            let _ = local.write_all(&socks_reply(0x01));
            return Err("malformed connect".into());
        }
    };

    // 3. One neko session per SOCKS connection (D071 form).
    let mut neko = TcpStream::connect(server_addr).map_err(|e| format!("connect: {e}"))?;
    crate::bulk::apply_f1_nodelay(&neko);
    let session = client_handshake(&mut neko, id, server_key)?;
    let runtime =
        // D068 exception constructor: the 2 GiB total ceiling is legal only
        // inside the measurement axis (NEKO_MEASUREMENT=1 gates this whole
        // command).
        SessionRuntime::new_measurement(SESSION, limits(), 0)
            .map_err(|_| "runtime rejected".to_string())?;
    let mut shared = Shared {
        neko,
        session,
        runtime,
        now_ms: 0,
    };
    shared.open_stream()?;
    let shared = Arc::new(Mutex::new(shared));

    // 4. CONNECT relay request.
    let mut req = Vec::with_capacity(1 + 16 + 2);
    req.push(TAG_CONNECT);
    match target {
        SocketAddr::V4(v4) => req.extend_from_slice(&v4.ip().octets()),
        SocketAddr::V6(v6) => req.extend_from_slice(&v6.ip().octets()),
    }
    req.extend_from_slice(&target.port().to_be_bytes());
    let (offset, len) = {
        let mut g = shared.lock().unwrap();
        g.send_record(&req)?
    };

    // 5. Drain until the CONNECT reply (acking the request on the way).
    let mut reader = shared
        .lock()
        .unwrap()
        .neko
        .try_clone()
        .map_err(|e| e.to_string())?;
    reader
        .set_read_timeout(Some(EXCHANGE_DEADLINE))
        .map_err(|e| e.to_string())?;
    let mut request_acked = false;
    let reply = loop {
        match read_inbound(&mut reader, &shared)? {
            Inbound::Ack { offset: a, len: l } if a == offset && l == len => {
                let mut g = shared.lock().unwrap();
                g.release(offset, len)?;
                request_acked = true;
            }
            Inbound::Payload { data, fin: false }
                if data.len() == 2 && data[0] == TAG_CONNECT_REPLY =>
            {
                if !request_acked {
                    return Err("connect reply before request ack".into());
                }
                break data[1];
            }
            _ => return Err("unexpected inbound before connect reply".into()),
        }
    };
    if reply != 0x00 {
        let _ = local.write_all(&socks_reply(reply));
        return Err("remote refused connect".into());
    }
    local
        .write_all(&socks_reply(0x00))
        .map_err(|e| e.to_string())?;

    // 6. Hand the established state to the pump.
    let Shared {
        neko,
        session,
        runtime,
        now_ms,
    } = Arc::try_unwrap(shared)
        .map_err(|_| "shared session leaked".to_string())?
        .into_inner()
        .unwrap();
    let _ = now_ms;
    local.set_read_timeout(None).ok();
    pump(local, neko, session, runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(o: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(o.into(), port))
    }

    #[test]
    fn greeting_requires_version_five_and_no_auth_method() {
        assert!(parse_greeting(&[5, 1, 0]).is_ok());
        assert_eq!(parse_greeting(&[5, 2, 1, 2]), Err(SocksError::NotNoAuth));
        assert_eq!(parse_greeting(&[4, 1, 0]), Err(SocksError::Malformed));
        assert_eq!(parse_greeting(&[5, 0]), Err(SocksError::Malformed));
        // NMETHODS must match the buffer length exactly.
        assert_eq!(parse_greeting(&[5, 2, 0]), Err(SocksError::Malformed));
    }

    #[test]
    fn connect_parses_ipv4_and_ipv6_literals_only() {
        let mut msg = vec![5, 1, 0, 1];
        msg.extend_from_slice(&[203, 0, 113, 7]);
        msg.extend_from_slice(&80u16.to_be_bytes());
        assert_eq!(parse_connect(&msg).unwrap(), v4([203, 0, 113, 7], 80));

        let mut msg = vec![5, 1, 0, 4];
        msg.extend_from_slice(&[0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        msg.extend_from_slice(&443u16.to_be_bytes());
        let parsed = parse_connect(&msg).unwrap();
        assert_eq!(parsed.port(), 443);
        assert!(parsed.is_ipv6());

        // BIND/UDP ASSOCIATE → command unsupported; domain → deferred.
        assert_eq!(
            parse_connect(&[5, 2, 0, 1, 1, 1, 1, 1, 0, 80]),
            Err(SocksError::CommandUnsupported)
        );
        assert_eq!(
            parse_connect(&[
                5, 1, 0, 3, 9, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm', 0,
                80
            ]),
            Err(SocksError::AddressUnsupported)
        );
        // Truncated and trailing-garbage forms are malformed.
        assert_eq!(
            parse_connect(&[5, 1, 0, 1, 1, 1, 1]),
            Err(SocksError::Malformed)
        );
        assert_eq!(
            parse_connect(&[5, 1, 0, 1, 1, 1, 1, 1, 0, 80, 0xFF]),
            Err(SocksError::Malformed)
        );
    }

    #[test]
    fn parser_survives_arbitrary_bytes_without_panicking() {
        // fuzz-smoke: every 0..=255 single byte, then pseudo-random bursts
        // from a deterministic LCG — no panic, no slice overrun, no
        // unbounded allocation.
        let mut lcg: u64 = 0x853c49e6748fea9b;
        let mut noise = |n: usize| -> Vec<u8> {
            (0..n)
                .map(|_| {
                    lcg = lcg
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (lcg >> 33) as u8
                })
                .collect()
        };
        for b in 0..=255_u8 {
            let _ = parse_greeting(&[b]);
            let _ = parse_connect(&[b, b, b, b, b, b, b, b, b, b]);
        }
        for n in [1, 2, 3, 7, 15, 64, 262, 263, 511, 512, 513] {
            let buf = noise(n);
            let _ = parse_greeting(&buf);
            let _ = parse_connect(&buf);
        }
    }

    #[test]
    fn internal_ranges_are_refused_even_when_whitelisted() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.1.1",
            "172.16.0.1",
            "172.31.255.255",
            "169.254.1.1",
            "0.0.0.1",
            "100.64.0.1",
        ] {
            let addr: SocketAddr = format!("{ip}:443").parse().unwrap();
            assert!(is_internal(addr.ip()), "{ip} must be internal");
        }
        for ip in ["203.0.113.10", "198.51.100.7"] {
            let addr: SocketAddr = format!("{ip}:443").parse().unwrap();
            assert!(!is_internal(addr.ip()));
        }
        let v6_local: SocketAddr = "[::1]:443".parse().unwrap();
        assert!(is_internal(v6_local.ip()));
        let v6_ula: SocketAddr = "[fd00::1]:443".parse().unwrap();
        assert!(is_internal(v6_ula.ip()));

        // permits(): a public IP in the set passes; the same IP removed, or
        // an internal IP force-fed into the set, fails.
        let public: SocketAddr = "203.0.113.10:443".parse().unwrap();
        let mut set = HashSet::new();
        set.insert(public);
        assert!(permits(&set, public));
        let internal: SocketAddr = "127.0.0.1:443".parse().unwrap();
        set.insert(internal);
        assert!(
            !permits(&set, internal),
            "internal must never pass, even listed"
        );
    }

    #[test]
    fn pump_chunk_and_caps_are_pinned() {
        // D071/D072 shape pins: parser cap, pump budget, concurrency cap.
        assert_eq!(SOCKS_CAP, 512);
        assert_eq!(PUMP_CHUNK, 1024);
        assert_eq!(MAX_CONNS, 16);
        assert_eq!(SESSION_TOTAL_BYTES, 2 * 1024 * 1024 * 1024);
        // The runtime's declared queue budget is the D072 64 KiB cap.
        let l = limits();
        assert_eq!(l.max_queue_bytes, 64 * 1024);
        assert_eq!(l.max_total_bytes, SESSION_TOTAL_BYTES);
        // A pump record (tag + chunk) fits the record ceiling with margin.
        assert!(PUMP_CHUNK < l.max_record_bytes);
    }
}
