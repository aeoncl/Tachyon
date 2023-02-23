use crate::models::{msn_user::MSNUser, msn_object::MSNObject};

#[derive(Clone, Debug)]
pub struct PresenceEventContent {
    pub user: MSNUser
}