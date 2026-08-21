mod msg_handler;

use crate::msn::presentation::switchboard::::msg_handler::handle_msg;
use crate::msn::presentation::switchboard::local_switchboard_data::LocalSwitchboardData;
use crate::msn::presentation::switchboard_server::SwitchboardSenderMsg;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::{Client, Room};
use msnp::msnp::switchboard::command::command::SwitchboardClientCommand;
use tokio::sync::mpsc::Sender;

pub(crate) async fn handle_ready(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardSenderMsg>, tachyon_client: TachyonClient, matrix_client: Client, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    match command {
        SwitchboardClientCommand::ANS(_) => {}
        SwitchboardClientCommand::USR(_) => {}
        SwitchboardClientCommand::CAL(_) => {}
        SwitchboardClientCommand::MSG(msg_command) => {
            handle_msg(msg_command, command_sender, tachyon_client, matrix_client, room, local_switchboard_data).await?;
        }
        SwitchboardClientCommand::OUT => {}
        SwitchboardClientCommand::RAW(_) => {}

    }
    Ok(())
}