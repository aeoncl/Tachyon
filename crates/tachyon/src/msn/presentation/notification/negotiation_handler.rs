use crate::msn::domain::msn_bridge::MsnpBridge;
use crate::msn::notification::models::local_client_data::LocalClientData;
use anyhow::anyhow;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::cvr::CvrServer;
use tokio::sync::mpsc::Sender;
use crate::utils::ReduceToString;

pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, local_client_data: &mut LocalClientData, bridge: MsnpBridge) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::VER(command) => {
            let candidates = vec![&command.first_candidate, &command.second_candidate];
            for candidate in candidates.as_slice() {
                if bridge.supports_protocol_version(candidate) {

                    bridge.set_protocol_version(candidate).map_err(|e|anyhow!("ghdgh"))?;

                    notif_sender.send(NotificationServerCommand::VER(command.get_response_for((*candidate).clone()))).await?;
                    return Ok(())
                }
            }

            Err(anyhow!("Could not find compatible protocol version. Candidates: {}. Supported: {}", candidates.reduce_to_string() , bridge.supported_protocol_versions().reduce_to_string()))
        },
        NotificationClientCommand::CVR(command) => {
            bridge.end_negotiation().map_err(|e|anyhow!("sfgs"))?;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("Received unsupported command in Negotiating phase: `{}`", raw_command))
        }
    }
}