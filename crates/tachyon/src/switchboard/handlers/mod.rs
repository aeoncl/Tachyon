use crate::switchboard::handlers::bootstrap_handlers::{handle_auth, handle_init};
use crate::switchboard::handlers::ready::handle_ready;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::global::global_state::GlobalState;
use anyhow::anyhow;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use tokio::sync::mpsc::Sender;
use crate::tachyon::repository::RepositoryStr;

mod bootstrap_handlers;
mod ready;

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, tachyon_state: &GlobalState, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

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