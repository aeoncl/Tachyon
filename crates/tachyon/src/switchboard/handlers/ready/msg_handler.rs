use matrix_sdk::Room;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use tokio::sync::mpsc::Sender;
use msnp::msnp::switchboard::command::ack::AckServer;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgClient, MsgPayload};
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::tachyon_client::TachyonClient;

pub(super) async fn handle_msg(msg_command: MsgClient, command_sender: Sender<SwitchboardServerCommand>, tachyon_client: TachyonClient, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    
    match msg_command.payload {
        MsgPayload::Raw(_) => {

        }
        MsgPayload::TextPlain(text_plain) => {
            let message = RoomMessageEventContent::text_plain(text_plain.body);
            match room.send(message).await {
                Ok(_) => {
                    match msg_command.ack_type {
                        MsgAcknowledgment::AckA | MsgAcknowledgment::AckD => {
                            command_sender.send(SwitchboardServerCommand::ACK(AckServer::new(msg_command.tr_id))).await?;
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    match msg_command.ack_type {
                        MsgAcknowledgment::AckOnFailure => {
                            command_sender.send(SwitchboardServerCommand::ACK(AckServer::new(msg_command.tr_id))).await?;
                        }
                        _ => {}
                    }
                }
            }

        }
    }

    Ok(())
}