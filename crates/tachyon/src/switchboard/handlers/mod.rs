use crate::switchboard::handlers::bootstrap_handlers::{handle_auth, handle_init};
use crate::switchboard::handlers::ready::handle_ready;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::tachyon_state::TachyonState;
use anyhow::anyhow;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use tokio::sync::mpsc::Sender;

mod bootstrap_handlers;
mod ready;

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, tachyon_state: &TachyonState, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match local_switchboard_data.phase {
        ConnectionPhase::Authenticating => {
            handle_auth(command, command_sender, tachyon_state, local_switchboard_data).await?
        }
        ConnectionPhase::Initializing => {
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            handle_init(command, command_sender, tachyon_client, local_switchboard_data).await?

        }
        ConnectionPhase::Ready => {
            let room = local_switchboard_data.room.as_ref().ok_or(anyhow!("Room should be here by now"))?.clone();
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, room, local_switchboard_data).await?
        }
    }
    Ok(())
}