use crate::notification::models::client_out::ClientOut;
use crate::tachyon::client::tachyon_client::TachyonClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::prp::{PrpClient, PrpOperation};

/// Pattern A shell (§3.1), sync — the Sender is gone. The only effect is
/// the own-user display-name update, which is shell-owned state on the
/// TachyonClient and never enters the decide fn (§10). The pre-D2 handler's
/// SessionState parameter was never read and is gone. Ordering preserved:
/// the name was stored before the echo was sent pre-D2, and the dispatcher
/// still sends only after this effect ran.
pub fn handle_prp(command: PrpClient, client_data: TachyonClient) -> ClientOut {
    match &command.operation {
        PrpOperation::ModifyName { display_name } => {
            client_data.own_user_mut().display_name = display_name_to_store(display_name);
        }
    }

    decide_prp(command)
}

/// The stored own-user display name: None when the client cleared it,
/// the name otherwise. Pure — this is the verb's only real decision.
pub(crate) fn display_name_to_store(display_name: &str) -> Option<String> {
    if display_name.is_empty() {
        None
    } else {
        Some(display_name.to_owned())
    }
}

/// Pure decision (§3.1): the property change is acked by echoing the PRP
/// back. Passthrough — the echo is unconditional for every PRP operation
/// the codec parses (today only MFN).
pub(crate) fn decide_prp(command: PrpClient) -> ClientOut {
    match &command.operation {
        PrpOperation::ModifyName { .. } => ClientOut::Command(NotificationServerCommand::PRP(command)),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::command::prp::{Prp, PrpClient, PrpOperation};
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::IntoBytes;

    use super::{decide_prp, display_name_to_store, handle_prp};
    use crate::notification::models::client_out::ClientOut;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn prp(tr_id: u128, display_name: &str) -> PrpClient {
        Prp { tr_id, operation: PrpOperation::ModifyName { display_name: display_name.to_string() } }
    }

    #[test]
    fn the_display_name_normalization_stores_none_only_for_the_cleared_name() {
        let table = vec![
            ("", None),
            ("Aeon Test", Some("Aeon Test".to_string())),
            (" ", Some(" ".to_string())),
        ];

        for (input, expected) in table {
            assert_eq!(display_name_to_store(input), expected);
        }
    }

    #[test]
    fn decide_echoes_the_mfn_change_back_as_the_ack() {
        let ClientOut::Command(command) = decide_prp(prp(2, "Aeon Test")) else {
            panic!("prp decides a bare wire command");
        };

        assert_eq!(String::from_utf8(command.into_bytes()).unwrap(), "PRP 2 MFN Aeon%20Test\r\n");
    }

    #[tokio::test]
    async fn an_mfn_change_lands_in_the_own_user_before_the_echo_is_produced() {
        let (snd, _rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);

        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        let tachyon_client = TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd, kill_snd, kill_recv, ContactList::default());

        let cleared = handle_prp(prp(4, ""), tachyon_client.clone());
        assert!(tachyon_client.own_user_mut().display_name.is_none());
        assert!(matches!(cleared, ClientOut::Command(NotificationServerCommand::PRP(_))));

        let renamed = handle_prp(prp(5, "Aeon Test"), tachyon_client.clone());
        assert_eq!(tachyon_client.own_user_mut().display_name.as_deref(), Some("Aeon Test"));
        assert!(matches!(renamed, ClientOut::Command(NotificationServerCommand::PRP(_))));
    }
}
