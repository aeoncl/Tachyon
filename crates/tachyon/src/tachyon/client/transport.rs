use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::MediaSource;
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::payload::msg::p2p_msg_payload::P2PMessagePayload;
use rand::random;
use std::sync::{Arc, Mutex};

impl TachyonClient {

    pub fn create_session(&self, session_id: u32, initial_transport: TransportType) -> TransportSession {

        let transport_session = TransportSession::new(session_id, initial_transport);
        self.inner.transport_sessions.insert(session_id, transport_session.clone());
        transport_session
    }

    pub fn get_session(&self, session_id: u32) -> Option<TransportSession>{
        self.inner.transport_sessions.get(&session_id).map(|e| e.value().clone())
    }

    pub fn remove_session(self, session_id: u32) {
        self.inner.transport_sessions.remove(&session_id);
    }

}

pub enum TransportType {
    Switchboard(SwitchboardHandle),
    TCP()
}

impl TransportType {
    pub async fn send_packet(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, packet: P2PTransportPacket) {
        match self {
            TransportType::Switchboard(handle) => {
                let msg = P2PMessagePayload::new(sender.to_owned(), receiver.clone(), packet, Some(sender_display_name.to_string()));
                handle.send_msg(&sender.email_addr, sender_display_name, msg).await;
            }
            TransportType::TCP() => {
                todo!("TCP not yet implemented")
            }
        }
    }
}

pub struct Transport {
    transport_type: TransportType
}

enum TransportSessionStatus {
    Negotiating,
    Ready
}

struct TransportSessionInner {
    sequence_number: Mutex<u32>,
    session_id: u32,
    status: Mutex<TransportSessionStatus>,
    transport: Mutex<TransportType>
}

#[derive(Clone)]
pub struct TransportSession {
    inner: Arc<TransportSessionInner>
}

impl TransportSession {
    pub fn new(session_id: u32, initial_transport: TransportType) -> TransportSession {
        let sequence_number: u32 = random();
        TransportSession {
            inner: Arc::new(TransportSessionInner {
                sequence_number: Mutex::new(sequence_number),
                session_id,
                status: Mutex::new(TransportSessionStatus::Negotiating),
                transport: Mutex::new(initial_transport),
            }),
        }
    }

    pub async fn receive_file(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, filename: String, filesize: usize, media_source: MediaSource) {



    }

    pub fn send_packet(packet: P2PTransportPacket) {

    }

}