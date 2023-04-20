use matrix_sdk::ruma::events::room::MediaSource;

use crate::models::msn_user::MSNUser;


#[derive(Clone, Debug)]
pub struct FileTransferAcceptedEventContent {
   pub source: MediaSource,
   pub session_id: u32
}