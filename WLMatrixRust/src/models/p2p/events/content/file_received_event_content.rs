use crate::models::{p2p::file::File, msn_user::MSNUser};

#[derive(Clone, Debug)]
pub struct FileReceivedEventContent {
   pub file: File,
}