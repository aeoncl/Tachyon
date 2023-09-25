use crate::models::{msn_object::MSNObject, msn_user::MSNUser};

#[derive(Clone, Debug)]
pub struct MSNObjectRequestedEventContent {
   pub msn_object: MSNObject,
   pub session_id: u32,
   pub inviter: MSNUser,
   pub invitee: MSNUser
}