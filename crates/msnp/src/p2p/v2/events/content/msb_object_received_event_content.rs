use crate::shared::models::msn_object::MsnObject;


#[derive(Clone, Debug)]
pub struct MSNObjectReceivedEventContent {
    pub msn_object: MsnObject,
    pub file_content: Vec<u8>
}