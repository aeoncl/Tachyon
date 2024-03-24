use crate::shared::models::{msn_object::MsnObject, msn_user::MSNUser, uuid::Uuid};


#[derive(Clone, Debug)]
pub struct MSNObjectRequestedEventContent {
   pub msn_object: MsnObject,
   pub session_id: u32,
   pub call_id: Uuid,
   pub inviter: MSNUser,
   pub invitee: MSNUser
}