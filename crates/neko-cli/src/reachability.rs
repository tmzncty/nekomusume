//! Local-only reachability artifact support. This module deliberately refuses
//! non-loopback targets and has no raw/privileged protocol implementation.
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

pub const SCHEMA_VERSION: &str = "reachability-matrix.v1";
pub const MAX_TIMEOUT_MS: u64 = 5_000;
pub const MAX_PAYLOAD_BYTES: usize = 1_200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transport {
    Tcp,
    Udp,
}
impl Transport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpVersion {
    V4,
    V6,
}
impl IpVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V4 => "ipv4",
            Self::V6 => "ipv6",
        }
    }
}

pub fn validate(
    target: SocketAddr,
    version: IpVersion,
    timeout_ms: u64,
    payload: usize,
) -> Result<(), &'static str> {
    if !target.ip().is_loopback() {
        return Err("target must be loopback; public/WAN probing is disabled");
    }
    if matches!(
        (target.ip(), version),
        (IpAddr::V4(_), IpVersion::V6) | (IpAddr::V6(_), IpVersion::V4)
    ) {
        return Err("IP version does not match target");
    }
    if target.port() == 0 {
        return Err("target port must be non-zero");
    }
    if timeout_ms == 0 || timeout_ms > MAX_TIMEOUT_MS {
        return Err("timeout must be 1-5000 ms");
    }
    if payload == 0 || payload > MAX_PAYLOAD_BYTES {
        return Err("payload must be 1-1200 bytes");
    }
    Ok(())
}

pub fn run(
    transport: Transport,
    version: IpVersion,
    target: SocketAddr,
    timeout_ms: u64,
    payload: usize,
) -> String {
    let timeout = Duration::from_millis(timeout_ms);
    let (reachable, rtt_ms, bytes, error) = match validate(target, version, timeout_ms, payload) {
        Err(e) => (false, None, 0, Some(e.to_string())),
        Ok(()) => {
            let start = Instant::now();
            let result = match transport {
                Transport::Tcp => TcpStream::connect_timeout(&target, timeout).map(|_| payload),
                Transport::Udp => UdpSocket::bind(if target.is_ipv4() {
                    "0.0.0.0:0"
                } else {
                    "[::]:0"
                })
                .and_then(|s| {
                    s.set_read_timeout(Some(timeout))?;
                    s.connect(target)?;
                    s.send(&vec![0x4e; payload])?;
                    let mut response = [0_u8; MAX_PAYLOAD_BYTES];
                    s.recv(&mut response)
                }),
            };
            match result {
                Ok(n) => (true, Some(start.elapsed().as_secs_f64() * 1000.0), n, None),
                Err(e) => (false, None, 0, Some(e.kind().to_string())),
            }
        }
    };
    let target_json = json_string(&target.to_string());
    let error_json = error.as_deref().map_or("null".to_owned(), json_string);
    let rtt_json = rtt_ms.map_or("null".to_owned(), |v| format!("{v:.3}"));
    format!(
        "{{\"schema_version\":\"{SCHEMA_VERSION}\",\"observed_at_unix_ms\":null,\"scope\":\"local-loopback\",\"metadata\":{{\"privileged\":false,\"raw_protocol\":false,\"third_party_scan\":false}},\"cases\":[{{\"transport\":\"{}\",\"ip_version\":\"{}\",\"target\":{},\"reachable\":{},\"rtt_ms\":{},\"payload_bytes\":{},\"error\":{}}}]}}",
        transport.as_str(),
        version.as_str(),
        target_json,
        reachable,
        rtt_json,
        bytes,
        error_json
    )
}
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wan_before_socket() {
        assert_eq!(
            validate("192.0.2.1:9".parse().unwrap(), IpVersion::V4, 100, 1),
            Err("target must be loopback; public/WAN probing is disabled")
        );
    }
    #[test]
    fn rejects_mismatched_family_and_bounds() {
        assert!(validate("127.0.0.1:9".parse().unwrap(), IpVersion::V6, 100, 1).is_err());
        assert!(validate("127.0.0.1:9".parse().unwrap(), IpVersion::V4, 5001, 1).is_err());
        assert!(validate("127.0.0.1:9".parse().unwrap(), IpVersion::V4, 100, 1201).is_err());
    }
    #[test]
    fn artifact_is_stable_except_explicit_observation_time() {
        let s = run(
            Transport::Tcp,
            IpVersion::V4,
            "127.0.0.1:9".parse().unwrap(),
            1,
            1,
        );
        assert!(s.contains("reachability-matrix.v1"));
        assert!(s.contains("local-loopback"));
        assert!(s.contains("\"reachable\":false"));
    }
}
