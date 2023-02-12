use crate::models::msn_user::MSNUser;

#[derive(Clone, Debug)]

pub struct PresenceEventContent {
    user: MSNUser,
}