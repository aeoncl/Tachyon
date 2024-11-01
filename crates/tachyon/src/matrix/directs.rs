use std::collections::HashSet;

use log::{debug, error, info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships, StateStoreExt};
use matrix_sdk::deserialized_responses::AnySyncOrStrippedState::Sync;
use matrix_sdk::deserialized_responses::{AnySyncOrStrippedState, RawMemberEvent, SyncOrStrippedState};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::{AnySyncStateEvent, GlobalAccountDataEventType, OriginalSyncStateEvent, SyncStateEvent};
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::events::room::member::{OriginalSyncRoomMemberEvent, RoomMemberEvent, RoomMemberEventContent, StrippedRoomMemberEvent};
use matrix_sdk::ruma::events::StateEvent::Original;
use matrix_sdk::ruma::events::StateEventType::RoomMember;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RoomMapping {
    Direct(OwnedUserId),
    PendingDirect(OwnedUserId),
    Group
}

pub struct RoomMappingInfo {
    id: OwnedRoomId,
    mapping: RoomMapping
}

impl RoomMappingInfo {
    pub fn new(id: OwnedRoomId, mapping: RoomMapping) -> Self {
        Self {
            id,
            mapping
        }
    }
}



pub async fn get_invite_room_mapping_info(room_id: &RoomId, direct_target: &UserId, event: &StrippedRoomMemberEvent , client: &Client) -> Result<RoomMapping,  matrix_sdk::Error> {

    let room = client.get_dm_room(direct_target);

    let is_direct = {
        match event.content.is_direct{
            None => {
                match room.as_ref() {
                    None => {
                        false
                    }
                    Some(room) => {
                        room.is_direct().await?
                    }
                }
            },
            Some(is_direct) => {
                is_direct
            }
        }
    };

    debug!("SYNC|MEMBERSHIPS|INVITE|MAPPING: Room: {} is is direct ? {}", room_id, is_direct);


    if !is_direct {
        return Ok(RoomMapping::Group)
    }

    let is_main_dm_room = {
       match room {
           None => {
               true
           }
           Some(dm_room) => {
               dm_room.room_id() == room_id
           }
       }
    };

    let is_one_on_one = match client.get_room(room_id) {
        None => {
            debug!("WESH: INVITE ROOM NOT FOUND: {}", room_id);
            false
        }
        Some(room) => {

            room.joined_members_count() <= 2
        }
    };

    if is_main_dm_room && is_one_on_one {
        Ok(RoomMapping::Direct(direct_target.to_owned()))
    } else {
        Ok(RoomMapping::Group)
    }
}

pub async fn get_joined_room_mapping_info(room: &Room, me: &UserId, event: &OriginalSyncStateEvent<RoomMemberEventContent>, client: &Client ) -> Result<RoomMapping,  matrix_sdk::Error> {

    let is_direct = {
        match event.content.is_direct {
            None => {
                if room.is_direct().await? {
                    true
                } else {
                    //Room may lack m.direct account data,
                    //Look in our member event, if we got invited, but the room is not fully synced, the is_direct flag will be set on the invite event.
                    if event.state_key == me {
                        if let Some(prev) = event.prev_content() {
                            prev.is_direct.unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        room.get_state_events(RoomMember).await?.iter().find_map(|e| {
                            if let Ok(AnySyncOrStrippedState::Sync(AnySyncStateEvent::RoomMember(mut event))) = e.deserialize() {
                                if let Some(og) = event.as_original() {
                                    if let Some(prev) = og.prev_content() {
                                        prev.is_direct
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                             None
                            }
                        }).unwrap_or(false)
                    }
                }
            },
            Some(is_direct) => {
                is_direct
            }
        }
    };

    debug!("SYNC|MEMBERSHIPS|JOIN|MAPPING: Room: {} is is direct ? {}", room.room_id(), is_direct);

    if !is_direct {
        return Ok(RoomMapping::Group)
    }

    let joined_members_count = {

        if room.joined_members_count() == 0 {
            let join_members = room.members_no_sync(RoomMemberships::JOIN).await?;
            join_members.len() as u64
        } else {
            room.joined_members_count()
        }
    };

    let is_one_on_one= joined_members_count <= 2;

    debug!("SYNC|MEMBERSHIPS|JOIN|MAPPING: Room: {} is_one_on_one? {} ({} joined people)?", room.room_id(), is_one_on_one, joined_members_count);
    if !is_one_on_one {
        return Ok(RoomMapping::Group)
    }

    let direct_target = resolve_direct_target(&room.direct_targets(), room, me, client).await?;

    let is_main_dm_room = match direct_target.as_ref() {
        None => {
            warn!("SYNC|MEMBERSHIPS|JOIN|MAPPING: Room: {} No direct target found.", &room.room_id());
            false
        }
        Some(direct_target) => {
            match client.get_dm_room(&direct_target) {
                None => {
                    true
                }
                Some(dm_room) => {
                    dm_room.room_id() == room.room_id()
                }
            }
        }
    };

    if is_main_dm_room {
        Ok(RoomMapping::Direct(direct_target.expect("to be here")))
    } else {
        Ok(RoomMapping::Group)
    }

}

pub async fn get_left_room_mapping_info(room: &Room, me: &UserId, client: &Client ) -> Result<RoomMapping,  matrix_sdk::Error> {

    //TODO check from ClientData contact list of room was a Group or not.


todo!()

}



pub async fn resolve_direct_target(direct_targets: &HashSet<OwnedUserId>, room: &Room, me: &UserId, client: &Client) -> Result<Option<OwnedUserId>, matrix_sdk::Error> {
    let maybe_found_direct_target = try_fetch_in_direct_targets(direct_targets, me);
    if maybe_found_direct_target.is_some() {
        debug!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Room: {} Direct Target found in direct_targets: {}", room.room_id(), maybe_found_direct_target.as_ref().expect("to be here"));
        return Ok(maybe_found_direct_target);
    }

    for member in room.members_no_sync(RoomMemberships::JOIN).await? {
        if member.user_id() != me {
            return Ok(Some(member.user_id().to_owned()));
        }
    }


    debug!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Room: {} Direct Target not found", room.room_id());
    return Ok(None);
}


fn try_fetch_in_direct_targets(direct_targets: &HashSet<OwnedUserId>, me: &UserId) -> Option<OwnedUserId> {
    if direct_targets.len() > 2 {
        debug!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Direct Target was more than size 2");
        return None;
    }

    for direct_target in direct_targets {
        if direct_target != me {
            return Some(direct_target.clone());
        }
    }

    return None;
}


pub async fn force_update_rooms_with_fresh_m_direct(client: &Client) -> Result<(), matrix_sdk::Error> {
    if let Some(raw_content) = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
        let mut e = raw_content.deserialize_as::<DirectEventContent>()?;
        for (mut user_id, rooms) in e.0 {
            for room_id in rooms {
                let room = client.get_room(&room_id);
                if let Some(room) = room {
                    room.direct_targets().insert(user_id.clone());
                    room.set_is_direct(true).await?
                }
            }
            }
        }

    return Ok(())

}


async fn fetch_m_direct_account_data(client: &Client) -> Result<Option<DirectEventContent>, matrix_sdk::Error> {

    error!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Fetching m.direct from the server...");

    if let Some(raw_content) = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
        error!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Received m.direct");
        return Ok(Some(raw_content.deserialize_as::<DirectEventContent>()?));
    }

    error!("SYNC|MEMBERSHIPS|JOIN|MAPPING|DIRECT_TARGET: Could not fetch m.direct from the server");
    return Ok(None)

}
