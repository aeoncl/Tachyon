use matrix_sdk::ruma::events::room::MediaSource;

#[derive(Clone, Debug)]
pub struct FileTransferSessionContent {
    pub filename: String,
    pub filesize: usize,
    pub source: Option<MediaSource>
}