use log::warn;
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use crate::models::msg_payload::factories::MsgPayloadFactory;
use crate::models::msn_user::MSNUser;
use crate::models::notification::msn_client::MSNClient;
use crate::repositories::msn_user_repository::MSNUserRepository;

pub async fn handle_sync_typing_event(ev: SyncTypingEvent, room: Room, msn_client: MSNClient, user_repo: MSNUserRepository ) {
    let room_id = room.room_id().to_string();

    if let Some(found) = msn_client.get_switchboards().find(&room_id) {
        for user_id in ev.content.user_ids {
            let typing_user = user_repo.get_msnuser(&room.room_id(), &user_id, false).await.unwrap();

            if &typing_user.get_msn_addr() != &msn_client.get_user().get_msn_addr() {
                let typing_user_payload = MsgPayloadFactory::get_typing_user(typing_user.get_msn_addr().clone());
                found.on_message_received(typing_user_payload, typing_user, None);
            }
        }
    } else {
        warn!("Received a typing event but could not find switchboard for room_id: {}", &room_id);
    }
}