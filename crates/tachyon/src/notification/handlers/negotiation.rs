use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use tokio::sync::mpsc::Sender;
use anyhow::anyhow;
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::session_state::SessionState;

pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, session_state: &mut SessionState) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::VER(command) => {
            if command.first_candidate != MSNP18 && command.second_candidate != MSNP18 {
                //Unsupported protocol version
                //TODO add error code
                notif_sender.send(NotificationServerCommand::OUT).await?;
                return Ok(());
            }

            notif_sender.send(NotificationServerCommand::VER(command.get_response_for(MSNP18))).await?;
            Ok(())
        },
        NotificationClientCommand::CVR(command) => {
            session_state.phase = ConnectionPhase::Authenticating;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("WTF are you doing here"))
        }
    }
}
#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
    use msnp::msnp::notification::command::cvr::CvrClient;
    use msnp::msnp::notification::command::ver::VerClient;
    use msnp::msnp::notification::models::msnp_version::MsnpVersion::{MSNP17, MSNP18, MSNP21};
    use msnp::msnp::raw_command_parser::RawCommand;
    use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
    use std::str::FromStr;
    use tokio::sync::mpsc::{Receiver, Sender};

    use super::handle_negotiation;
    use crate::notification::models::connection_phase::ConnectionPhase;
    use crate::notification::models::session_state::SessionState;

    fn setup() -> (Sender<NotificationServerCommand>, Receiver<NotificationServerCommand>, SessionState) {
        let (snd, rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);
        (snd, rcv, SessionState::new())
    }

    #[tokio::test]
    async fn ver_offering_msnp18_negotiates_msnp18_byte_exact() {
        let (snd, mut rcv, mut data) = setup();

        handle_negotiation(NotificationClientCommand::VER(VerClient::new(1, MSNP18, MSNP17)), snd, &mut data).await.unwrap();

        let ver_respo = rcv.recv().await.unwrap();
        assert!(matches!(ver_respo, NotificationServerCommand::VER(_)));
        let NotificationServerCommand::VER(ver_resp) = ver_respo else {
            unreachable!();
        };

        assert_eq!(String::from_utf8(ver_resp.into_bytes()).unwrap(), "VER 1 MSNP18\r\n");
        assert_eq!(data.phase, ConnectionPhase::Negotiating);
    }

    #[tokio::test]
    async fn ver_with_msnp18_only_in_the_second_slot_still_negotiates_msnp18() {
        let (snd, mut rcv, mut data) = setup();

        handle_negotiation(NotificationClientCommand::VER(VerClient::new(1, MSNP17, MSNP18)), snd, &mut data).await.unwrap();

        let ver_respo = rcv.recv().await.unwrap();
        assert!(matches!(ver_respo, NotificationServerCommand::VER(_)));
        let NotificationServerCommand::VER(ver_resp) = ver_respo else {
            unreachable!();
        };

        assert_eq!(String::from_utf8(ver_resp.into_bytes()).unwrap(), "VER 1 MSNP18\r\n");
    }

    #[tokio::test]
    async fn ver_without_msnp18_in_either_slot_gets_out() {
        let (snd, mut rcv, mut data) = setup();

        handle_negotiation(NotificationClientCommand::VER(VerClient::new(1, MSNP17, MSNP17)), snd.clone(), &mut data).await.unwrap();
        assert!(matches!(rcv.recv().await.unwrap(), NotificationServerCommand::OUT));

        // MSNP21 parses at the codec level but is not accepted by negotiation
        handle_negotiation(NotificationClientCommand::VER(VerClient::new(2, MSNP21, MSNP21)), snd, &mut data).await.unwrap();
        assert!(matches!(rcv.recv().await.unwrap(), NotificationServerCommand::OUT));

        assert_eq!(data.phase, ConnectionPhase::Negotiating);
    }

    #[tokio::test]
    async fn cvr_reply_pins_the_hardcoded_version_and_url_strings() {
        let (snd, mut rcv, mut data) = setup();

        let cvr = CvrClient::try_from_raw(RawCommand::from_str("CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local").unwrap()).unwrap();
        handle_negotiation(NotificationClientCommand::CVR(cvr), snd, &mut data).await.unwrap();

        let cvr_respo = rcv.recv().await.unwrap();
        assert!(matches!(cvr_respo, NotificationServerCommand::CVR(_)));
        let NotificationServerCommand::CVR(cvr_resp) = cvr_respo else {
            unreachable!();
        };

        assert_eq!(String::from_utf8(cvr_resp.into_bytes()).unwrap(), "CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n");
        assert_eq!(data.phase, ConnectionPhase::Authenticating);
    }
}
