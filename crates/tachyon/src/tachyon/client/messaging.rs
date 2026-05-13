use log::debug;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::RoomId;
use msnp::p2p::v2::factories::P2PPayloadFactory;
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use msnp::p2p::v2::slp::session_slp_context::PreviewData;
use msnp::p2p::v2::slp::raw_slp_payload::SlpPayloadFactory;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::p2p_msg_payload::P2PMessagePayload;
use msnp::shared::traits::IntoBytes;
use crate::tachyon::client::transport::TransportType;

impl TachyonClient {
    pub async fn receive_file(&self, room_id: &RoomId, inviter: &MsnUser, sender: &MsnUser, filesize: usize, filename: String, source: MediaSource) {

        let session_id: u32 = rand::random();
        debug!("Create session: {}", session_id);

        let switchboard_handle = self.switchboards().get_or_initialize(room_id, inviter);
        let session = self.create_session(session_id, TransportType::Switchboard(switchboard_handle.clone()));


        let slp_payload = SlpPayloadFactory::get_file_transfer_request(&inviter.endpoint_id, &self.own_user().endpoint_id,  &PreviewData::new(filesize, filename), session_id).unwrap();
        let mut packet = P2PPayloadFactory::get_sip_text_message();
        packet.set_payload(slp_payload.into_bytes());
        let transport_packet = P2PTransportPacket::new(0, Some(packet));

        let msg = P2PMessagePayload::new(inviter.endpoint_id.clone(), self.own_user().endpoint_id.clone(), transport_packet, Some(sender.compute_display_name().to_string()));
        switchboard_handle.send_msg(inviter.get_email_address(), sender.compute_display_name(), msg).await;

    }
}
