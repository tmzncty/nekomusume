//! Bounded authenticated research probe runtime; never a proxy or tunnel.
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, TrustPolicy, TrustRecord,
    TrustStatus,
};
use std::{
    env, fs,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    time::{Duration, Instant},
};
const USAGE: &str = "Usage: neko <server|client|probe|lab|failover|keygen> [bounded options]

  --count N: bounded authenticated exchanges (1-64)\n\nBounded authenticated research probe only; no proxy/tunnel behavior.\n";
const MAX_PORT: u16 = 40100;
const MAX_BYTES: usize = neko_crypto::MAX_UNRELIABLE_DATAGRAM;
const MAX_DURATION: u64 = 30;
const DOMAIN: &[u8] = b"nekomusume-vps-probe";
fn json_mode(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
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
fn failover_gate(args: &[String]) {
    // WAN failover is intentionally a hard gate until a real dual-listener
    // runner has independent review. Keep argument validation bounded so a
    // future implementation cannot widen the experiment by accident.
    let count = exchange_count(args);
    let bytes = parse(args, "--bytes", Some("32"))
        .parse::<usize>()
        .unwrap_or_else(|_| fail("invalid bytes"));
    if bytes == 0 || bytes > MAX_BYTES {
        fail("bytes outside 1-1200");
    }
    let secs = parse(args, "--duration", Some("10"))
        .parse::<u64>()
        .unwrap_or_else(|_| fail("invalid duration"));
    if secs == 0 || secs > MAX_DURATION {
        fail("duration outside 1-30");
    }
    let udp_port = parse(args, "--udp-port", Some("40081"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid UDP port"));
    let tcp_port = parse(args, "--tcp-port", Some("40080"))
        .parse::<u16>()
        .unwrap_or_else(|_| fail("invalid TCP port"));
    if !(40080..=MAX_PORT).contains(&udp_port) || !(40080..=MAX_PORT).contains(&tcp_port) {
        fail("ports outside 40080-40100");
    }
    if !args.iter().any(|a| a == "--loopback-only") {
        fail("WAN failover runner is gated: use --loopback-only for the bounded simulator");
    }
    if json_mode(args) {
        println!(
            "{{\"ok\":true,\"gate\":\"failover-simulator\",\"wan\":false,\"count\":{},\"bytes\":{},\"duration_s\":{},\"udp_port\":{},\"tcp_port\":{}}}",
            count, bytes, secs, udp_port, tcp_port
        );
    } else {
        println!(
            "failover_gate_ok wan=false loopback_only=true count={} bytes={} duration_s={} udp_port={} tcp_port={}",
            count, bytes, secs, udp_port, tcp_port
        );
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

fn main() {
    let a: Vec<String> = env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("server") => server(&a),
        Some("client") | Some("probe") => client(&a),
        Some("lab") => lab(&a),
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
