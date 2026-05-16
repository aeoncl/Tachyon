use crate::p2p::client::session::{ReceiveFileContent, SessionType};
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::RoomId;
use msnp::shared::models::msn_user::MsnUser;

impl TachyonClient {
    pub async fn receive_file(&self, room_id: &RoomId, inviter: &MsnUser, sender: &MsnUser, file_size: usize, filename: String, media_source: MediaSource) {


        let transport = self.get_or_create_transport(room_id, inviter);
        let (session_id, session) = self.create_session(transport, SessionType::ReceiveFile(ReceiveFileContent {
            sender: inviter.endpoint_id.clone(),
            sender_display_name: sender.compute_display_name().to_string(),
            receiver: self.own_user().endpoint_id,
            media_source,
            file_size,
            filename,
        }));

        session.receive_invite().await;

    }
}
