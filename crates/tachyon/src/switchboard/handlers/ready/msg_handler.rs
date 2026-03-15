use std::str::FromStr;
use matrix_sdk::Room;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::msg::MsgServer;
use msnp::msnp::switchboard::command::ack::AckServer;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgClient, MsgPayload};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use msnp::shared::payload::msg::service_msg::ServiceMessagePayload;
use msnp::shared::traits::MSNPPayload;
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
        MsgPayload::ServiceMessage(_) => {}
    }

    Ok(())
}