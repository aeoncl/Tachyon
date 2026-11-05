use log::debug;
use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uun::{UunClient, UunPayload};
use msnp::msnp::notification::command::ubn::{UbnPayload, UbnServer};
use msnp::p2p::v2::slp::raw_slp_payload::SlpPayloadFactory;

/// Pure decision (§3.1): the whole verb is decide — no shell, no IO. Every
/// notification is acked with the UUN OK except the unknown payloads, which
/// are only logged (the pre-D2 handler sent nothing for them either). A P2P
/// transport request additionally earns the 500 direct-connect refusal UBN —
/// the NS never brokers P2P. The never-read TachyonClient and matrix Client
/// parameters died with the Sender.
/// Ordering: OK before UBN, preserved by the Vec order — the dispatcher
/// sends in order with `?` on every send, the pre-D2 policy.
pub(crate) fn decide_uun(command: UunClient) -> Vec<ClientOut> {
    let ok_response = command.get_ok_response();

    match command.payload {
        UunPayload::DisconnectClient => {
            vec![ClientOut::Command(NotificationServerCommand::OK(ok_response))]
        }
        UunPayload::DisconnectAllClients => {
            vec![ClientOut::Command(NotificationServerCommand::OK(ok_response))]
        }
        UunPayload::ConversationWindowClosed { .. } => {
            vec![ClientOut::Command(NotificationServerCommand::OK(ok_response))]
        }
        UunPayload::DismissUserInvite { .. } => {
            vec![ClientOut::Command(NotificationServerCommand::OK(ok_response))]
        }
        UunPayload::Resynchronize(_) => {
            vec![ClientOut::Command(NotificationServerCommand::OK(ok_response))]
        }
        UunPayload::P2PData(transport_req) => {

            let slp_transport_req_error_response = SlpPayloadFactory::get_500_error_direct_connect(
                &transport_req,
                String::from("TCPv1"),
            ).unwrap();


            vec![
                ClientOut::Command(NotificationServerCommand::OK(ok_response)),
                ClientOut::Command(NotificationServerCommand::UBN(UbnServer::new(command.destination, UbnPayload::P2PData(slp_transport_req_error_response)))),
            ]
        }
        UunPayload::Unknown(content) => {

            debug!("UUN: {}", String::from_utf8(content).unwrap());

            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::command::uun::{UunClient, UunPayload, UunSoapStatePayload};
    use msnp::msnp::notification::command::ubn::{UbnPayload, UbnServer};
    use msnp::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, SlpPayloadFactory};
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::traits::IntoBytes;

    use super::decide_uun;
    use crate::notification::models::client_out::ClientOut;

    fn uun(tr_id: u128, payload: UunPayload) -> UunClient {
        UunClient { tr_id, destination: EndpointId::from_str("aeon@test.com").unwrap(), payload }
    }

    fn slp_transport_request() -> String {
        [
            "INVITE MSNMSGR:aeon@test.com MSNSLP/1.0",
            "To: aeon@test.com",
            "From: peer@test.com",
            "Via: MSNSLP/1.0/TLP ;branch={8B0F4A24-8B0F-4A24-8B0F-4A248B0F4A24}",
            "CSeq: 1",
            "Call-ID: {1F8BFCBF-4F72-587D-8A18-489C017B448B}",
            "Max-Forwards: 0",
            "Content-Type: application/x-msnmsgr-transreqbody",
            "",
            "Bridges: TCPv1",
            "Nonce: {00000000-0000-0000-0000-000000000000}",
            "",
        ].join("\r\n")
    }

    #[test]
    fn every_plain_notification_is_acked_with_the_uun_ok() {
        let table = vec![
            (1, UunPayload::DisconnectClient),
            (2, UunPayload::DisconnectAllClients),
            (3, UunPayload::ConversationWindowClosed { email_addr: "peer@test.com".to_string() }),
            (4, UunPayload::DismissUserInvite { email_addr: "peer@test.com".to_string(), unknown: 1 }),
            (5, UunPayload::Resynchronize(UunSoapStatePayload::default())),
        ];

        for (tr_id, payload) in table {
            let mut outs = decide_uun(uun(tr_id, payload)).into_iter();

            let ClientOut::Command(command) = outs.next().expect("the ack") else {
                panic!("uun decides bare wire commands");
            };
            assert!(matches!(command, NotificationServerCommand::OK(_)));
            assert_eq!(String::from_utf8(command.into_bytes()).unwrap(), format!("UUN {tr_id} OK\r\n"));
            assert!(outs.next().is_none());
        }
    }

    #[test]
    fn a_p2p_transport_request_is_acked_then_refused_with_the_500_direct_connect_ubn() {
        let transport_req = RawSlpPayload::from_str(&slp_transport_request()).unwrap();

        let expected = UbnServer::new(
            EndpointId::from_str("aeon@test.com").unwrap(),
            UbnPayload::P2PData(SlpPayloadFactory::get_500_error_direct_connect(&transport_req, String::from("TCPv1")).unwrap()),
        ).into_bytes();
        assert!(String::from_utf8(expected.clone()).unwrap().starts_with("UBN aeon@test.com 3 "));

        let mut outs = decide_uun(uun(7, UunPayload::P2PData(transport_req))).into_iter();

        let ClientOut::Command(ok) = outs.next().expect("the ack first") else {
            panic!("uun decides bare wire commands");
        };
        assert_eq!(String::from_utf8(ok.into_bytes()).unwrap(), "UUN 7 OK\r\n");

        let ClientOut::Command(ubn) = outs.next().expect("the refusal second") else {
            panic!("uun decides bare wire commands");
        };
        assert!(matches!(ubn, NotificationServerCommand::UBN(_)));
        assert_eq!(ubn.into_bytes(), expected);
        assert!(outs.next().is_none());
    }

    #[test]
    fn an_unknown_notification_payload_is_logged_and_never_acked() {
        let outs = decide_uun(uun(6, UunPayload::Unknown(b"<Data></Data>".to_vec())));

        assert!(outs.is_empty());
    }
}
