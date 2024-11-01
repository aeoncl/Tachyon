use std::collections::HashMap;
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use serde::{Deserialize, Serialize};
use crate::matrix::directs::RoomMapping;

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "com.tachyon.room.roster", kind = RoomAccountData)]
struct RoomMappingsEventContent {
    pub roster: Vec<OwnedUserId>
}