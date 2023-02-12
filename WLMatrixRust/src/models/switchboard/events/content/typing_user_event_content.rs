use crate::models::msn_user::MSNUser;
#[derive(Clone, Debug)]

pub struct TypingUserEventContent {
    pub instance_id: String,

    pub typing_user: MSNUser
}

impl TypingUserEventContent {
    pub fn new(instance_id: String, typing_user: MSNUser) -> Self {
        return TypingUserEventContent {instance_id, typing_user};
    }
}
