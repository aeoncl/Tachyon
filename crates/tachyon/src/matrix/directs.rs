use std::collections::HashSet;

use log::{error, info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::GlobalAccountDataEventType;

pub enum RoomMappingInfo {
    Direct(OwnedUserId),
    Group
}


pub async fn get_room_mapping_info(room: &Room, me: &UserId, client: &Client ) -> Result<RoomMappingInfo,  matrix_sdk::Error> {

    if !room.is_direct().await? {
        return Ok(RoomMappingInfo::Group);
    }

    let direct_target = resolve_direct_target(&room.direct_targets(), room, me, client).await?;

    let is_main_dm_room = match direct_target.as_ref() {
        None => {
            warn!("No direct target for room: {}", &room.room_id());
            false
        }
        Some(direct_target) => {
            match client.get_dm_room(&direct_target) {
                None => {
                    warn!("Direct room couldn't be fetched by get_dm_room method: {}", &room.room_id());
                    false
                }
                Some(dm_room) => {
                    dm_room.room_id() == room.room_id()
                }
            }
        }
    };

    let is_one_on_one= room.joined_members_count() <= 2;

    if is_main_dm_room && is_one_on_one {
        Ok(RoomMappingInfo::Direct(direct_target.expect("to be here")))
    } else {
        Ok(RoomMappingInfo::Group)
    }

}


pub async fn resolve_direct_target(direct_targets: &HashSet<OwnedUserId>, room: &Room, me: &UserId, client: &Client) -> Result<Option<OwnedUserId>, matrix_sdk::Error> {
    let maybe_found_direct_target = try_fetch_in_direct_targets(direct_targets, me);
    if maybe_found_direct_target.is_some() {
        return Ok(maybe_found_direct_target);
    }

    let maybe_found_m_direct = find_direct_target_from_account_data(client, &room.room_id().to_owned()).await?;
    if maybe_found_m_direct.is_some() {
        return Ok(maybe_found_m_direct);
    }

    return Ok(None);
}

fn try_fetch_in_direct_targets(direct_targets: &HashSet<OwnedUserId>, me: &UserId) -> Option<OwnedUserId> {
    if direct_targets.len() > 2 {
        return None;
    }

    for direct_target in direct_targets {
        if direct_target != me {
            return Some(direct_target.clone());
        }
    }

    return None;
}

async fn find_direct_target_from_account_data(client: &Client, room_id: &OwnedRoomId) -> Result<Option<OwnedUserId>, matrix_sdk::Error> {
    if let Some(event_content) =  get_m_direct_account_data(client).await? {

        for (current_user, dm_rooms) in event_content.0 {
            if dm_rooms.contains(room_id) {
                return Ok(Some(current_user));
            }
        }
    }

    return Ok(None);
}

async fn get_m_direct_account_data(client: &Client) -> Result<Option<DirectEventContent>, matrix_sdk::Error> {

    let account_data = client.account().account_data::<DirectEventContent>().await?;
    if let Some(raw_content) = account_data {
        return Ok(Some(raw_content.deserialize()?));
    }

    if let Some(raw_content) = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
        return  Ok(Some(raw_content.deserialize_as::<DirectEventContent>()?));
    }

    error!("Could not fetch account data either from the server or locally");
    return Ok(None)

}
