use crate::notification::models::client_out::ClientOut;
use crate::notification::models::session_state::SessionState;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uux::{UuxClient, UuxPayload};

/// Pattern A shell (§3.1), sync — the Sender is gone. The only effect is
/// the private-endpoint-data store on the SessionState, which is shell-owned
/// state and never enters the decide fn (§10). Ordering preserved: the data
/// was stored before the echo was sent pre-D2, and the dispatcher still
/// sends only after this effect ran. Send policy preserved exactly (? on
/// the send, via the dispatcher).
pub fn handle_uux(mut command: UuxClient, local_store: &mut SessionState) -> ClientOut {
    match command.payload.take() {
        None => {}
        Some(payload) => {
            match payload {
                UuxPayload::PrivateEndpointData(private_endpoint_data) => {
                    local_store.private_endpoint_data = private_endpoint_data;
                    //TODO
                }
                UuxPayload::Unknown(_) => {}
            }
        }
    }

    decide_uux(&command)
}

/// Pure decision (§3.1): the UUX acknowledgement — the echo of the tr_id
/// with no payload, exactly what get_ok_response built pre-D2 (it only ever
/// reads the tr_id, so taking the payload in the shell above cannot change
/// the echo). Passthrough — no real decision; every UUX earns the same echo
/// whatever its payload was (the payload's only consumer is the shell
/// effect above).
pub(crate) fn decide_uux(command: &UuxClient) -> ClientOut {
    ClientOut::Command(NotificationServerCommand::UUX(command.get_ok_response()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::notification::command::uux::{UuxClient, UuxPayload};
    use msnp::shared::traits::IntoBytes;

    use super::{decide_uux, handle_uux};
    use crate::notification::models::client_out::ClientOut;
    use crate::notification::models::session_state::SessionState;

    fn private_endpoint_payload() -> UuxPayload {
        UuxPayload::from_str("<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>").unwrap()
    }

    #[test]
    fn the_payload_less_echo_is_the_uux_ack_for_every_tr_id() {
        let table = vec![8u128, 9, 10];

        for tr_id in table {
            let command = UuxClient { tr_id, payload: Some(private_endpoint_payload()) };

            let ClientOut::Command(out) = decide_uux(&command) else {
                panic!("uux decides a bare wire command");
            };
            assert_eq!(String::from_utf8(out.into_bytes()).unwrap(), format!("UUX {tr_id} 0\r\n"));
        }
    }

    #[test]
    fn private_endpoint_data_lands_in_the_session_state_before_the_echo_is_produced() {
        let table = vec![
            (11, Some(private_endpoint_payload()), "M1CROW8Vl"),
            (12, Some(UuxPayload::Unknown("junk".to_string())), ""),
            (13, None, ""),
        ];

        for (tr_id, payload, expected_ep_name) in table {
            let mut session_state = SessionState::new();

            let ClientOut::Command(out) = handle_uux(UuxClient { tr_id, payload }, &mut session_state) else {
                panic!("uux decides a bare wire command");
            };

            assert_eq!(session_state.private_endpoint_data.ep_name, expected_ep_name);
            assert_eq!(String::from_utf8(out.into_bytes()).unwrap(), format!("UUX {tr_id} 0\r\n"));
        }
    }
}
