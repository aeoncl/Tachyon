use crate::models::msn_user::MSNUser;


#[derive(Clone, Debug)]

pub struct FileUploadEventContent {
    pub sender: MSNUser,
    pub filename: String,
    pub filesize: usize,
    pub uri: String

}

impl FileUploadEventContent {
    pub fn new(sender: MSNUser, filename: String, uri: String, filesize: usize) -> Self {
        return FileUploadEventContent {sender, filename, uri, filesize};
    }
}