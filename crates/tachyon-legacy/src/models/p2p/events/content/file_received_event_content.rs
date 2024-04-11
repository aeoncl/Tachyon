use crate::models::p2p::file::File;

#[derive(Clone, Debug)]
pub struct FileReceivedEventContent {
   pub file: File,
}