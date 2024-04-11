
use crate::{p2p::v2::pending_packet::PendingPacket, shared::models::msn_user::MSNUser};

use super::{p2p_session_type::P2PSessionType, p2p_status::P2PSessionStatus};

#[derive(Clone, Debug)]
pub struct P2PSession {
    session_type: P2PSessionType,
    session_id: u32,
    status: P2PSessionStatus,
    inviter: MSNUser,
    invitee: MSNUser,
    content: Option<PendingPacket>,
}

impl P2PSession {

    pub fn new(session_type: P2PSessionType, session_id: u32, inviter: MSNUser, invitee: MSNUser) -> Self {
        P2PSession { session_type, session_id, status: P2PSessionStatus::WAITING, inviter, invitee, content: None }
    }

    pub fn set_session_id(&mut self, session_id: u32) {
        self.session_id = session_id;
    }

    pub fn get_session_id(&self) -> u32 {
        self.session_id
    }

    pub fn get_status(&self) -> P2PSessionStatus {
        self.status
    }

    pub fn get_inviter(&self) -> MSNUser {
        self.inviter.clone()
    }

    pub fn get_invitee(&self) -> MSNUser {
        self.invitee.clone()
    }

    pub fn get_content(&self) -> Option<&PendingPacket> {
        self.content.as_ref()
    }

    pub fn get_content_as_mut(&mut self) -> Option<&mut PendingPacket> {
        self.content.as_mut()
    }

    pub fn get_type(&self) -> &P2PSessionType {
        &self.session_type
    }

}