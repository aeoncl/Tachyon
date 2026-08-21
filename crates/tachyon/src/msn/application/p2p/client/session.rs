use crate::msn::application::p2p::client::transport::Transport;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::MediaSource;
use msnp::p2p::v2::factories::P2PPayloadFactory;
use msnp::p2p::v2::slp::raw_slp_payload::SlpPayloadFactory;
use msnp::p2p::v2::slp::session_slp_context::PreviewData;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::traits::IntoBytes;
use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use ruma::{OwnedRoomId, RoomId};
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use msnp::shared::models::msn_object::MsnObject;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::uuid::Uuid;

pub type SessionId = u32;

impl TachyonClient {
    pub fn create_session_with_random_id(&self, transport: Transport, session_type: SessionType) -> (SessionId, P2PSession) {
        let session_id: u32 = rand::random();
        self.create_session(transport, session_type, session_id)
    }

    pub fn create_session(&self, transport: Transport, session_type: SessionType, session_id: SessionId) -> (SessionId, P2PSession) {
        let session = P2PSession::new(session_id, transport, session_type);

        self.inner.sessions.insert(session_id, session.clone());
        (session_id, session)
    }

    pub fn get_session(&self, session_id: SessionId) -> Option<P2PSession> {
        self.inner.sessions.get(&session_id).map(|r| r.value().clone())
    }

    pub fn clear_session(&self, session_id: SessionId) {
        self.inner.sessions.remove(&session_id);
        self.inner.chunked_uploads.remove(&session_id);
    }

}

pub struct P2PSessionInner {
    session_id: SessionId,
    transport: Transport,
    session_type: SessionType,
    session_status: Mutex<SessionStatus>,
    call_id: Uuid,
}

#[derive(Clone)]
pub struct P2PSession {
    inner: Arc<P2PSessionInner>
}

#[derive(Debug)]
pub enum SessionStatus {
    Invite,
    Established,
    Cancelled,
    Denied,
}

impl P2PSession {
    pub async fn receive_invite(&self) {
        match &self.inner.session_type {
            SessionType::ReceiveFile(content) => {
                let slp_payload = SlpPayloadFactory::get_file_transfer_request(&content.sender, &content.receiver,  &PreviewData::new(content.file_size, content.filename.clone()), self.inner.session_id, &self.inner.call_id).unwrap();
                let mut packet = P2PPayloadFactory::get_sip_text_message();
                packet.set_payload(slp_payload.into_bytes());
                self.inner.transport.receive_data_packet(&content.sender, &content.sender_display_name, &content.receiver, packet).await;
            }
            SessionType::SendFile(_) => {}
            SessionType::ReceiveMsnObject(content) => {

            }
            SessionType::SendMsnObject(content) => {
                let slp_payload = SlpPayloadFactory::get_msn_object_request(&content.requester, &content.owner, &content.msn_object, self.inner.session_id).unwrap();
                let mut packet = P2PPayloadFactory::get_sip_text_message();
                packet.set_payload(slp_payload.into_bytes());
                self.inner.transport.receive_data_packet(&content.requester.endpoint_id, content.requester.compute_display_name(), &content.owner.endpoint_id, packet).await;
            }
        }
    }

    pub async fn receive_packet(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, packet: RawP2PPayload){
        self.transport().receive_data_packet(sender, sender_display_name, receiver, packet).await;
    }
    
    pub fn session_type(&self) -> &SessionType {
        &self.inner.session_type
    }

    pub fn call_id(&self) -> &Uuid {
        &self.inner.call_id
    }

    pub(crate) fn accept(&self) -> Result<(), anyhow::Error> {
        let mut lock = self.inner.session_status.lock().expect("Not to be poisonned");
        if matches!(*lock, SessionStatus::Invite) {
            *lock = SessionStatus::Established;
            Ok(())
        } else {
            Err(anyhow!("Invalid state transition: trying to go from {:?} to {:?}", *lock, SessionStatus::Established))
        }
    }
}

impl P2PSession {
    pub fn new(session_id: SessionId, transport: Transport, session_type: SessionType) -> Self {
        Self {
            inner: Arc::new(P2PSessionInner {
                session_id,
                transport,
                session_type,
                session_status: Mutex::new(SessionStatus::Invite),
                call_id: Uuid::from_seed(&session_id.to_string()),
            }),
        }
    }

    pub fn transport(&self) -> Transport {
        self.inner.transport.clone()
    }
}

pub enum SessionType {
    ReceiveFile(ReceiveFileContent),
    ReceiveMsnObject(ReceiveMsnObject),
    SendFile(SendFileContent),
    SendMsnObject(SendMsnObjectContent),
}

pub struct ReceiveFileContent {
    pub sender: EndpointId,
    pub sender_display_name: String,
    pub receiver: EndpointId,
    pub media_source: MediaSource,
    pub file_size: usize,
    pub filename: String
}

pub struct ReceiveMsnObject {
    pub msn_object: MsnObject
}

pub struct SendFileContent {
    pub room_id: OwnedRoomId,
    pub file_size: usize,
    pub filename: String

}

pub struct SendMsnObjectContent {
    pub room_id: OwnedRoomId,
    pub requester: MsnUser,
    pub owner: MsnUser,
    pub msn_object: MsnObject,
}