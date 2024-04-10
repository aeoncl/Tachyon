use std::collections::HashSet;
use log::{info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::GlobalAccountDataEventType;

pub async fn resolve_direct_target(direct_targets: &HashSet<OwnedUserId>, room: &Room, me: &UserId, client: &Client) -> Option<OwnedUserId> {
    let maybe_found_direct_target = try_fetch_in_direct_targets(direct_targets, me);
    if maybe_found_direct_target.is_some() {
        return maybe_found_direct_target;
    }

    let maybe_found_m_direct = find_direct_target_from_account_data(client, &room.room_id().to_owned()).await;
    if maybe_found_m_direct.is_some() {
        return maybe_found_m_direct;
    }

    let members = room.members(RoomMemberships::union(RoomMemberships::ACTIVE, RoomMemberships::LEAVE)).await.unwrap();
    log::info!("TryGetDirectTarget2 - members count: {}, me: {}", members.len(), &me);
    for member in members {
        if member.user_id() != me {
            log::info!("TryGetDirectTarget2 - members found: {}", &member.user_id());
            return Some(member.user_id().to_owned());
        }
    }

    return None;
}

fn try_fetch_in_direct_targets(direct_targets: &HashSet<OwnedUserId>, me: &UserId) -> Option<OwnedUserId> {
    log::info!("TryGetDirectTarget - target count: {}, me: {}", direct_targets.len(), &me);
    for direct_target in direct_targets {
        if direct_target != me {
            log::info!("TryGetDirectTarget - found {}", &direct_target);
            return Some(direct_target.clone());
        }
    }
    log::info!("TryGetDirectTarget - found none");
    return None;
}

async fn find_direct_target_from_account_data(client: &Client, room_id: &OwnedRoomId) -> Option<OwnedUserId> {

    info!("find_direct_target_from_account_data");
    if let Ok(Some(event_content)) =  get_m_direct_account_data(client).await {

        for (current_user, dm_rooms) in event_content.0 {
            if dm_rooms.contains(room_id) {
                info!("find_direct_target_from_account_data: Found: {}", &room_id);
                return Some(current_user)
            }
        }
    }
    info!("find_direct_target_from_account_data was None");
    return None;

}

async fn get_m_direct_account_data(client: &Client) -> Result<Option<DirectEventContent>, Error> {

    if let Some(raw_content) = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
        return  Ok(Some(raw_content.deserialize_as::<DirectEventContent>()?));
    } else {
        warn!("fetched account data was none");
        return Ok(None)
    }
}
