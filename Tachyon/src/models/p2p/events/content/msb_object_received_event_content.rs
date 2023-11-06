use crate::models::{msn_object::MSNObject, msn_user::MSNUser, uuid::UUID};

#[derive(Clone, Debug)]
pub struct MSNObjectReceivedEventContent {
    pub msn_object: MSNObject,
    pub file_content: Vec<u8>
}