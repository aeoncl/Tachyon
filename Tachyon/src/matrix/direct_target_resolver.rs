use std::collections::HashSet;

use log::{error, info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::GlobalAccountDataEventType;

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
