use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use neko_carrier::{DataId, FailoverController};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, ResumeBinding,
    ResumeGuard, TrustPolicy, TrustRecord, TrustStatus,
};
use neko_session::{
    InboundRecord, ProcessMessage, ResumeWireBinding, RuntimeEventKind, RuntimeLimits,
    RuntimeState, SessionId, SessionRuntime, StreamId,
};

const SESSION: SessionId = SessionId(7001);
const STREAM: StreamId = StreamId(1);

fn limits() -> RuntimeLimits {
    RuntimeLimits {
        max_streams: 1,
        max_queue_records: 8,
        max_queue_bytes: 128,
        max_total_bytes: 128,
        max_record_bytes: 32,
        max_session_window: 128,
        max_stream_window: 64,
        idle_timeout_ms: 1_000,
        close_timeout_ms: 20,
    }
}
fn context(generation: u64) -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: generation,
        stream_id: STREAM.0,
        direction: 0,
    }
}
fn fresh_resume(generation: u64) -> (ResumeGuard, Vec<u8>, ResumeWireBinding) {
    let client = LocalIdentity::generate().unwrap();
    let server = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client.public_key().to_vec(),
        scope: b"process-runner".to_vec(),
        status: TrustStatus::Active,
    }]);
    let original = ResumeBinding {
        session_id: SESSION.0,
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: generation - 1,
        expires_at_ms: 10_000,
        token: [9; 32],
    };
    let guard = ResumeGuard::new(client.public_key(), &original).unwrap();
    let claim_binding = ResumeBinding {
        path_generation: generation,
        ..original.clone()
    };
    let mut initiator = InitiatorHandshake::with_resume_binding(
        &client,
        server.public_key(),
        b"process-runner",
        b"loopback",
        &claim_binding,
    )
    .unwrap();
    let first = initiator.first_message().unwrap();
    let (response, _responder, peer, claim) = ResponderHandshake::new(&server, policy, b"loopback")
        .unwrap()
        .receive_first_with_resume(&first, context(generation))
        .unwrap();
    let _session = initiator.finish(&response, context(generation)).unwrap();
    let wire = ResumeWireBinding {
        session_id: SessionId(claim.session_id),
        delivery_epoch: claim.delivery_epoch,
        key_phase: claim.key_phase,
        path_generation: claim.path_generation,
        expires_at_ms: claim.expires_at_ms,
        token: claim.token,
    };
    (guard, peer, wire)
}

enum SenderCommand {
    Resume(ResumeWireBinding),
    Initial(Vec<Vec<u8>>),
    Resend(Vec<usize>),
    Shutdown,
}
enum BrokerCommand {
    Resume,
    Initial(usize),
    Forward(usize),
    Shutdown,
}

fn sender_process(
    commands: Receiver<SenderCommand>,
    transport: Sender<Vec<u8>>,
    report: Sender<RuntimeState>,
) {
    let mut runtime = SessionRuntime::new(SESSION, limits(), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let mut sent = Vec::new();
    while let Ok(command) = commands.recv() {
        match command {
            SenderCommand::Resume(binding) => transport
                .send(ProcessMessage::Resume { binding }.encode().unwrap())
                .unwrap(),
            SenderCommand::Initial(records) => {
                for data in records {
                    runtime.queue_send(STREAM, &data, 1).unwrap();
                    let frame = ProcessMessage::Data {
                        session: SESSION,
                        record: runtime.pop_send(2).unwrap().unwrap(),
                    };
                    sent.push(frame.encode().unwrap());
                    transport.send(sent.last().unwrap().clone()).unwrap();
                }
            }
            SenderCommand::Resend(indices) => {
                for index in indices {
                    transport.send(sent[index].clone()).unwrap();
                }
            }
            SenderCommand::Shutdown => break,
        }
    }
    report.send(runtime.state()).unwrap();
}

fn receiver_process(
    transport: Receiver<Vec<u8>>,
    acknowledgements: Sender<Vec<u8>>,
    report: Sender<(Vec<u8>, RuntimeState, usize, usize)>,
    mut guard: ResumeGuard,
    peer: Vec<u8>,
) {
    let mut runtime = SessionRuntime::new(SESSION, limits(), 0).unwrap();
    runtime.open_stream(STREAM, 0).unwrap();
    let mut application = Vec::new();
    let mut resumes = 0;
    while let Ok(bytes) = transport.recv() {
        match ProcessMessage::decode(&bytes).unwrap() {
            ProcessMessage::Resume { binding } => {
                let claim = ResumeBinding {
                    session_id: binding.session_id.0,
                    delivery_epoch: binding.delivery_epoch,
                    key_phase: binding.key_phase,
                    path_generation: binding.path_generation,
                    expires_at_ms: binding.expires_at_ms,
                    token: binding.token,
                };
                guard.attach(&peer, &claim, 1).unwrap();
                resumes += 1;
            }
            ProcessMessage::Data { session, record } => {
                assert_eq!(session, SESSION);
                let before = runtime
                    .observable_events()
                    .filter(|e| e.kind == RuntimeEventKind::DataReceived)
                    .count();
                runtime
                    .receive(
                        InboundRecord {
                            stream: record.stream,
                            offset: record.offset,
                            data: record.data,
                        },
                        3,
                    )
                    .unwrap();
                let after = runtime
                    .observable_events()
                    .filter(|e| e.kind == RuntimeEventKind::DataReceived)
                    .count();
                if after != before {
                    let delivered = runtime.pop_receive(4).unwrap().unwrap();
                    acknowledgements
                        .send(
                            ProcessMessage::DeliveryAck {
                                session: SESSION,
                                stream: delivered.stream,
                                offset: delivered.offset,
                                len: delivered.data.len(),
                            }
                            .encode()
                            .unwrap(),
                        )
                        .unwrap();
                    application.extend_from_slice(&delivered.data);
                }
            }
            ProcessMessage::DeliveryAck { .. } => panic!("ack on receiver input"),
        }
    }
    runtime.close_graceful(5).unwrap();
    runtime.close_remote(6).unwrap();
    let dedup = runtime
        .observable_events()
        .filter(|e| e.kind == RuntimeEventKind::DuplicateDedup)
        .count();
    report
        .send((application, runtime.state(), dedup, resumes))
        .unwrap();
}

fn broker_process(
    input: Receiver<Vec<u8>>,
    output: Sender<Vec<u8>>,
    commands: Receiver<BrokerCommand>,
) {
    let mut buffered = Vec::new();
    while let Ok(command) = commands.recv() {
        match command {
            BrokerCommand::Resume => output.send(input.recv().unwrap()).unwrap(),
            BrokerCommand::Initial(count) => {
                buffered.clear();
                for _ in 0..count {
                    buffered.push(input.recv().unwrap());
                }
                output.send(buffered[0].clone()).unwrap();
            }
            BrokerCommand::Forward(count) => {
                for frame in buffered.iter().take(count) {
                    output.send(frame.clone()).unwrap();
                }
            }
            BrokerCommand::Shutdown => break,
        }
    }
}

#[test]
fn process_boundary_udp_blackhole_tcp_resume_is_bounded_exactly_once() {
    let (guard, peer, binding) = fresh_resume(2);
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (wire_tx, wire_rx) = mpsc::channel();
    let (broker_tx, broker_rx) = mpsc::channel();
    let (broker_cmd_tx, broker_cmd_rx) = mpsc::channel();
    let (ack_tx, ack_rx) = mpsc::channel();
    let (sender_report_tx, sender_report_rx) = mpsc::channel();
    let (receiver_report_tx, receiver_report_rx) = mpsc::channel();
    let sender = thread::spawn(move || sender_process(cmd_rx, wire_tx, sender_report_tx));
    let broker = thread::spawn(move || broker_process(wire_rx, broker_tx, broker_cmd_rx));
    let receiver =
        thread::spawn(move || receiver_process(broker_rx, ack_tx, receiver_report_tx, guard, peer));
    let records = vec![
        b"alpha".to_vec(),
        b"-bounded".to_vec(),
        b"-exactly-once".to_vec(),
    ];
    let mut failover = FailoverController::new(2, 8, 128).unwrap();
    for (id, data) in records.iter().enumerate() {
        failover.track_uncertain(DataId(id as u64), data).unwrap();
    }
    cmd_tx.send(SenderCommand::Resume(binding)).unwrap();
    broker_cmd_tx.send(BrokerCommand::Resume).unwrap();
    cmd_tx.send(SenderCommand::Initial(records)).unwrap();
    broker_cmd_tx.send(BrokerCommand::Initial(3)).unwrap();
    assert!(matches!(
        ProcessMessage::decode(&ack_rx.recv().unwrap()).unwrap(),
        ProcessMessage::DeliveryAck { offset: 0, .. }
    ));
    assert!(!failover.udp_pto_at(10));
    assert!(failover.udp_pto_at(20));
    assert_eq!(failover.tcp_resend().unwrap().len(), 3);
    cmd_tx.send(SenderCommand::Resend(vec![1, 2, 0])).unwrap();
    broker_cmd_tx.send(BrokerCommand::Forward(3)).unwrap();
    for id in [DataId(1), DataId(2)] {
        let ProcessMessage::DeliveryAck { offset, .. } =
            ProcessMessage::decode(&ack_rx.recv().unwrap()).unwrap()
        else {
            panic!("missing ack")
        };
        assert!(offset > 0);
        failover.confirm(id).unwrap();
    }
    failover.confirm(DataId(0)).unwrap();
    assert!(failover.tcp_resend().unwrap().is_empty());
    cmd_tx.send(SenderCommand::Shutdown).unwrap();
    drop(cmd_tx);
    sender.join().unwrap();
    broker_cmd_tx.send(BrokerCommand::Shutdown).unwrap();
    drop(broker_cmd_tx);
    broker.join().unwrap();
    drop(ack_rx);
    let (application, state, dedup, resumes) = receiver_report_rx.recv().unwrap();
    receiver.join().unwrap();
    assert_eq!(application, b"alpha-bounded-exactly-once");
    assert_eq!(state, RuntimeState::Closed);
    assert_eq!(dedup, 1);
    assert_eq!(resumes, 1);
    assert_eq!(sender_report_rx.recv().unwrap(), RuntimeState::Open);
}
