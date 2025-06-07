use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;
use log::{debug, warn};
use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState;
use matrix_sdk::ruma::events::direct::{DirectEventContent, DirectUserIdentifier, OwnedDirectUserIdentifier};
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, GlobalAccountDataEventType, StateEventType};
use matrix_sdk::ruma::{MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::{Client, Error, Room, RoomCreateWithCreatorEventContent, RoomMemberships, RoomState};
use matrix_sdk::ruma::__private_macros::room_id;

struct RoomWithTimestamp {
    room: Room,
    timestamp: MilliSecondsSinceUnixEpoch
}

pub trait OneOnOneDmClient {
    async fn get_canonical_dm_room_id(&self, user_id: &UserId) -> Result<Option<OwnedRoomId>, matrix_sdk::Error>;

    async fn force_update_rooms_with_fresh_m_direct(&self) -> Result<(), matrix_sdk::Error>;
}




async fn find_oldest_1o1_dm_room(matrix_client: &Client, user_id: &UserId) -> Result<Option<OwnedRoomId>, Error> {
    const LOG_LABEL: &str = "FindCanonical |";

    debug!("{} {}", LOG_LABEL, user_id);

    let joined_dm_rooms = {
        let mut joined_rooms = matrix_client.joined_rooms();
        let mut dm_rooms = Vec::new();
        for room in joined_rooms.drain(..) {
            debug!("{} Checking room {}", LOG_LABEL, room.room_id());

            if let Some(direct_target) = extract_o1o_direct_target(&room).await? {
                if &direct_target == user_id {
                    match room.creation_timestamp().await? {
                        None => {
                            warn!("Joined room didnt have creation timestamp, skipping: {}", room.room_id());
                        }
                        Some(timestamp) => {
                            dm_rooms.push(RoomWithTimestamp {
                                room,
                                timestamp,
                            })
                        }
                    };
                }
            }
        }
        dm_rooms.sort_by( |a, b|  {
            a.timestamp.cmp(&b.timestamp)
        });
        dm_rooms
    };

    let oldest_joined_dm_room = joined_dm_rooms.first().map( |room| room.room.room_id().to_owned() );
    if let Some(found) = oldest_joined_dm_room {
        return Ok(Some(found));
    }

    //We did not find any suitable room in joined rooms, look at invites.

    let invited_dm_rooms = {
        let mut invited_rooms = matrix_client.invited_rooms();
        let mut invited_dm_rooms = Vec::new();
        for room in invited_rooms.drain(..) {
            if let Some(direct_target) = extract_o1o_direct_target(&room).await? {
                if &direct_target == user_id {
                    invited_dm_rooms.push(room);
                }
            }
        }

        invited_dm_rooms.sort_by( |a, b|  {a.room_id().cmp(&b.room_id())});
        invited_dm_rooms
    };

    let first_invite_dm_room = invited_dm_rooms.first().map( |room| room.room_id().to_owned() );
    Ok(first_invite_dm_room)

}

impl OneOnOneDmClient for Client {
    async fn get_canonical_dm_room_id(&self, user_id: &UserId) -> Result<Option<OwnedRoomId>, Error> {
        find_oldest_1o1_dm_room(&self, user_id).await
    }

    async fn force_update_rooms_with_fresh_m_direct(&self) -> Result<(), Error> {
        if let Some(raw_content) = self.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
            let mut e = raw_content.deserialize_as::<DirectEventContent>()?;
            for (mut user_id, rooms) in e.0 {
                for room_id in rooms {
                    let room = self.get_room(&room_id);
                    if let Some(room) = room {
                        room.direct_targets().insert(user_id.clone());
                        room.set_is_direct(true).await?
                    }
                }
            }
        }

        return Ok(())
    }
}

pub trait TachyonRoomExtensions {

    async fn is_room_1o1_direct(&self) -> Result<bool, matrix_sdk::Error>;

    async fn get_1o1_direct_target(&self) -> Result<Option<OwnedUserId>, Error>;

    async fn creation_timestamp(&self) -> Result<Option<MilliSecondsSinceUnixEpoch>, matrix_sdk::Error>;
}


async fn extract_o1o_direct_target(room: &Room) -> Result<Option<OwnedUserId>, Error> {

    const LOG_LABEL: &str = "FindDirectTarget |";
    debug!("{} {}", LOG_LABEL, room.room_id());

    if !room.is_direct().await? {
        debug!("{} Room {} not a direct, aborting...", LOG_LABEL, room.room_id());
        return Ok(None);
    }
    
    let active_members = match room.members(RoomMemberships::ACTIVE).await {
        Ok(active_members) => Ok(active_members),
        Err(err) => {
            if RoomState::Joined != room.state() {
                room.members_no_sync(RoomMemberships::ACTIVE).await
            } else {
                Err(err)
            }
        }
    }?;
    
    debug!("{} Room {} active members ({}): {:?}", LOG_LABEL, room.room_id(), active_members.len(), &active_members);

    if active_members.len() > 2 {
        debug!("{} Room {} active members was more than 2, aborting...", LOG_LABEL, room.room_id());
        return Ok(None);
    }

    let me_user_id = room.own_user_id();
    let direct_targets = room.direct_targets();
    let mut not_me_direct_targets = direct_targets.iter().filter_map(|target| {

        if let Some(target_user_id) = target.as_user_id() {
            if target_user_id != me_user_id {
                Some(target_user_id.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } ).collect::<Vec<_>>();

    debug!("{} Room {} not me direct_targets({}): {:?}", LOG_LABEL, room.room_id(), not_me_direct_targets.len(), &not_me_direct_targets);

    if not_me_direct_targets.is_empty() || not_me_direct_targets.len() > 1 {
        debug!("{} Room {} not me direct_targets is empty or more than one, aborting...", LOG_LABEL, room.room_id());

        return Ok(None);
    }

    let direct_target = not_me_direct_targets.remove(0);

    debug!("{} Room {} Direct Target: {}", LOG_LABEL, room.room_id(), &direct_target);
    Ok(Some(direct_target))
}

impl TachyonRoomExtensions for Room {
    async fn is_room_1o1_direct(&self) -> Result<bool, Error> {
        Ok(self.get_1o1_direct_target().await?.is_some())
    }

    async fn get_1o1_direct_target(&self) -> Result<Option<OwnedUserId>, Error> {
        Ok(extract_o1o_direct_target(self).await?)
    }

    //TODO remove store access each time( store it in room.create_content)
    async fn creation_timestamp(&self) -> Result<Option<MilliSecondsSinceUnixEpoch>, Error> {
        let room_create_event = self.get_state_event(StateEventType::RoomCreate, "").await?.expect("RoomCreateEvent to be present");

        match room_create_event {
            RawAnySyncOrStrippedState::Sync(raw_sync) => {
                if let Ok(AnySyncStateEvent::RoomCreate(room_create_event)) = raw_sync.deserialize() {
                    return Ok(Some(room_create_event.origin_server_ts()));
                }
            }
            RawAnySyncOrStrippedState::Stripped(raw_stripped) => {
                if let Ok(AnyStrippedStateEvent::RoomCreate(room_create_event)) = raw_stripped.deserialize() {
                    return Ok(None);
                }
            }
        }

        return Err(Error::InsufficientData);
    }
}

pub type DirectsHashMap = HashMap<OwnedUserId, Vec<OwnedRoomId>>;


pub enum DirectDiff {
    RoomRemoved(OwnedUserId, OwnedRoomId),
    RoomAdded(OwnedUserId, OwnedRoomId)
}

pub trait TachyonDirectAccountDataContent {

    fn into_hashmap(self) -> DirectsHashMap;

    fn from_hashmap(hashmap: &DirectsHashMap) -> Self;

    fn compute_diff(&self, other: &Self) -> Vec<DirectDiff>;

}

impl TachyonDirectAccountDataContent for DirectEventContent {
    fn into_hashmap(self) -> DirectsHashMap {
            let mut directs = HashMap::with_capacity(self.0.len());

            for (user_id, rooms) in self.0.into_iter() {
                let user_id_str = user_id.as_str().to_owned();
                match user_id.into_user_id() {
                    None => {
                        warn!("malformed user_id in m.directs account_data: '{}'", &user_id_str)
                    }
                    Some(user_id) => {
                        directs.insert(user_id, rooms);
                    }
                }
            }

            directs
        }

    fn from_hashmap(hashmap: &DirectsHashMap) -> Self {
        let mut map = BTreeMap::new();

        for (key, value) in hashmap {
            map.insert(key.into(), value.clone());
        }

        Self{
            0: map,
        }
    }

    fn compute_diff(&self, other: &Self) -> Vec<DirectDiff> {
        let mut diffs = Vec::new();
        for (other_user_id, other_rooms) in &other.0 {
            if let None = other_user_id.as_user_id() {
                warn!("malformed user_id in m.directs account_data: '{}', skipping.", &other_user_id.as_str());
                continue;
            }

            let other_user_id_parsed = other_user_id.as_user_id().unwrap();
            match self.0.get(other_user_id) {
                None => {
                    let mut diffs_to_add = other_rooms.iter().map(|room| DirectDiff::RoomAdded(other_user_id_parsed.to_owned(), room.clone()) ).collect::<Vec<_>>();
                    diffs.append(&mut diffs_to_add);
                }
                Some(self_rooms) => {
                    for other_room in other_rooms {
                        if !self_rooms.contains(other_room) {
                            diffs.push(DirectDiff::RoomAdded(other_user_id_parsed.to_owned(), other_room.clone()));
                        }
                    }

                }
            }

        }

        for (self_user_id, self_rooms) in &self.0 {
            if let None = self_user_id.as_user_id() {
                warn!("malformed user_id in m.directs account_data: '{}', skipping.", &self_user_id.as_str());
                continue;
            }

            let self_user_id_parsed = self_user_id.as_user_id().unwrap();
            match other.0.get(self_user_id) {
                None => {
                    let mut diffs_to_add = self_rooms.iter().map(|room| DirectDiff::RoomRemoved(self_user_id_parsed.to_owned(), room.clone()) ).collect::<Vec<_>>();
                    diffs.append(&mut diffs_to_add);
                }
                Some(other_rooms) => {
                    for self_room in self_rooms {
                        if !other_rooms.contains(self_room) {
                            diffs.push(DirectDiff::RoomRemoved(self_user_id_parsed.to_owned(), self_room.clone()));
                        }
                    }
                }
            }

        }

        diffs
    }
}