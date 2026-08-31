use neko_crypto::{
    LocalIdentity, RecordContext, ResponderHandshake, TrustPolicy, TrustRecord, TrustStatus,
};
use neko_wire::{NEGOTIATION_VERSION, NegotiationRole, VersionNegotiator};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

fn identity(bin: &str, name: &str) -> (PathBuf, String) {
    let path = std::env::temp_dir().join(format!("neko-multistream-{name}-{}", std::process::id()));
    let out = Command::new(bin)
        .args(["keygen", "--identity", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let public = String::from_utf8_lossy(&out.stdout)
        .trim()
        .strip_prefix("client_public_key=")
        .unwrap()
        .to_owned();
    (path, public)
}

#[test]
fn bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity, server_key) = identity(bin, "server");
    let (client_identity, client_key) = identity(bin, "client");
    let server = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "server",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "3",
            "--records",
            "4",
            "--bytes",
            "17",
            "--session-window",
            "17",
            "--stream-window",
            "17",
            "--identity",
            server_identity.to_str().unwrap(),
            "--client-key",
            &client_key,
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(50));
    let client = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "client",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "3",
            "--records",
            "4",
            "--bytes",
            "17",
            "--session-window",
            "17",
            "--stream-window",
            "17",
            "--identity",
            client_identity.to_str().unwrap(),
            "--server-key",
            &server_key,
        ])
        .output()
        .unwrap();
    assert!(
        client.status.success(),
        "{}",
        String::from_utf8_lossy(&client.stderr)
    );
    let status = server.wait_with_output().unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let evidence = String::from_utf8(client.stdout).unwrap();
    assert!(evidence.contains("\"ok\":true"));
    assert!(evidence.contains("\"streams\":3"));
    assert!(evidence.contains("\"records\":12"));
    assert!(evidence.contains("\"payload_bytes\":204"));
    fn counter(evidence: &str, name: &str) -> u64 {
        evidence
            .split(&format!("\"{name}\":"))
            .nth(1)
            .unwrap()
            .split([',', '}'])
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }
    assert!(counter(&evidence, "window_exhausted") > 0);
    assert_eq!(counter(&evidence, "ack_released"), 12);
    assert_eq!(counter(&evidence, "resumed"), 12);
    let _ = fs::remove_file(server_identity);
    let _ = fs::remove_file(client_identity);
}

#[test]
fn unauthorized_client_is_rejected_by_allowlist() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity, server_key) = identity(bin, "negative-server");
    let (allowed_identity, allowed_key) = identity(bin, "allowed");
    let (unauthorized_identity, _) = identity(bin, "unauthorized");
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let server = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "server",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            server_identity.to_str().unwrap(),
            "--client-key",
            &allowed_key,
        ])
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(50));
    let client = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "client",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            unauthorized_identity.to_str().unwrap(),
            "--server-key",
            &server_key,
        ])
        .output()
        .unwrap();
    assert!(!client.status.success());
    let status = server.wait_with_output().unwrap();
    assert!(!status.status.success());
    for path in [server_identity, allowed_identity, unauthorized_identity] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn multistream_rejects_unbounded_payload_before_connecting() {
    let out = Command::new(env!("CARGO_BIN_EXE_neko-cli"))
        .args([
            "multistream",
            "--mode",
            "client",
            "--streams",
            "17",
            "--addr",
            "192.0.2.1:40082",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("streams outside"));
}

const DOMAIN: &[u8] = b"nekomusume/neko-cli/multistream/v1";
const SCOPE: &[u8] = b"neko-cli/multistream";

fn decode_hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

fn load_identity(path: &Path) -> LocalIdentity {
    let text = fs::read_to_string(path).unwrap();
    let (private, public) = text.trim().split_once(':').unwrap();
    LocalIdentity::from_keypair(&decode_hex(private), &decode_hex(public)).unwrap()
}

fn frame_read(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length)?;
    let mut frame = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

fn frame_write(stream: &mut TcpStream, frame: &[u8]) {
    stream
        .write_all(&(frame.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(frame).unwrap();
}

fn multistream_client(
    bin: &str,
    port: u16,
    identity: &Path,
    server_key: &str,
) -> std::process::Output {
    Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "client",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            identity.to_str().unwrap(),
            "--server-key",
            server_key,
        ])
        .output()
        .unwrap()
}

fn assert_uniform_handshake_rejection(output: &std::process::Output) {
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "neko: handshake rejected\n"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\":true"));
    assert!(!stdout.contains("records"));
}

#[test]
fn executable_rejects_one_byte_different_negotiation_binding_before_session_data() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity_path, server_key) = identity(bin, "binding-server");
    let (client_identity_path, client_key) = identity(bin, "binding-client");
    let server_identity = load_identity(&server_identity_path);
    let client_key = decode_hex(&client_key);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let peer = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let hello = frame_read(&mut socket).unwrap();
        let mut negotiation =
            VersionNegotiator::new(NegotiationRole::Server, &[NEGOTIATION_VERSION]).unwrap();
        let response = negotiation.server_accept_hello(&hello).unwrap();
        frame_write(&mut socket, &response);
        let mut binding = negotiation
            .authenticated_binding()
            .unwrap()
            .as_bytes()
            .to_vec();
        binding[0] ^= 1;
        let policy = TrustPolicy::new(vec![TrustRecord {
            version: 1,
            public_key: client_key,
            scope: SCOPE.to_vec(),
            status: TrustStatus::Active,
        }]);
        let handshake = ResponderHandshake::new_with_prologue_binding(
            &server_identity,
            policy,
            DOMAIN,
            &binding,
        )
        .unwrap();
        let first_noise_frame = frame_read(&mut socket).unwrap();
        assert!(
            handshake
                .receive_first(
                    &first_noise_frame,
                    RecordContext {
                        delivery_epoch: 1,
                        key_phase: 0,
                        path_generation: 1,
                        stream_id: 0,
                        direction: 0,
                    },
                )
                .is_err(),
            "a transcript-binding mismatch must fail before SecureSession exists"
        );
        // `receive_first` failed while consuming the first Noise frame, before it
        // could construct a SecureSession or read a ProcessMessage. Closing the
        // peer makes the executable surface its uniform handshake rejection.
    });

    let output = multistream_client(bin, port, &client_identity_path, &server_key);
    assert_uniform_handshake_rejection(&output);
    peer.join().unwrap();
    let _ = fs::remove_file(server_identity_path);
    let _ = fs::remove_file(client_identity_path);
}

#[test]
fn executable_rejects_unsupported_only_negotiation_before_noise_or_data() {
    let bin = env!("CARGO_BIN_EXE_neko-cli");
    let (server_identity_path, _) = identity(bin, "unsupported-server");
    let (client_identity_path, client_key) = identity(bin, "unsupported-client-key");
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reservation.local_addr().unwrap().port();
    drop(reservation);
    let server = Command::new(bin)
        .args([
            "multistream",
            "--mode",
            "server",
            "--addr",
            &format!("127.0.0.1:{port}"),
            "--streams",
            "1",
            "--records",
            "1",
            "--bytes",
            "1",
            "--identity",
            server_identity_path.to_str().unwrap(),
            "--client-key",
            &client_key,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(50));
    let mut socket = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    // A syntactically valid N1 hello whose only offer is unsupported by this executable.
    frame_write(&mut socket, &[b'N', b'V', 1, 1, 0, 2]);
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(
        socket.read(&mut byte).unwrap(),
        0,
        "server entered Noise/data admission"
    );
    let output = server.wait_with_output().unwrap();
    assert_uniform_handshake_rejection(&output);
    let _ = fs::remove_file(server_identity_path);
    let _ = fs::remove_file(client_identity_path);
}
