use log::debug;
use crate::notification::models::client_out::ClientOut;
use crate::tachyon::client::tachyon_client::TachyonClient;
use msnp::msnp::notification::command::adl::RmlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;

/// Pattern A shell (§3.1), sync — the Sender is gone. The contact-list
/// mutation is shell-owned state and runs before the ack, exactly as
/// pre-D2; the failure path is preserved too: if the payload yields no
/// contacts, the `?` propagates and no ack is produced or sent.
pub fn handle_rml(command: RmlClient, client_data: TachyonClient) -> Result<ClientOut, anyhow::Error> {
    debug!("RML: {:?}", &command);

    client_data.inner.contact_list.lock().unwrap().remove_contacts(command.payload.get_contacts()?);

    Ok(decide_rml(&command))
}

/// Pure decision (§3.1): the RML acknowledgement. Passthrough — no real
/// decision; every well-formed RML earns the same OK.
pub(crate) fn decide_rml(command: &RmlClient) -> ClientOut {
    ClientOut::Command(NotificationServerCommand::OK(command.get_ok_response("RML")))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::command::adl::RmlClient;
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::models::adl_payload::ADLPayload;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::{IntoBytes, TryFromBytes};

    use super::{decide_rml, handle_rml};
    use crate::notification::models::client_out::ClientOut;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn rml(tr_id: u128, payload_xml: &str) -> RmlClient {
        RmlClient {
            tr_id,
            payload: ADLPayload::try_from_bytes(payload_xml.as_bytes().to_vec()).unwrap(),
        }
    }

    #[test]
    fn every_removal_is_acked_with_the_rml_operand_and_its_own_tr_id() {
        let table = vec![
            (5, "<ml><d n=\"test.com\"><c n=\"aeon\" l=\"4\" /></d></ml>", "RML 5 OK\r\n"),
            (9, "<ml><d n=\"yahoo.com.TR\"><c n=\"anyyahoouser\" l=\"4\" /></d></ml>", "RML 9 OK\r\n"),
        ];

        for (tr_id, xml, expected_bytes) in table {
            let command = rml(tr_id, xml);
            let ClientOut::Command(ack) = decide_rml(&command) else {
                panic!("rml decides a bare wire command");
            };

            let NotificationServerCommand::OK(ok) = &ack else {
                panic!("expected the OK ack");
            };
            assert_eq!(ok.operand, "RML");
            assert_eq!(ok.tr_id, tr_id);
            assert_eq!(String::from_utf8(ack.into_bytes()).unwrap(), expected_bytes);
        }
    }

    #[tokio::test]
    async fn an_rml_removes_the_listed_contact_before_the_ack_is_produced() {
        let (snd, _rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));

        let xml = "<ml><d n=\"test.com\"><c n=\"peer\" l=\"4\" /></d></ml>";
        let mut contact_list = ContactList::default();
        contact_list.add_contacts(ADLPayload::try_from_bytes(xml.as_bytes().to_vec()).unwrap().get_contacts().unwrap(), true);
        assert_eq!(contact_list.contact_list.len(), 1);

        let tachyon_client = TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd, kill_snd, kill_recv, contact_list);

        let out = handle_rml(rml(6, xml), tachyon_client.clone()).unwrap();

        assert!(tachyon_client.inner.contact_list.lock().unwrap().contact_list.is_empty());
        let ClientOut::Command(NotificationServerCommand::OK(ok)) = out else {
            panic!("expected the RML ack");
        };
        assert_eq!(ok.operand, "RML");
        assert_eq!(ok.tr_id, 6);
    }
}
