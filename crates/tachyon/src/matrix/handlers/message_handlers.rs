use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::matrix::handlers::context::TachyonContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::message::{FormattedBody, MessageFormat, MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::{Client, Room};
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    let msn_user = room.to_msn_user_lazy().await.unwrap();
    let switchboard = context.client_data.switchboards().get_or_create(room.room_id(), &msn_user);

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
                sender: msn_user.get_email_address().clone(),
                display_name: msn_user.compute_display_name().to_string(),
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