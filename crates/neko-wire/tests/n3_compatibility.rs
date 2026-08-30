//! N3 compatibility harness for the integrated N1 version-negotiation contract.
//!
//! This deliberately tests only current/current and unsupported/future behavior.
//! Previous/current remains deferred until the release ledger names a formal,
//! frozen previous release.

use neko_wire::{NegotiationError, NegotiationRole, NegotiationState, VersionNegotiator};

const CURRENT: u16 = 0;
const FUTURE: u16 = CURRENT + 1;

fn negotiate(
    client_versions: &[u16],
    server_versions: &[u16],
) -> (VersionNegotiator, VersionNegotiator, u16) {
    let mut client = VersionNegotiator::new(NegotiationRole::Client, client_versions)
        .expect("client contract fixture must be valid");
    let mut server = VersionNegotiator::new(NegotiationRole::Server, server_versions)
        .expect("server contract fixture must be valid");
    let hello = client.client_hello().expect("client hello");
    let response = server.server_accept_hello(&hello).expect("server response");
    let selected = client
        .client_accept_response(&response)
        .expect("client response");
    (client, server, selected)
}

#[test]
fn current_current_establishes_highest_common_version_before_data_admission() {
    let (client, server, selected) = negotiate(&[CURRENT, 2], &[CURRENT, 1]);
    assert_eq!(selected, CURRENT);
    assert_eq!(client.state(), NegotiationState::Established(CURRENT));
    assert_eq!(server.state(), NegotiationState::Established(CURRENT));
    assert_eq!(client.admit_data(), Ok(CURRENT));
    assert_eq!(server.admit_data(), Ok(CURRENT));
}

#[test]
fn unsupported_or_future_only_offer_is_rejected_and_terminal() {
    let mut server = VersionNegotiator::new(NegotiationRole::Server, &[CURRENT]).unwrap();
    let client = VersionNegotiator::new(NegotiationRole::Client, &[FUTURE]).unwrap();
    let hello = client.client_hello().unwrap();

    assert_eq!(
        server.server_accept_hello(&hello),
        Err(NegotiationError::NoCompatibleVersion)
    );
    assert_eq!(server.state(), NegotiationState::Rejected);
    assert_eq!(
        server.admit_data(),
        Err(NegotiationError::UnexpectedMessage)
    );
    assert_eq!(
        server.server_accept_hello(&hello),
        Err(NegotiationError::LateMessage)
    );
}

#[test]
fn future_selected_response_is_rejected_before_data_admission() {
    let mut client = VersionNegotiator::new(NegotiationRole::Client, &[CURRENT]).unwrap();
    let _hello = client.client_hello().unwrap();
    // N1 response: N1 || type=2 || reserved=0 || selected=u16be.
    let future_response = [b'N', b'1', 2, 0, (FUTURE >> 8) as u8, FUTURE as u8];

    assert_eq!(
        client.client_accept_response(&future_response),
        Err(NegotiationError::UnsupportedSelected(FUTURE))
    );
    assert_eq!(client.state(), NegotiationState::Rejected);
    assert_eq!(
        client.admit_data(),
        Err(NegotiationError::UnexpectedMessage)
    );
}
