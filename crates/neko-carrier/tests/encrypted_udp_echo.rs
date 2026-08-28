use neko_carrier::{UdpCarrier, UdpLimits, UdpLoopbackPair};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, PreauthBudget, PreauthLimits, RecordContext,
    ResponderHandshake, TrustPolicy, TrustRecord, TrustStatus,
};
use std::{
    thread,
    time::{Duration, Instant},
};

fn recv(endpoint: &impl UdpCarrier) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match endpoint.recv_datagram() {
            Ok(Some(v)) => return v,
            Err(neko_carrier::UdpError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(1))
            }
            other => panic!("receive failed: {other:?}"),
        }
    }
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

#[test]
fn authenticated_encrypted_loopback_echo_and_close() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    assert!(client.local_addr().unwrap().ip().is_loopback());
    assert!(server.local_addr().unwrap().ip().is_loopback());
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"echo", b"udp-loopback").unwrap();
    let first = ih.first_message().unwrap();
    let mut budget = PreauthBudget::new(PreauthLimits::default()).unwrap();
    client.send_datagram(&first).unwrap();
    let received = recv(&server);
    budget.charge_input(received.len()).unwrap();
    let rh = ResponderHandshake::new(&si, policy, b"udp-loopback").unwrap();
    let (response, mut ss) = rh.receive_first(&received, context()).unwrap();
    budget.charge_response(response.len()).unwrap();
    server.send_datagram(&response).unwrap();
    let mut cs = ih.finish(&recv(&client), context()).unwrap();
    let plaintext = b"meow over encrypted UDP";
    let encrypted = cs.seal(plaintext).unwrap();
    assert!(!encrypted.windows(plaintext.len()).any(|w| w == plaintext));
    client.send_datagram(&encrypted).unwrap();
    let opened = ss.open(&recv(&server)).unwrap();
    assert_eq!(opened, plaintext);
    let reply = ss.seal(&opened).unwrap();
    server.send_datagram(&reply).unwrap();
    assert_eq!(cs.open(&recv(&client)).unwrap(), plaintext);
    client.close().unwrap();
    client.close().unwrap();
    assert!(client.send_datagram(b"x").is_err());
}

#[test]
fn corrupted_ciphertext_produces_no_echo() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"echo", b"udp-loopback").unwrap();
    let first = ih.first_message().unwrap();
    client.send_datagram(&first).unwrap();
    let rh = ResponderHandshake::new(&si, policy, b"udp-loopback").unwrap();
    let (response, mut ss) = rh.receive_first(&recv(&server), context()).unwrap();
    server.send_datagram(&response).unwrap();
    let mut cs = ih.finish(&recv(&client), context()).unwrap();
    let mut bad = cs.seal(b"must not echo").unwrap();
    *bad.last_mut().unwrap() ^= 1;
    client.send_datagram(&bad).unwrap();
    assert!(ss.open(&recv(&server)).is_err());
    assert!(matches!(
        client.recv_datagram(),
        Err(neko_carrier::UdpError::WouldBlock)
    ));
}
