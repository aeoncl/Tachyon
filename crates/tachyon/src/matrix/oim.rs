use chrono::DateTime;
use log::debug;
use matrix_sdk::Client;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::api::client::filter::RoomEventFilter;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::room::message::{MessageType, SyncRoomMessageEvent};
use matrix_sdk::ruma::{uint, EventId, OwnedRoomId, UserId};
use matrix_sdk::sync::SyncResponse;

use thiserror::Error;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::oim::OIM;
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::shared::payload::msg::raw_msg_payload::MsgContentType;
use crate::tachyon::client_store::ClientStoreError;
use crate::tachyon::tachyon_client::TachyonClient;
use crate::shared::identifiers::MatrixIdCompatible;

#[derive(Error, Debug)]
pub enum OIMError {

    #[error("Logged-in User missing from Cient Store")]
    ClientStoreError(#[from] ClientStoreError),
    #[error(transparent)]
    MatrixSdkError(#[from] matrix_sdk::Error),
    #[error("Couldn't send OIM notification message")]
    NotificationSenderError(#[from] tokio::sync::mpsc::error::SendError<NotificationServerCommand>),
    #[error("Couldn't convert Event timestamp: {} to NaiveDateTime", .event_ts)]
    EventTimestampConvertionError {event_ts: i64},
    #[error("Couldn't convert Event NaiveDateTime to LocalDateTime")]
    NativeDatetimeConversionError{ source: anyhow::Error}
}

pub async fn handle_oims(client: Client, response: SyncResponse, mut client_data: TachyonClient, notif_sender: Sender<NotificationServerCommand>, first_sync_token: Option<String>) -> Result<(), OIMError>{
    let me_email_addr = client_data.own_user()?.endpoint_id.email_addr;

    for (room_id, room) in &response.rooms.joined {
        let room_uuid = Uuid::from_seed(room_id.to_string().as_str());
        let mut seq_num = 1;

        if let Some(prev_batch) = room.timeline.prev_batch.as_ref() {
            //We missed some events
            if let Some(room) = client.get_room(&room_id) {
                let mut config = get_message_options(first_sync_token.clone(), Some(prev_batch.clone()));

                while {
                    let messages = room.messages(config).await?;
                    config = get_message_options(messages.end, Some(prev_batch.clone()));

                    for event in messages.chunk {

                        if let Ok(AnySyncTimelineEvent::MessageLike(e)) = event.raw().deserialize() {
                         //   debug!("Loopty_LOOP: {:?}", e);

                            match e {
                                AnySyncMessageLikeEvent::RoomMessage(ref room_message) => {
                                    let oim = match room_message {
                                        SyncMessageLikeEvent::Original(ref original_event) => {

                                            let member = room.get_member(e.sender()).await?.expect("to be here");
                                            let display_name = member.display_name().map(|e| e.to_string());

                                            handle_original_message(&original_event.content.msgtype, &room_id, room_uuid.clone(),e.event_id(), original_event.origin_server_ts.0.into(), e.sender(), display_name, seq_num, me_email_addr.clone())?
                                        },
                                        SyncMessageLikeEvent::Redacted(_redacted) => {
                                            None
                                        }
                                    };

                                    match oim {
                                        None => {}
                                        Some(oim) => {
                                            client_data.soap_holder().add_oim(oim);
                                            seq_num+=1;
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }


                    }

                    config.from != None
                } {};
            };
        };


        for event in &room.timeline.events {

            if let Ok(AnySyncTimelineEvent::MessageLike(e)) = event.raw().deserialize() {
                debug!("{:?}", e);

                match e {
                    AnySyncMessageLikeEvent::RoomMessage(ref room_message) => {
                        let oim = match room_message {
                            SyncMessageLikeEvent::Original(ref original_event) => {

                                let display_name = {
                                    match client.get_room(&room_id) {
                                        None => {
                                            None
                                        }
                                        Some(room) => {
                                            let member = room.get_member(e.sender()).await?.expect("to be here");
                                            member.display_name().map(|e| e.to_string())
                                        }
                                    }
                                };
                                handle_original_message(&original_event.content.msgtype, &room_id, room_uuid.clone(),e.event_id(), original_event.origin_server_ts.0.into(), e.sender(), display_name, seq_num, me_email_addr.clone())?
                            },
                            SyncRoomMessageEvent::Redacted(_redacted) => {
                                None
                            }
                        };

                        match oim {
                            None => {}
                            Some(oim) => {
                                client_data.soap_holder().add_oim(oim);
                                seq_num+=1;
                            }
                        }
                    },
                    _ => {}
                }
            }

        }
    }


    let payload = if !client_data.soap_holder().oims.is_empty() { RawMsgPayloadFactory::get_initial_mail_data_too_large_notification() } else { RawMsgPayloadFactory::get_initial_mail_data_empty_notification() };

    notif_sender.send(NotificationServerCommand::MSG(MsgServer {
        sender: "Hotmail".to_string(),
        display_name: "Hotmail".to_string(),
        payload: MsgPayload::Raw(payload)
    })).await?;

    Ok(())

}

pub fn handle_original_message(message_type: &MessageType, room_id: &OwnedRoomId, room_uuid: Uuid, event_id: &EventId, event_timestamp: i64, sender: &UserId, sender_display_name: Option<String>, seq_num: u32, me: EmailAddress) -> Result<Option<OIM>, OIMError>{

    Ok(match message_type {
        MessageType::Audio(_) => {None}
        MessageType::Emote(_) => {None}
        MessageType::File(_) => {None}
        MessageType::Image(_) => {None}
        MessageType::Location(_) => {None}
        MessageType::Notice(_) => {None}
        MessageType::ServerNotice(_) => {None}
        MessageType::Text(text) => {
            Some(handle_text_message_event(&room_id, room_uuid, event_id, event_timestamp, sender, sender_display_name, seq_num, &text.body, me)?)
        }
        MessageType::Video(_) => {None}
        MessageType::VerificationRequest(_) => {None}
        MessageType::_Custom(_) => {None}
        _ => {None}
    })

}

pub fn get_message_options(from: Option<String>, to: Option<String>) -> MessagesOptions {
    let mut config = MessagesOptions::forward();
    config.from = from;
    config.to = to;
    config.limit = uint!(10);
    config.filter = RoomEventFilter::empty();
    config.filter.types = Some(vec!["m.room.message".to_string()]);
    config
}


pub fn handle_text_message_event(room_id: &OwnedRoomId, room_uuid: Uuid, event_id: &EventId, event_timestamp: i64, sender: &UserId, sender_display_name: Option<String>, seq_num: u32, body: &str, me: EmailAddress) -> Result<OIM, OIMError> {

    let recv_datetime = DateTime::from_timestamp_millis(event_timestamp).ok_or(OIMError::EventTimestampConvertionError{event_ts: event_timestamp })?;

    Ok(OIM{
        recv_datetime,
        sender: EmailAddress::from_user_id(sender),
        sender_display_name,
        receiver: me,
        run_id: room_uuid,
        seq_number: seq_num,
        message_id: format!("{room_id}_{event_id}", room_id = room_id.as_str(), event_id = event_id.as_str()),
        content: body.to_owned(),
        content_type: MsgContentType::TextPlain,
        read: false,
    })
}
