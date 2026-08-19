use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::message_dedup::SendWithDedup;
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::switchboard::extensions::CustomStyles;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use log::info;
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use matrix_sdk::{Client, Room};
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::control_msg::ControlMessagePayload;
use msnp::shared::payload::msg::datacast_msg::DatacastMessagePayload;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {

    if room.is_event_deduped(event.event_id.as_ref()).await {
        return;
    }

    let room_user = room.to_msn_user_lazy().await.unwrap();
    let switchboard = tachyon_client.switchboards().get_or_initialize(room.room_id(), &room_user);


    let message_sender = if event.sender != room.own_user_id() {

        match room.get_single_direct_target() {
            None => {
                room.get_member_no_sync(event.sender.as_ref()).await.unwrap().unwrap().to_msn_user_lazy().await.unwrap()
            }
            Some(direct_target) => {
                room_user.clone()
            }
        }

    } else {
        let mut own_user = tachyon_client.own_user();
        own_user.endpoint_id = EndpointId::from_email_addr(own_user.get_email_address().clone());
        own_user
    };


    match &event.content.msgtype {
        MessageType::Audio(audio) => {
            //Audio goes out as a WLM voice clip when it fits in one, and as a plain file otherwise.
            match tachyon_client.prepare_voice_clip(&room_user, audio).await {
                Ok(msn_object) => {
                    let voice_clip = SwitchboardServerCommand::MSG(MsgServer {
                        sender: room_user.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                        payload: MsgPayload::Datacast(DatacastMessagePayload::new_msn_object(msn_object)),
                    });
                    
                    let notice = SwitchboardServerCommand::MSG(MsgServer {
                       sender: message_sender.get_email_address().clone(), 
                        display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload {
                            font_family: Default::default(),
                            right_to_left: false,
                            font_styles: Default::default(),
                            font_color: Default::default(),
                            body: "has sent you an audio message".to_string(),
                        })

                    });
                    
                    switchboard.receive_command(notice).await.unwrap();
                    switchboard.receive_command(voice_clip).await.unwrap();
                }
                Err(e) => {
                    info!("Could not send audio message as a voice clip, falling back to a file transfer: {}", e);

                    let notice = SwitchboardServerCommand::MSG(MsgServer {
                        sender: message_sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload {
                            font_family: Default::default(),
                            right_to_left: false,
                            font_styles: Default::default(),
                            font_color: Default::default(),
                            body: "has sent you a file".to_string(),
                        })
                    });

                    switchboard.receive_command(notice).await.unwrap();

                    let size = audio.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
                    let filename = audio.filename.as_ref().unwrap_or(&audio.body).to_owned();
                    //TODO fix filename
                    tachyon_client.receive_file(room.room_id(), &room_user, &message_sender, size, filename, audio.source.clone()).await;
                }
            }
        }
        MessageType::Emote(emote) => {

        }
        MessageType::File(file) => {
            let size = file.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = file.filename.as_ref().unwrap_or(&file.body).to_owned();
            //TODO fix filename
            tachyon_client.receive_file(room.room_id(), &room_user, &message_sender, size, filename, file.source.clone()).await;
        }
        MessageType::Image(image) => {

            let size = image.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = image.filename.as_ref().unwrap_or(&image.body).to_owned();
            //TODO fix filename
            tachyon_client.receive_file(room.room_id(), &room_user, &message_sender, size, filename, image.source.clone()).await;
        }
        MessageType::Location(_) => {}
        MessageType::Notice(message) => {
            
            let msg = SwitchboardServerCommand::MSG(MsgServer {
                sender: message_sender.get_email_address().clone(),
                display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_notice_style(&message.body)),
            }
            );

            switchboard.receive_command(msg).await.unwrap();

        }
        MessageType::ServerNotice(server) => {

        }
        MessageType::Text(message) => {
            let msg = SwitchboardServerCommand::MSG(MsgServer {
                sender: message_sender.get_email_address().clone(),
                display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_default_style(&message.body)),
            }
            );

            switchboard.receive_command(msg).await.unwrap();
        }
        MessageType::Video(video) => {
            let size = video.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = video.filename.as_ref().unwrap_or(&video.body).to_owned();
            //TODO fix filename
            tachyon_client.receive_file(room.room_id(), &room_user, &message_sender, size, filename, video.source.clone()).await;
        }
        MessageType::VerificationRequest(_) => {}
        MessageType::_Custom(_) => {

            match event.content.msgtype.msgtype() {
                "chat.tachyon.buzz" => {

                    let nudge = SwitchboardServerCommand::MSG(MsgServer {
                        sender: message_sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                        payload: MsgPayload::Datacast(DatacastMessagePayload::new_nudge()),
                    }
                    );

                    switchboard.receive_command(nudge).await.unwrap();
                },
                &_ => {}
            }
        }
        _ => {}
    }

}

pub(crate) async fn handle_typing_notice(event: SyncTypingEvent, room: Room, tachyon_client: TachyonClient, client: Client) {
    if let Some(switchboard) = tachyon_client.switchboards().get(room.room_id()) {
        for user_id in event.content.user_ids.iter() {
            if user_id != room.own_user_id() {

                let sender = {
                    let member = room.get_member_no_sync(&user_id).await;
                    if let Ok(Some(member)) = member {
                        if let Ok(member) = member.to_msn_user_lazy().await {
                            member
                        } else {
                            MsnUser::from_user_id(&user_id)
                        }
                    } else {
                        MsnUser::from_user_id(&user_id)
                    }
                };

                switchboard.receive_command(SwitchboardServerCommand::MSG(MsgServer {
                    sender: sender.get_email_address().clone(),
                    display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                    payload: MsgPayload::Control(ControlMessagePayload::new(sender.get_email_address().clone()))
                })).await;

            }
        }
    }
}