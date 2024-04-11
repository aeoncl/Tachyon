use matrix_sdk::ruma::events::room::MediaSource;

use crate::models::msn_user::MSNUser;

#[derive(Clone, Debug)]

pub struct FileUploadEventContent {
    pub sender: MSNUser,
    pub receiver: MSNUser,
    pub filename: String,
    pub filesize: usize,
    pub source: MediaSource
}

impl FileUploadEventContent {
    pub fn new(sender: MSNUser, receiver: MSNUser, filename: String, source: MediaSource, filesize: usize) -> Self {
        return FileUploadEventContent {sender, receiver, filename, source, filesize,  };
    }
}