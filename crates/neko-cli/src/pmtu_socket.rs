//! CLI-only adapter that makes a UDP socket fit for PLPMTUD probing (D067).
//!
//! Linux fragments oversized UDP datagrams by default. Once an ICMP
//! frag-needed or PTB message has populated the kernel PMTU cache, a probe
//! larger than the path MTU is fragmented at the source and delivered, so it
//! falsely succeeds (D067, measured). `IP(V6)_PMTUDISC_PROBE` sets DF, never
//! fragments and ignores that cache. An oversize send then fails locally with
//! `EMSGSIZE` or is lost on the path, which is what RFC 8899 PLPMTUD needs.
//!
//! This module is the only user of `rustix`. It uses exactly
//! `set_ip_mtu_discover`, `set_ipv6_mtu_discover`, `ip_mtu` and `ipv6_mtu`;
//! the tests also use the matching getters to read the option back. It is
//! used only behind the explicit opt-in PLPMTUD path. On platforms other
//! than Linux it returns `Unsupported`, and callers must fail closed instead
//! of probing on a fragmentable socket.
#![cfg_attr(not(test), allow(dead_code))] // consumed by R slice 3 (--plpmtud)

use std::io;
use std::net::UdpSocket;

/// Put `socket` in "DF, never fragment, ignore the kernel PMTU cache" mode for
/// its own address family.
#[cfg(target_os = "linux")]
pub(crate) fn set_probe_df(socket: &UdpSocket) -> io::Result<()> {
    let ipv4 = socket.local_addr()?.is_ipv4();
    set_probe_df_for_family(socket, ipv4)
}

/// Family-explicit core of [`set_probe_df`]; errors from the kernel are
/// returned unchanged so the opt-in path can fail closed.
#[cfg(target_os = "linux")]
fn set_probe_df_for_family(socket: &UdpSocket, ipv4: bool) -> io::Result<()> {
    use rustix::net::sockopt::{
        Ipv4PathMtuDiscovery, Ipv6PathMtuDiscovery, set_ip_mtu_discover, set_ipv6_mtu_discover,
    };
    let result = if ipv4 {
        set_ip_mtu_discover(socket, Ipv4PathMtuDiscovery::PROBE)
    } else {
        set_ipv6_mtu_discover(socket, Ipv6PathMtuDiscovery::PROBE)
    };
    result.map_err(io::Error::from)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn set_probe_df(_socket: &UdpSocket) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "PLPMTUD probe socket (DF, no fragmentation) is only implemented on Linux",
    ))
}

/// The kernel's current MTU for a *connected* socket's route. Diagnostic only:
/// PLPMTUD never takes its decision from this value.
#[cfg(target_os = "linux")]
pub(crate) fn kernel_path_mtu(socket: &UdpSocket) -> io::Result<u32> {
    use rustix::net::sockopt::{ip_mtu, ipv6_mtu};
    let result = if socket.local_addr()?.is_ipv4() {
        ip_mtu(socket)
    } else {
        ipv6_mtu(socket)
    };
    result.map_err(io::Error::from)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn kernel_path_mtu(_socket: &UdpSocket) -> io::Result<u32> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "kernel path MTU query is only implemented on Linux",
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use rustix::net::sockopt::{
        Ipv4PathMtuDiscovery, Ipv6PathMtuDiscovery, ip_mtu_discover, ipv6_mtu_discover,
    };

    #[test]
    fn ipv4_probe_df_is_read_back_as_probe_and_default_is_not() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        // The default must differ; otherwise this test would prove nothing.
        assert_ne!(
            ip_mtu_discover(&socket).unwrap(),
            Ipv4PathMtuDiscovery::PROBE
        );
        set_probe_df(&socket).unwrap();
        assert_eq!(
            ip_mtu_discover(&socket).unwrap(),
            Ipv4PathMtuDiscovery::PROBE
        );
    }

    #[test]
    fn ipv6_probe_df_is_read_back_as_probe_and_default_is_not() {
        let socket = UdpSocket::bind("[::1]:0").unwrap();
        assert_ne!(
            ipv6_mtu_discover(&socket).unwrap(),
            Ipv6PathMtuDiscovery::PROBE
        );
        set_probe_df(&socket).unwrap();
        assert_eq!(
            ipv6_mtu_discover(&socket).unwrap(),
            Ipv6PathMtuDiscovery::PROBE
        );
    }

    #[test]
    fn setting_probe_df_leaves_the_other_family_option_untouched() {
        // The family dispatch must pick the socket's own family: a v6 socket
        // keeps its IPv4-level option at the default after set_probe_df.
        let socket = UdpSocket::bind("[::1]:0").unwrap();
        let v4_before = ip_mtu_discover(&socket).ok();
        set_probe_df(&socket).unwrap();
        assert_eq!(ip_mtu_discover(&socket).ok(), v4_before);
    }

    #[test]
    fn a_kernel_refusal_is_returned_not_swallowed() {
        // IPv6-level options on an IPv4 socket are refused by the kernel. The
        // error must reach the caller, because swallowing it would leave the
        // opt-in path probing on a fragmentable socket.
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        assert!(set_probe_df_for_family(&socket, false).is_err());
        assert_ne!(
            ip_mtu_discover(&socket).unwrap(),
            Ipv4PathMtuDiscovery::PROBE
        );
    }

    #[test]
    fn kernel_path_mtu_reports_the_loopback_route_for_a_connected_socket() {
        let peer = UdpSocket::bind("127.0.0.1:0").unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        // Unconnected: the kernel has no route to report.
        assert!(kernel_path_mtu(&socket).is_err());
        socket.connect(peer.local_addr().unwrap()).unwrap();
        let mtu = kernel_path_mtu(&socket).unwrap();
        assert!(mtu >= 1280, "loopback path MTU {mtu}");
    }
}
