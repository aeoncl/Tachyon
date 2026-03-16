use futures_util::FutureExt;
use log::debug;
use crate::matrix::extensions::message_dedup::SendWithDedup;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::Room;
use msnp::msnp::switchboard::command::ack::AckServer;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgClient, MsgPayload};
use msnp::msnp::switchboard::command::nak::NakServer;
use tokio::sync::mpsc::Sender;
use msnp::shared::payload::msg::chunked_msg_payload::{ChunkMetadata, ChunkedMsgPayload, MsgChunks};

pub(super) async fn handle_msg(msg_command: MsgClient, command_sender: Sender<SwitchboardServerCommand>, tachyon_client: TachyonClient, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    let result = if let MsgPayload::Chunked(chunk) = msg_command.payload {
        let chunked_now_complete = handle_chunked(chunk, local_switchboard_data).await?;
        match chunked_now_complete {
            None => {
                Ok(())
            }
            Some(complete) => {
                handle_msg_payload(complete, &room).await
            }
        }
    } else {
        handle_msg_payload(msg_command.payload, &room).await
    };

    if let Err(e) = result {
        match msg_command.ack_type {
            MsgAcknowledgment::AckOnFailure | MsgAcknowledgment::AckA | MsgAcknowledgment::AckD => {
                command_sender.send(SwitchboardServerCommand::NAK(NakServer::new(msg_command.tr_id))).await?;
            }
            _ => {}
        }
    } else {
        match msg_command.ack_type {
            MsgAcknowledgment::AckA | MsgAcknowledgment::AckD => {
                command_sender.send(SwitchboardServerCommand::ACK(AckServer::new(msg_command.tr_id))).await?;
            }
            _ => {}
        }
    }

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

pub async fn handle_msg_payload(payload: MsgPayload, room: &Room) -> Result<(), anyhow::Error> {

    match payload {
        MsgPayload::Raw(_) => {

        }
        MsgPayload::TextPlain(text_plain) => {

            let message = RoomMessageEventContent::text_plain(text_plain.body);
            room.send_with_dedup(message).await?;

        }
        MsgPayload::Datacast(datacast) => {
            debug!("received DATACAST {:?}", &datacast.get_type() );
        }
        MsgPayload::Control => {}
        MsgPayload::P2P(_) => {}
        MsgPayload::Chunked(_) => {
        }
    }

    Ok(())


}