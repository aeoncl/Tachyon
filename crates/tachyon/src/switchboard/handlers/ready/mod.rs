mod msg_handler;

use crate::switchboard::handlers::ready::msg_handler::handle_msg;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::switchboard_server::SwitchboardSenderMsg;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::Room;
use msnp::msnp::switchboard::command::command::SwitchboardClientCommand;
use tokio::sync::mpsc::Sender;

/// Pattern-B shell (§3.1, adl precedent): the Sender survives into the
/// Ready phase because the MSG flow's acks are produced from the spawned
/// payload task — §3.6's dispatch (P2P inline, everything else spawned) is
/// shell policy, and a spawned task cannot return ops to the connection
/// loop. The pure decisions (the payload-arm table and the ack table) live
/// in msg_handler's decide fns; the matrix_client parameter died with the
/// D4 migration because no Ready flow ever read it.
pub(crate) async fn handle_ready(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardSenderMsg>, tachyon_client: TachyonClient, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    match command {
        // Honest empty arms (§5): ANS/USR/CAL were consumed by the
        // bootstrap phases; a stray one in Ready produced nothing pre-D4
        // and produces nothing here.
        SwitchboardClientCommand::ANS(_) => {}
        SwitchboardClientCommand::USR(_) => {}
        SwitchboardClientCommand::CAL(_) => {}
        SwitchboardClientCommand::MSG(msg_command) => {
            handle_msg(msg_command, command_sender, tachyon_client, room, local_switchboard_data).await?;
        }
        // OUT: the client is leaving — the socket close ends the connection
        // in handle_client; pre-D4 produced no wire answer, and none is
        // produced now. RAW: unparsed operands are ignored by design.
        SwitchboardClientCommand::OUT => {}
        SwitchboardClientCommand::RAW(_) => {}

    }
    Ok(())
}
