use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use tokio::sync::mpsc::Sender;
use anyhow::anyhow;
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;

pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, local_client_data: &mut LocalClientData) -> Result<(), anyhow::Error> {
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
            local_client_data.phase = ConnectionPhase::Authenticating;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("WTF are you doing here"))
        }
    }
}