use crate::models::msn_user::MSNUser;

#[derive(Clone, Debug)]
pub struct PresenceEventContent {
    pub user: MSNUser,
}