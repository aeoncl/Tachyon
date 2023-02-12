use crate::models::{msg_payload::MsgPayload, msn_user::MSNUser};
#[derive(Clone, Debug)]

pub struct MessageEventContent {
    pub msg: MsgPayload, 
    pub sender: MSNUser
}