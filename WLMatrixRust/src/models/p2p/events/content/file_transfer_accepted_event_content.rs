use matrix_sdk::ruma::events::room::MediaSource;

#[derive(Clone, Debug)]
pub struct FileTransferAcceptedEventContent {
   pub source: MediaSource,
   pub session_id: u32
}