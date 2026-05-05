use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::message_dedup::SendWithDedup;
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::services::session::incoming_message_service::{
    IncomingMessagingService, IncomingTextMessage,
};
use crate::tachyon::services::session::user_service::UserService;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use matrix_sdk::{Client, Room};
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::control_msg::ControlMessagePayload;
use std::sync::Arc;

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    incoming_messaging_portal: Arc<dyn IncomingMessagingService>,
    _tachyon_client: TachyonClient,
    user_service: Arc<dyn UserService>,
    _client: Client,
) {
    if room.is_event_deduped(event.event_id.as_ref()).await {
        return;
    }

    let room_user = user_service
        .resolve_room_proxy_user(room.room_id())
        .await
        .unwrap();
    let message_sender = if event.sender != room.own_user_id() {
        match room.get_single_direct_target() {
            None => room
                .get_member_no_sync(event.sender.as_ref())
                .await
                .unwrap()
                .unwrap()
                .to_msn_user_lazy()
                .await
                .unwrap(),
            Some(_direct_target) => room_user.clone(),
        }
    } else {
        let mut own_user = user_service.own_user();
        own_user.endpoint_id = EndpointId::from_email_addr(own_user.get_email_address().clone());
        own_user
    };

    match event.content.msgtype {
        MessageType::Audio(_) => {}
        MessageType::Emote(_) => {}
        MessageType::File(_) => {}
        MessageType::Image(_) => {}
        MessageType::Location(_) => {}
        MessageType::Notice(message) => incoming_messaging_portal.receive_message(
            message_sender.get_email_address(),
            room.room_id(),
            IncomingTextMessage::new_with_default_style(&message.body),
        ),
        MessageType::ServerNotice(_) => {}
        MessageType::Text(message) => incoming_messaging_portal.receive_notice(
            message_sender.get_email_address(),
            room.room_id(),
            IncomingTextMessage::new_with_default_style(&message.body),
        ),
        MessageType::Video(_) => {}
        MessageType::VerificationRequest(_) => {}
        MessageType::_Custom(_) => {}
        _ => {}
    }
}


pub(crate) async fn handle_typing_notice(
    event: SyncTypingEvent,
    room: Room,
    tachyon_client: TachyonClient,
    _client: Client,
) {
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

                switchboard
                    .send_command(SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::Control(ControlMessagePayload::new(
                            sender.get_email_address().clone(),
                        )),
                    }))
                    .await;
            }
        }
    }
}
