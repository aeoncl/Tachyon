use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::matrix::handlers::context::TachyonContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::message::{FormattedBody, MessageFormat, MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::OwnedUserId;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::message_dedup::SendWithDedup;

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    if room.is_event_deduped(event.event_id.as_ref()) {
        return;
    }
    
    let room_user = room.to_msn_user_lazy().await.unwrap();
    let switchboard = context.client_data.switchboards().get_or_initialize(room.room_id(), &room_user);


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
        let mut own_user = context.client_data.own_user().unwrap();
        own_user.endpoint_id = EndpointId::from_email_addr(own_user.get_email_address().clone());
        own_user
    };


    match event.content.msgtype {
        MessageType::Audio(_) => {}
        MessageType::Emote(_) => {}
        MessageType::File(_) => {}
        MessageType::Image(_) => {}
        MessageType::Location(_) => {}
        MessageType::Notice(_) => {}
        MessageType::ServerNotice(_) => {}
        MessageType::Text(message) => {

            let msg = SwitchboardServerCommand::MSG(MsgServer {
                sender: message_sender.get_email_address().clone(),
                display_name: message_sender.compute_display_name().to_string(),
                payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_default_style(&message.body)),
            }
            );

            switchboard.send_command(msg).await.unwrap();
        }
        MessageType::Video(_) => {}
        MessageType::VerificationRequest(_) => {}
        MessageType::_Custom(_) => {}
        _ => {}
    }

}