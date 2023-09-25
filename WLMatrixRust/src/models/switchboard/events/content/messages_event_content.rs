use crate::models::{msg_payload::MsgPayload, msn_user::MSNUser};

#[derive(Clone, Debug)]

pub struct MessagesEventContent {
    pub msgs: Vec<MsgPayload>, 
    pub sender: MSNUser
}