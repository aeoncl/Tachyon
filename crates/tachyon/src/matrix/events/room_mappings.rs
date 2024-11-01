use std::collections::HashMap;
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::{OwnedRoomId, UserId};
use serde::{Deserialize, Serialize};
use crate::matrix::directs::{RoomMapping, RoomMappingInfo};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "com.tachyon.room.mappings", kind = GlobalAccountData)]
struct RoomMappingsEventContent {
    pub rooms: HashMap<OwnedRoomId, RoomMapping>
}

impl RoomMappingsEventContent {
    pub fn get_direct_mapping_for_user(&self, user_id: &UserId) -> Option<RoomMappingInfo> {

        self.rooms.iter().find_map(|(key, value)| {
            match value {
                RoomMapping::Direct(id) => {
                  if id == user_id {
                    Some(RoomMappingInfo::new(key.clone(), value.clone()))
                  } else {
                      None
                  }

                },
                _ => { None }
            }
        })
    }

    pub fn get_pending_mappings_for_user(&self, user_id: &UserId) -> Vec<RoomMappingInfo> {
        self.rooms.iter().filter_map(|(key, value)| {
            match value {
                RoomMapping::PendingDirect(id) => {
                    if id == user_id {
                        Some(RoomMappingInfo::new(key.clone(), value.clone()))
                    } else {
                        None
                    }

                },
                _ => { None }
            }
        }).collect()
    }

    pub fn get_direct_mappings_for_user(&self, user_id: &UserId) -> Vec<RoomMappingInfo> {
        self.rooms.iter().filter_map(|(key, value)| {
            match value {
                RoomMapping::Direct(id) | RoomMapping::PendingDirect(id) => {
                    if id == user_id {
                        Some(RoomMappingInfo::new(key.clone(), value.clone()))
                    } else {
                        None
                    }

                },
                _ => { None }
            }
        }).collect()
    }


}