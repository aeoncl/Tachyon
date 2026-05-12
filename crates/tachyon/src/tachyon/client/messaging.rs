use matrix_sdk::ruma::RoomId;
use msnp::shared::models::msn_user::MsnUser;
use crate::tachyon::client::tachyon_client::TachyonClient;

impl TachyonClient {
    pub fn receive_file(&self, room_id: &RoomId, inviter: &MsnUser) {

        let switchboard = self.switchboards().get_or_initialize(room_id, inviter);
        



    }
}