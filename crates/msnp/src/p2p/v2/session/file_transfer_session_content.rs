#[derive(Clone, Debug)]
pub struct FileTransferSessionContent {
    pub filename: String,
    pub filesize: usize,
    pub identifier: Option<String>
}