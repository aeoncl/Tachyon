use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::fqy::{FqyClient, FqyServer};

/// Pure decision (§3.1): echo the queried payload back untouched.
/// Passthrough — no real decision: the yahoo "actual=" enrichment below is
/// commented out since before D2 (it needs DM rooms Tachyon hasn't joined
/// yet), so no contact-list lock or lookup happens — contrary to the D2b-i
/// slice notes there is no gather for fqy; the code is authoritative. The
/// unused TachyonClient parameter died with the Sender.
pub(crate) fn decide_fqy(command: FqyClient) -> ClientOut {

/*    for domain in payload.domains.as_mut_slice() {
        for contact in domain.contacts.as_mut_slice() {
            let email_address = EmailAddress::from_str(format!("{}@{}", &contact.email_part, &domain.domain).as_str()).unwrap();
            let user_id = email_address.to_owned_user_id();

            //Doesnt work because we don't have the room in joined yet. :(
            if let Some(found) = matrix_client.get_dm_room(&user_id) {
                contact.actual = Some(found.to_email_address().unwrap().to_string());
            }
        }
    }*/

    ClientOut::Command(NotificationServerCommand::FQY(FqyServer {
        tr_id: command.tr_id,
        payload: command.payload,
    }))
}

#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::command::fqy::FqyClient;
    use msnp::msnp::notification::models::adl_payload::ADLPayload;
    use msnp::shared::traits::{IntoBytes, TryFromBytes};

    use super::decide_fqy;
    use crate::notification::models::client_out::ClientOut;

    fn fqy(tr_id: u128, payload_xml: &str) -> FqyClient {
        FqyClient {
            tr_id,
            payload: ADLPayload::try_from_bytes(payload_xml.as_bytes().to_vec()).unwrap(),
        }
    }

    #[test]
    fn a_forward_list_query_is_echoed_back_untouched() {
        let table = vec![
            (7, "<ml><d n=\"yahoo.com.TR\"><c n=\"anyyahoouser\" /></d></ml>"),
            (8, "<ml><d n=\"test.com\"><c n=\"aeon\" /><c n=\"peer\" /></d></ml>"),
        ];

        for (tr_id, xml) in table {
            let expected_payload = ADLPayload::try_from_bytes(xml.as_bytes().to_vec()).unwrap();

            let ClientOut::Command(command) = decide_fqy(fqy(tr_id, xml)) else {
                panic!("fqy decides a bare wire command");
            };
            let NotificationServerCommand::FQY(echo) = command else {
                panic!("expected the FQY echo");
            };

            assert_eq!(echo.tr_id, tr_id);
            assert_eq!(echo.payload.into_bytes(), expected_payload.into_bytes());
        }
    }
}
