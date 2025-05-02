use std::collections::HashMap;
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::exports::ruma_macros::event_enum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "com.tachyon.room.mappings", kind = GlobalAccountData)]
pub struct RoomMappingsEventContent {
    pub mappings: HashMap<OwnedUserId, Option<OwnedRoomId>>
}

pub enum RoomMappingType {
    Room(OwnedRoomId),
    Orphan,
}

impl RoomMappingsEventContent {

    pub fn get_room_ids(&self) -> Vec<&RoomId> {
        self.mappings.values()
            .filter_map(|val| val.as_ref().map(|room_id| room_id.as_ref()))
            .collect::<Vec<_>>()
    }

    pub fn get_mapping_room_for_user(&self, user_id: &UserId) -> Option<RoomMappingType> {
        self.mappings.get(user_id)
            .map_or(None, |val| if val.is_none() { Some(RoomMappingType::Orphan) } else { Some(RoomMappingType::Room(val.clone().unwrap())) } )

    }

    pub fn get_contact_for_room(&self, room_id: &RoomId) -> Option<OwnedUserId> {
        self.mappings.iter().find_map(|(key, value)| {
           match value {
               None => None,
               Some(r) => {
                   if r == room_id {
                       Some(key.clone())
                   } else {
                       None
                   }
               }
           }
        } )
    }

}