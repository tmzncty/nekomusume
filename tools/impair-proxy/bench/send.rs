// Saturating sender. args: PORT WINDOW_MS MSS MODE
use std::io::Write;
use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};

fn main() {
    let port: u16 = std::env::args().nth(1).unwrap().parse().unwrap();
    let window_ms: u64 = std::env::args().nth(2).unwrap().parse().unwrap();
    let mss: usize = std::env::args().nth(3).unwrap().parse().unwrap();
    let mode = std::env::args().nth(4).unwrap_or_else(|| "tcp".into());
    let deadline = Instant::now() + Duration::from_millis(window_ms);
    let payload = vec![0u8; mss];
    if mode == "udp" {
        let s = UdpSocket::bind("127.0.0.1:0").unwrap();
        while Instant::now() < deadline {
            for _ in 0..200 {
                if s.send_to(&payload, ("127.0.0.1", port)).is_err() {
                    return;
                }
            }
        }
    } else {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_nodelay(true).unwrap();
        while Instant::now() < deadline {
            for _ in 0..64 {
                if s.write_all(&payload).is_err() {
                    return;
                }
            }
        }
    }
}
