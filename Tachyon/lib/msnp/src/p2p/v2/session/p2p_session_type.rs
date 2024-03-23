use crate::shared::models::msn_object::MSNObject;

use super::file_transfer_session_content::FileTransferSessionContent;

#[derive(Clone, Debug)]
pub enum P2PSessionType {
    FileTransfer(FileTransferSessionContent),
    MSNObject(MSNObject)
}