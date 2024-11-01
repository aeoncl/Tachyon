use js_int::UInt;
use log::{error, info, warn};
use matrix_sdk::{Client, Room, RoomMemberships};
use matrix_sdk::media::{MediaFormat, MediaRequest};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::message::{AudioMessageEventContent, FileMessageEventContent, ImageMessageEventContent, MessageType, RoomMessageEventContent, SyncRoomMessageEvent, VideoMessageEventContent};

use crate::matrix::direct_target_resolver;
use crate::models::conversion::audio_conversion::convert_incoming_audio_message;
use crate::models::msg_payload::factories::MsgPayloadFactory;
use crate::models::msn_object::MSNObjectFactory;
use crate::models::msn_user::MSNUser;
use crate::models::notification::msn_client::MSNClient;
use crate::models::switchboard::switchboard::Switchboard;
use crate::repositories::msn_user_repository::MSNUserRepository;
use crate::utils::emoji::emoji_to_smiley;
use crate::utils::string::encode_base64;

pub(crate) async fn handle_sync_room_message_event(ev: SyncRoomMessageEvent, room: Room, client: Client, msn_client: MSNClient) {

    if let SyncRoomMessageEvent::Original(ev) = ev {
        let joined_members = room.members(RoomMemberships::JOIN).await.unwrap_or(Vec::new());

        let debug = room.is_direct();
        let debug_len = joined_members.len();

        if room.is_direct().await.unwrap_or(false) && joined_members.len() <= 2 {
        let me_user_id = client.user_id().unwrap();
            if let Some(target) = direct_target_resolver::resolve_direct_target(&room.direct_targets(), &room, &me_user_id, &client).await {
               let switchboard = msn_client.get_or_init_switchboard( room.room_id().to_string(), MSNUser::from_matrix_id(target.clone()));
               handle_message(client.clone(), &room.room_id(), &switchboard, &ev).await;
            }
        }

    }
}

async fn handle_message(matrix_client: Client, room_id: &RoomId, switchboard: &Switchboard, msg_event: &OriginalSyncMessageLikeEvent<RoomMessageEventContent>) {
    info!("Handle message!");

    let user_repo = MSNUserRepository::new(matrix_client.clone());

    let sender = user_repo.get_msnuser(&room_id, &msg_event.sender, false).await.unwrap();

    match &msg_event.content.msgtype {
        MessageType::Text(content) => {
            let msg = MsgPayloadFactory::get_message(emoji_to_smiley(&content.body));
            switchboard.on_message_received(msg, sender, Some(msg_event.event_id.to_string()));
        },
        MessageType::File(content) => {
            switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), get_size_or_default_file(&content), msg_event.event_id.to_string());
        },
        MessageType::Audio(content) => {
            match &content.source {
                MediaSource::Plain(source) => {
                    let base64_mxc = encode_base64(source.to_string());

                    let media_request = MediaRequest{ source: MediaSource::Plain(source.to_owned()), format: MediaFormat::File };
                    let media_client = &matrix_client.media();
                    let media = media_client.get_media_content(&media_request, true).await.unwrap(); //TODO exception handling
                    let media_length = get_size_or_default_audio(&content);

                    match convert_incoming_audio_message(media).await {
                        Ok(converted_media) => {
                            if converted_media.len() <= 30000 {
                                //30Ko is the max allowed size of a voice message for WLM 2009
                                let obj = MSNObjectFactory::get_voice_message(&converted_media, sender.get_msn_addr(), Some(base64_mxc));
                                let msg = MsgPayloadFactory::get_msnobj_datacast(&obj);
                                switchboard.on_message_received(msg, sender, Some(msg_event.event_id.to_string()));
                            } else {
                                //Fallback on normal file upload
                                switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), media_length, msg_event.event_id.to_string());
                            }
                        },
                        Err(err) => {
                            error!("Conversion error, falling back to file message: {}", err);
                            switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), media_length, msg_event.event_id.to_string());
                        }
                    }


                },
                MediaSource::Encrypted(source) => {
                    warn!("Encrypted audio message received {:?}", msg_event);
                }
            };
        },
        MessageType::Image(content) => {
            switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), get_size_or_default_image(&content), msg_event.event_id.to_string());
        },
        MessageType::Video(content)=> {
            switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), get_size_or_default_video(&content), msg_event.event_id.to_string());
        },
        MessageType::Emote(content) => {
            log::info!("Received an Emote: {:?}", &content);
        },
        MessageType::Location(content) => {
            log::info!("Received location message: {:?} - plain text representation: {}", &content, content.plain_text_representation());
        },
        MessageType::Notice(content) => {
            log::info!("Received a Notice: {:?}", &content);
        },
        MessageType::ServerNotice(content)=> {
            log::info!("Received a ServerNotice: {:?}", &content);
        },
        MessageType::VerificationRequest(content)=> {
            log::info!("Received a VerificationRequest: {:?}", &content);

        },
        MessageType::_Custom(content) => {
            log::info!("Received a Custom Event: {:?}", &content);
        },
        _ => {}
    }
}

fn get_size_or_default_file(content: &FileMessageEventContent) -> usize {
    let mut size: i32 = 0;
    if let Some(info) = content.info.as_ref() {
        if let Ok(valid_size) = i32::try_from(info.size.unwrap_or(UInt::new(0).unwrap())) {
            size = valid_size;
        }
    }
    return usize::try_from(size).expect("Matrix file size to be a usize");
}

fn get_size_or_default_audio(content: &AudioMessageEventContent) -> usize {
    let mut size: i32 = 0;
    if let Some(info) = content.info.as_ref() {
        if let Ok(valid_size) = i32::try_from(info.size.unwrap_or(UInt::new(0).unwrap())) {
            size = valid_size;
        }
    }
    return usize::try_from(size).expect("Matrix file size to be a usize");
}

fn get_size_or_default_image(content: &ImageMessageEventContent) -> usize {
    let mut size: i32 = 0;
    if let Some(info) = content.info.as_ref() {
        if let Ok(valid_size) = i32::try_from(info.size.unwrap_or(UInt::new(0).unwrap())) {
            size = valid_size;
        }
    }
    return usize::try_from(size).expect("Matrix file size to be a usize");
}

fn get_size_or_default_video(content: &VideoMessageEventContent) -> usize {
    let mut size: i32 = 0;
    if let Some(info) = content.info.as_ref() {
        if let Ok(valid_size) = i32::try_from(info.size.unwrap_or(UInt::new(0).unwrap())) {
            size = valid_size;
        }
    }
    return usize::try_from(size).expect("Matrix file size to be a usize");
}

