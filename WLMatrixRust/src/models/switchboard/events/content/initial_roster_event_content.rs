use crate::models::msn_user::MSNUser;


#[derive(Clone, Debug)]

pub struct InitialRosterEventContent {
    instance_id: String,

    roster: MSNUser

}

impl InitialRosterEventContent {
    pub fn new(instance_id: String, roster: MSNUser) -> Self {
        return InitialRosterEventContent {roster, instance_id };
    }
}