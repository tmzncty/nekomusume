// Byte-counting sink. args: PORT WINDOW_MS [MODE=tcp]
use std::io::Read;
use std::net::{TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let port: u16 = std::env::args().nth(1).unwrap().parse().unwrap();
    let window_ms: u64 = std::env::args().nth(2).unwrap().parse().unwrap();
    let mode = std::env::args().nth(3).unwrap_or_else(|| "tcp".into());
    let window = Duration::from_millis(window_ms);
    let deadline = Instant::now() + window + Duration::from_secs(3);
    let total = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let start = Arc::new(Mutex::new(None::<Instant>));

    if mode == "udp" {
        let s = UdpSocket::bind(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        let total = total.clone();
        let start = start.clone();
        let stop2 = stop.clone();
        let s2 = s.try_clone().unwrap();
        let h = std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            loop {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                match s2.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let mut g = start.lock().unwrap();
                        if g.is_none() {
                            *g = Some(Instant::now());
                        } else if g.unwrap().elapsed() > window {
                            drop(g);
                            stop2.store(true, Ordering::Relaxed);
                            break;
                        }
                        drop(g);
                        total.fetch_add(n as u64, Ordering::Relaxed);
                    }
                    Err(_) => {}
                }
            }
        });
        while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        stop.store(true, Ordering::Relaxed);
        let _ = h.join();
    } else {
        let l = TcpListener::bind(("127.0.0.1", port)).unwrap();
        l.set_nonblocking(true).unwrap();
        let mut threads = Vec::new();
        while Instant::now() < deadline {
            let over = {
                let g = start.lock().unwrap();
                g.map(|s| s.elapsed() > window).unwrap_or(false)
            };
            if over {
                break;
            }
            if let Ok((s, _)) = l.accept() {
                let total = total.clone();
                let start = start.clone();
                threads.push(std::thread::spawn(move || {
                    let mut s = s;
                    let mut buf = [0u8; 1 << 16];
                    loop {
                        match s.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let mut g = start.lock().unwrap();
                                if g.is_none() {
                                    *g = Some(Instant::now());
                                } else if g.unwrap().elapsed() > window {
                                    break;
                                }
                                drop(g);
                                total.fetch_add(n as u64, Ordering::Relaxed);
                            }
                        }
                    }
                }));
            } else {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        for t in threads {
            let _ = t.join();
        }
    }
    println!("{}", total.load(Ordering::Relaxed));
}
