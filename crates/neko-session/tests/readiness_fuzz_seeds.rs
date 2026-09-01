use neko_session::ProcessMessage;
#[test]
fn readiness_fuzz_seeds_are_exact_process_messages() {
    for bytes in [
        include_bytes!("../../../fuzz/corpus/decode/readiness-request").as_slice(),
        include_bytes!("../../../fuzz/corpus/decode/readiness-response").as_slice(),
    ] {
        let message = ProcessMessage::decode(bytes).unwrap();
        assert_eq!(message.encode().unwrap(), bytes);
    }
}
