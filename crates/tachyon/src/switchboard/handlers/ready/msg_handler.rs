use futures_util::FutureExt;
use log::{debug, error};
use crate::matrix::extensions::message_dedup::SendWithDedup;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::{Client, Error, Room};
use matrix_sdk::room::futures::SendMessageLikeEventResult;
use msnp::msnp::switchboard::command::ack::AckServer;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgClient, MsgPayload};
use msnp::shared::command::nak::NakServer;
use tokio::sync::mpsc::Sender;
use msnp::shared::payload::msg::chunked_msg_payload::{ChunkMetadata, ChunkedMsgPayload, MsgChunks};
use crate::tachyon::identifiers::matrix_id_compatible::MatrixIdCompatible;

pub(super) async fn handle_msg(msg_command: MsgClient, command_sender: Sender<SwitchboardServerCommand>, tachyon_client: TachyonClient, matrix_client: Client, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    if let MsgPayload::Chunked(chunk) = msg_command.payload {
        if let Some(complete) = handle_chunked(chunk, local_switchboard_data).await? {
            handle_msg_payload_task(msg_command.tr_id, msg_command.ack_type, complete, &room, &command_sender);
        }
    } else {
        handle_msg_payload_task(msg_command.tr_id, msg_command.ack_type, msg_command.payload, &room, &command_sender);
    };

    Ok(())
}

pub async fn handle_chunked(chunk: ChunkedMsgPayload, local_switchboard_data: &mut LocalSwitchboardData) -> Result<Option<MsgPayload>, anyhow::Error> {
    let message_id = chunk.message_id();
    match local_switchboard_data.chunks.remove(&message_id) {
        Some(mut chunks) => {
            chunks.append_chunk(chunk);
            if chunks.is_complete()? {
                return Ok(Some(chunks.drain_chunks()?));
            } else {
                local_switchboard_data.chunks.insert(message_id, chunks);
            }
        }
        None => {
            local_switchboard_data.chunks.insert(message_id, MsgChunks::from_first_chunk(chunk)?);
        }
    }

    Ok(None)
}

pub fn handle_msg_payload_task(tr_id: u128, ack_type: MsgAcknowledgment, payload: MsgPayload, room: &Room, command_sender: &Sender<SwitchboardServerCommand>) {

    let room_clone = room.clone();
    let command_sender_clone = command_sender.clone();

    tokio::spawn(async move {
        match payload {
            MsgPayload::Raw(_) => {

            }
            MsgPayload::TextPlain(text_plain) => {

                let message = RoomMessageEventContent::text_plain(text_plain.body);
                if let Err(e) = room_clone.send_with_dedup(message).await  {
                        error!("Could not send message {:?}", e);
                        match ack_type {
                            MsgAcknowledgment::AckOnFailure | MsgAcknowledgment::AckA | MsgAcknowledgment::AckD => {
                                command_sender_clone.send(SwitchboardServerCommand::NAK(NakServer::new(tr_id))).await;
                            }
                            _ => {}
                        }
                    } else {
                        match ack_type {
                            MsgAcknowledgment::AckA | MsgAcknowledgment::AckD => {
                                command_sender_clone.send(SwitchboardServerCommand::ACK(AckServer::new(tr_id))).await;
                            }
                            _ => {}
                        }
                    }
            }
            MsgPayload::Datacast(datacast) => {
                debug!("received DATACAST {:?}", &datacast.get_type() );
            }
            MsgPayload::Control(control) => {

                let typing_user_id = control.typing_user.to_owned_user_id();

                if &typing_user_id == room_clone.own_user_id() {
                    room_clone.typing_notice(true).await;
                }
            }
            MsgPayload::P2P(_) => {}
            MsgPayload::Chunked(_) => {
            }
        }
    });

}