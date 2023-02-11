use matrix_sdk::ruma::OwnedRoomId;

use crate::models::msn_user::MSNUser;

#[derive(Clone, Debug)]

pub struct UserJoinedEventContent {
    
    instance_id: String,
    user: MSNUser,
    room_id: OwnedRoomId

}

impl UserJoinedEventContent {
    pub fn new(instance_id: String, user: MSNUser, room_id: OwnedRoomId) -> Self {
        return UserJoinedEventContent { instance_id, user, room_id };
    }
}