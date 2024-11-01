use crate::p2p::v2::file::File;


#[derive(Clone, Debug)]
pub struct FileReceivedEventContent {
   pub file: File,
}