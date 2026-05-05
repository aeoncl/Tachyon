use crate::tachyon::state::session::tachyon_client_repository::TachyonSessionData;
use matrix_sdk::ruma::{OwnedRoomId, RoomId};
use msnp::shared::models::email_address::EmailAddress;

pub trait RoomProxyRepository: Send + Sync {
    fn insert(&self, email: &EmailAddress, room_id: &RoomId);
    fn get_room_for_email(&self, email_address: &EmailAddress) -> Option<OwnedRoomId>;
}

impl RoomProxyRepository for TachyonSessionData {
    fn insert(&self, email: &EmailAddress, room_id: &RoomId) {
        self.session_data
            .room_proxy_reverse_lookup_table
            .insert(email.to_string(), room_id.to_owned());

        self.session_data
            .room_proxy_lookup_table
            .insert(room_id.to_owned(), email.to_string());
    }


    fn get_room_for_email(&self, email_address: &EmailAddress) -> Option<OwnedRoomId> {
        self.session_data
            .room_proxy_reverse_lookup_table
            .get(email_address.as_str())
            .map(|e| e.value().clone())
    }
}
