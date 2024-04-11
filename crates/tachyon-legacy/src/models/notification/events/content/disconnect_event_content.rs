use crate::models::msn_user::MSNUser;

#[derive(Clone, Debug)]
pub struct DisconnectEventContent{
    pub msn_user: MSNUser
}