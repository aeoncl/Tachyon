use crate::models::{msn_user::MSNUser, p2p::pending_packet::PendingPacket};

use super::{p2p_status::P2PSessionStatus, file_transfer_session_content::FileTransferSessionContent};

#[derive(Clone, Debug)]
pub enum P2PSessionType {
    FileTransfer(FileTransferSessionContent),
}