use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::message_dedup::SendWithDedup;
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::matrix::handlers::context::TachyonContext;
use crate::switchboard::extensions::CustomStyles;
use crate::tachyon::identifiers::matrix_id_compatible::MatrixIdCompatible;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use matrix_sdk::{Client, Room};
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::control_msg::ControlMessagePayload;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    if room.is_event_deduped(event.event_id.as_ref()).await {
        return;
    }

    let room_user = room.to_msn_user_lazy().await.unwrap();
    let switchboard = context.tachyon_client.switchboards().get_or_initialize(room.room_id(), &room_user);


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
        let mut own_user = context.tachyon_client.own_user();
        own_user.endpoint_id = EndpointId::from_email_addr(own_user.get_email_address().clone());
        own_user
    };


    match event.content.msgtype {
        MessageType::Audio(_) => {}
        MessageType::Emote(_) => {}
        MessageType::File(_) => {}
        MessageType::Image(_) => {}
        MessageType::Location(_) => {}
        MessageType::Notice(message) => {
            
            let msg = SwitchboardServerCommand::MSG(MsgServer {
                sender: message_sender.get_email_address().clone(),
                display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
                payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_notice_style(&message.body)),
            }
            );

            switchboard.send_command(msg).await.unwrap();

        }
        MessageType::ServerNotice(_) => {}
        MessageType::Text(message) => {

            let msg = SwitchboardServerCommand::MSG(MsgServer {
                sender: message_sender.get_email_address().clone(),
                display_name: DisplayName::new_from_ref(message_sender.compute_display_name()),
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

pub(crate) async fn handle_typing_notice(event: SyncTypingEvent, room: Room, context: Ctx<TachyonContext>, client: Client) {
    if let Some(switchboard) = context.tachyon_client.switchboards().get(room.room_id()) {
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

                switchboard.send_command(SwitchboardServerCommand::MSG(MsgServer {
                    sender: sender.get_email_address().clone(),
                    display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                    payload: MsgPayload::Control(ControlMessagePayload::new(sender.get_email_address().clone()))
                })).await;

            }
        }
    }
}