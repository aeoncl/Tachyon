use anyhow::anyhow;
use tokio::sync::mpsc::Sender;
use msnp::msnp::switchboard::command::command::SwitchboardClientCommand;
use crate::app_state::AppState;
use crate::msn::presentation::switchboard::bootstrap_handlers::{handle_auth, handle_init};
use crate::msn::presentation::switchboard::ready_handlers::handle_ready;
use crate::msn::presentation::switchboard_server::SwitchboardSenderMsg;
use crate::msn::presentation::switchboard::connection_phase::ConnectionPhase;
use crate::msn::presentation::switchboard::local_switchboard_data::LocalSwitchboardData;

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardSenderMsg>, tachyon_state: &AppState, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match local_switchboard_data.phase {
        ConnectionPhase::Authenticating => {
            handle_auth(command, command_sender, tachyon_state, local_switchboard_data).await?
        }
        ConnectionPhase::Initializing => {
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            let matrix_client = local_switchboard_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client Data should be here by now"))?.clone();
            handle_init(command, command_sender, tachyon_client, matrix_client, local_switchboard_data).await?

        }
        ConnectionPhase::Ready => {
            let room = local_switchboard_data.room.as_ref().ok_or(anyhow!("Room should be here by now"))?.clone();
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Tachyon Client should be here by now"))?.clone();
            let matrix_client = local_switchboard_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client Data should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, matrix_client, room, local_switchboard_data).await?
        }
    }
    Ok(())
}