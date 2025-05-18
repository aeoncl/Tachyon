use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, RwLockWriteGuard};
use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::Ref;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships, RoomState, SlidingSyncBuilder, SlidingSyncListBuilder};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::locks::RwLock;
use matrix_sdk::ruma::events::direct::{DirectEvent, DirectEventContent};
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEvent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::ListFilters;
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, GlobalAccountDataEventType, StaticEventContent};
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::events::room::create::RoomCreateEvent;
use matrix_sdk::ruma::events::room::tombstone::SyncRoomTombstoneEvent;
use matrix_sdk::sync::RoomUpdates;
use matrix_sdk_ui::{RoomListService, Timeline};
use matrix_sdk_ui::timeline::RoomExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use msnp::soap::abch::sharing_service::find_membership::response::Memberships;
use crate::matrix::directs::direct_extensions::{DirectDiff, DirectsHashMap, OneOnOneDmClient, TachyonDirectAccountDataContent, TachyonRoomExtensions};
use crate::matrix::sync2::TachyonContext;

#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.tachyon.direct_mappings", kind = GlobalAccountData)]
pub struct DirectMappingsEventContent {
    // Map from UserId to RoomId for direct chats
    pub mappings: HashMap<OwnedUserId, OwnedRoomId>,
}


impl DirectMappingsEventContent {

    pub fn get_room_ids(&self) -> Vec<&RoomId> {
        self.mappings.values()
            .filter_map(|val| Some(val.as_ref()))
            .collect::<Vec<_>>()
    }

    // pub fn get_mapping_room_for_user(&self, user_id: &UserId) -> Option<RoomMappingType> {
    //     self.mappings.get(user_id)
    //         .map_or(None, |val| if val.is_none() { Some(RoomMappingType::Orphan) } else { Some(RoomMappingType::Room(val.clone().unwrap())) } )
    //
    // }

    pub fn get_contact_for_room(&self, room_id: &RoomId) -> Option<OwnedUserId> {
        self.mappings.iter().find_map(|(key, value)| {
            if value == room_id {
                Some(key.clone())
            } else {
                None
            }
        } )
    }

}

#[derive(Clone, Debug)]

pub enum MappingDiff {
    NewMapping(OwnedUserId, OwnedRoomId),
    UpdatedMapping(OwnedUserId, OwnedRoomId),
    RemovedMapping(OwnedUserId)
}

impl MappingDiff {
   pub fn apply(mut self, output: &mut DirectMappingsHashMap) {
        match self {
            MappingDiff::NewMapping(user_id, room_id) => {
                output.insert(user_id, room_id);
            }
            MappingDiff::UpdatedMapping(user_id, room_id) => {
                output.insert(user_id, room_id);
            }
            MappingDiff::RemovedMapping(user_id) => {
                output.remove(&user_id);
            }
        }
   }

   pub fn apply_as_ref(&self, output: &mut DirectMappingsHashMap) {
       self.clone().apply(output);
   }
}

#[derive(Debug)]
pub enum RoomMapping {
    Canonical(OwnedUserId, OwnedRoomId),
    Group
}

struct DirectServiceInner {
    direct_mappings: RwLock<DirectMappingsHashMap>,
    directs: Mutex<DirectEventContent>,
    current_diffs: Mutex<Vec<MappingDiff>>,
    mappings_sender : broadcast::Sender<MappingDiff>,
    mappings_receiver : broadcast::Receiver<MappingDiff>,
}

type DirectMappingsHashMap = HashMap<OwnedUserId, OwnedRoomId>;

#[derive(Clone)]
pub struct DirectService {
    inner: Arc<DirectServiceInner>,
    matrix_client: Client
}


impl DirectService {

    pub fn default(matrix_client: Client) -> Self {
        Self::new(HashMap::new(), Default::default(), matrix_client)
    }

    pub fn new(mappings:  DirectMappingsHashMap, directs: DirectEventContent, matrix_client: Client) -> Self {

        let (mappings_sender, mappings_receiver) = broadcast::channel(100);

        Self {
            inner: Arc::new(DirectServiceInner {
                direct_mappings: RwLock::new(mappings),
                directs: Mutex::new(directs),
                current_diffs: Default::default(),
                mappings_sender,
                mappings_receiver,
            }),
            matrix_client
        }

    }

    pub async fn new_init_from_cache(matrix_client: Client) -> Result<Self,  matrix_sdk::Error> {
        let cached_mappings = Self::load_direct_mappings_from_cache(&matrix_client).await?;
        let directs = Self::load_directs_from_cache(&matrix_client).await?;
        Ok(Self::new(cached_mappings, directs, matrix_client))
    }

    async fn load_directs_from_cache(matrix_client: &Client) ->  Result<DirectEventContent, matrix_sdk::Error> {
        debug!("Loading direct mappings from cache...");

        if let Ok(Some(raw_direct)) = matrix_client.account().account_data::<DirectEventContent>().await {

            if let Ok(content) = raw_direct.deserialize() {
                return Ok(content);
            }
        }

        info!("Found no directs in cache.");
        return Ok(DirectEventContent::default());
    }

    async fn load_direct_mappings_from_cache(matrix_client: &Client) ->  Result<DirectMappingsHashMap, matrix_sdk::Error> {
        debug!("Loading direct mappings from cache...");

        if let Ok(Some(raw_direct)) = matrix_client.account().account_data::<DirectMappingsEventContent>().await {
            if let Ok(mut content) = raw_direct.deserialize() {
                info!("Found {:?} direct mappings in cache.",  &content.mappings.len());
                return Ok(content.mappings)
            }
        }

        info!("Found no direct mappings in cache.");
        return Ok(DirectMappingsHashMap::default());
    }



    pub fn get_mapping_for_user(&self, user_id: &UserId) -> RoomMapping {
        match self.inner.direct_mappings.read().get(user_id).cloned() {
            None => {
                RoomMapping::Group
            }
            Some(found_room) => {
                RoomMapping::Canonical(user_id.to_owned(), found_room)
            }
        }
    }

    pub fn get_mapping_for_room(&self, query_room: &RoomId) -> RoomMapping {
        let maybe = self.inner.direct_mappings.read().iter().find(|(user_id, room_id)| room_id == &query_room).map(|(user_id, room_id)| (user_id.clone(), room_id.clone()));

        match maybe {
            None => {
                RoomMapping::Group
            }
            Some((user_id, room_id)) => {
                RoomMapping::Canonical(user_id, room_id)
            }
        }

    }

    fn compute_mappings_diff(old_mappings: &DirectMappingsHashMap, new_mappings: &DirectMappingsHashMap) -> Vec<MappingDiff> {
        let mut out = Vec::new();

        for (new_mapping_user, new_mapping_room) in new_mappings.iter() {

            match old_mappings.get(new_mapping_user) {
                None => {
                    // New mapping
                    out.push(MappingDiff::NewMapping(new_mapping_user.clone(), new_mapping_room.clone()));
                }
                Some(existing_mapping) => {
                    if existing_mapping != new_mapping_room {
                        // Updated mapping
                        out.push(MappingDiff::UpdatedMapping(new_mapping_user.clone(), new_mapping_room.clone()));
                    }
                }
            }
        }

        for (user_id, room_id) in old_mappings.iter() {
            if !new_mappings.contains_key(user_id) {
                // Removed mapping
                out.push(MappingDiff::RemovedMapping(user_id.clone()));
            }
        }

        out
    }



    pub(crate) async fn handle_directs_update(&self, new_directs: DirectEventContent) -> Result<(), anyhow::Error> {
        let diffs = {
            let mut old_directs = self.inner.directs.lock().unwrap();
             old_directs.compute_diff(&new_directs)
        };

        for diff in diffs.into_iter() {
            match diff {
                DirectDiff::RoomRemoved(user_id, room_id) => {

                    let found_room = {
                        self.inner.direct_mappings.read().get(&user_id).cloned()
                    };

                    if let Some(found_room) = found_room {
                        if(found_room == room_id) {
                            match self.matrix_client.get_room(&found_room) {
                                None => {
                                    self.inner.current_diffs.lock().unwrap().push(MappingDiff::RemovedMapping(user_id));
                                }
                                Some(room) => {
                                    self.evaluate_mapping(&room).await?;
                                }
                            }
                        }
                    }

                }
                DirectDiff::RoomAdded(user_id, room_id) => {
                    let found_room = {
                        self.inner.direct_mappings.read().get(&user_id).cloned()
                    };

                    if found_room.is_none() {
                        match self.matrix_client.get_room(&room_id) {
                            None => {
                                // We don't know about the room yet.
                            }
                            Some(room) => {
                                self.evaluate_mapping(&room).await?;
                            }
                        }
                    }

                }
            }

        }

        let mut old_directs = self.inner.directs.lock().unwrap();
        *old_directs = new_directs;

        Ok(())
    }

    pub async fn evaluate_mapping(&self, room: &Room) -> Result<(), anyhow::Error> {
        if let Some(diff) = self.evaluate_mapping_diff(room).await? {
            self.inner.current_diffs.lock().unwrap().push(diff);
        }

        Ok(())
    }

    async fn evaluate_mapping_diff(&self, room: &Room) -> Result<Option<MappingDiff>, anyhow::Error> {

        let mapping_by_room_id = self.inner
            .direct_mappings
            .read()
            .iter()
            .find(|(user_id, room_id)| *room_id == room.room_id())
            .map(|(user_id, room_id)| (user_id.clone(), room_id.clone()));

        if let Some((mapped_user_id, mapped_room_id)) = mapping_by_room_id {

            let we_are_active_in_room = room.state() == RoomState::Joined || room.state() == RoomState::Invited;
            if !we_are_active_in_room || (room.is_state_fully_synced() && !room.is_direct().await?) {
                //Room is no longer a direct
                return Ok(Some(MappingDiff::RemovedMapping(mapped_user_id)));
            }

            return self.check_if_mapping_has_changed(&mapped_user_id, &mapped_room_id).await;
        }

        match room.get_1o1_direct_target().await? {
            None => {},
            Some(direct_target) => {

                let maybe_user_mapping = {
                    self.inner.direct_mappings.read().get(&direct_target).cloned()
                };

                match maybe_user_mapping {
                    None => {
                        if let Some(found_mapping) = self.matrix_client.get_canonical_dm_room_id(&direct_target).await? {
                            return Ok(Some(MappingDiff::NewMapping(direct_target, found_mapping)));
                        }
                    }
                    Some(mapped_room) => {
                        if mapped_room == room.room_id() {
                            return self.check_if_mapping_has_changed(&direct_target, &mapped_room).await
                        }
                    }
                }
            }
        }


        Ok(None)
    }

    async fn check_if_mapping_has_changed(&self, user_id: &UserId, room_to_compare: &RoomId) -> Result<Option<MappingDiff>, anyhow::Error> {
        match self.matrix_client.get_canonical_dm_room_id(&user_id).await? {
            None => {
                return Ok(Some(MappingDiff::RemovedMapping(user_id.to_owned())));
            }
            Some(found) => {
                if(&found != &room_to_compare) {
                    return Ok(Some(MappingDiff::UpdatedMapping(user_id.to_owned(), found)));
                }
            }
        }

        Ok(None)
    }

    fn compute_effective_diff(mappings: DirectMappingsHashMap, mut diffs: Vec<MappingDiff>)  -> (DirectMappingsHashMap, Vec<MappingDiff>){
        let old_state = mappings;
        let new_state = {
            let mut temp = old_state.clone();

            diffs.into_iter().for_each(|diff| {
                diff.apply(&mut temp)
            });

            temp
        };


        let effective_diff = Self::compute_mappings_diff(&old_state, &new_state);

        (old_state, effective_diff)
    }

    // First apply all the diffs in order, then compute the effective diff between the two snapshots.
    pub async fn apply_pending_mappings(&mut self) -> Result<Vec<MappingDiff>, matrix_sdk::Error> {

        let current_diff = self.inner.current_diffs.lock().unwrap().drain(..).collect::<Vec<_>>();

        let(old_state, effective_diff) = Self::compute_effective_diff(self.inner.direct_mappings.read().iter().map(|(key, value)| (key.clone(), value.clone())).collect(), current_diff);

        Self::save_mappings_to_account_data(&self.matrix_client, &old_state, &effective_diff).await?;

        let mut direct_mappings_write = self.inner.direct_mappings.write();
        effective_diff.iter().for_each(|mapping| {
            mapping.apply_as_ref(&mut direct_mappings_write)
        });

        return Ok(effective_diff);
    }

    async fn save_mappings_to_account_data(client: &Client, mappings: &DirectMappingsHashMap, diffs: &Vec<MappingDiff>) -> Result<(), matrix_sdk::Error> {

        if diffs.is_empty() {
            info!("No mappings to save...");
            return Ok(());
        }

        let room_mappings_from_server = client.account()
            .fetch_account_data(GlobalAccountDataEventType::from("org.tachyon.direct_mappings")).await?
            .map(|raw| raw.deserialize_as::<DirectMappingsEventContent>());


        let mut content = match room_mappings_from_server {
            None | Some(Err(_)) => {
                let mut content = DirectMappingsEventContent::default();
                for (key, value) in mappings.iter() {
                    content.mappings.insert(key.clone(), value.clone());
                }
                content
            }
            Some(Ok(existing_mappings)) => {
                existing_mappings
            }
        };
        

        for diff in diffs {
            diff.apply_as_ref(&mut content.mappings);
        }

        let mappings_count = content.mappings.len();
        // Save to account data
        client.account()
            .set_account_data(
                content
            )
            .await?;

        debug!("Saved {} direct mappings to account data", mappings_count);

        Ok(())
    }



}



struct FakeClient {
}

impl FakeClient {



}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {




    }


}