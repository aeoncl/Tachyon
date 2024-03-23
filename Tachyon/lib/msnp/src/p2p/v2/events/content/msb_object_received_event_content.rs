use crate::shared::models::msn_object::MSNObject;


#[derive(Clone, Debug)]
pub struct MSNObjectReceivedEventContent {
    pub msn_object: MSNObject,
    pub file_content: Vec<u8>
}