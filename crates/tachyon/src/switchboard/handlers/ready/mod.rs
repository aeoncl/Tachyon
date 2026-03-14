mod msg_handler;

use crate::switchboard::handlers::ready::msg_handler::handle_msg;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::Room;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use tokio::sync::mpsc::Sender;

pub(crate) async fn handle_ready(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, tachyon_client: TachyonClient, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    match command {
        SwitchboardClientCommand::ANS(_) => {}
        SwitchboardClientCommand::USR(_) => {}
        SwitchboardClientCommand::CAL(_) => {}
        SwitchboardClientCommand::MSG(msg_command) => {
            handle_msg(msg_command, command_sender, tachyon_client, room, local_switchboard_data).await?;
        }
        SwitchboardClientCommand::OUT => {}
        SwitchboardClientCommand::RAW(_) => {}

    }
    Ok(())
}