use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::Ordering::Relaxed;
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
    RemovedMapping(OwnedUserId, OwnedRoomId)
}

impl MappingDiff {
   pub fn apply(self, output: &mut DirectMappingsHashMap) {
        match self {
            MappingDiff::NewMapping(user_id, room_id) => {
                output.insert(user_id, room_id);
            }
            MappingDiff::UpdatedMapping(user_id, room_id) => {
                output.insert(user_id, room_id);
            }
            MappingDiff::RemovedMapping(user_id, _) => {
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

const LOG_LABEL: &str = "DirectService |";


pub(in crate::matrix)  struct DirectServiceInner {
    pub(in crate::matrix)  direct_mappings: RwLock<DirectMappingsHashMap>,
    direct_mappings_next_tick: RwLock<DirectMappingsHashMap>,
    directs: Mutex<DirectEventContent>,
    fully_initialized:  AtomicBool,
    mappings_sender : broadcast::Sender<MappingDiff>,
    mappings_receiver : broadcast::Receiver<MappingDiff>,
}

type DirectMappingsHashMap = HashMap<OwnedUserId, OwnedRoomId>;

#[derive(Clone)]
pub struct DirectService {
    pub(in crate::matrix) inner: Arc<DirectServiceInner>,

    matrix_client: Client
}

impl DirectService {

    pub fn default(matrix_client: Client) -> Self {
        Self::new(HashMap::new(), Default::default(), matrix_client)
    }

    pub fn new(mappings: DirectMappingsHashMap, directs: DirectEventContent, matrix_client: Client) -> Self {

        let (mappings_sender, mappings_receiver) = broadcast::channel(100);

        Self {
            inner: Arc::new(DirectServiceInner {
                direct_mappings: RwLock::new(mappings.clone()),
                direct_mappings_next_tick: RwLock::new(mappings),
                directs: Mutex::new(directs),
                fully_initialized: AtomicBool::new(false),
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

    async fn verify_mappings(&mut self) ->  Result<(), matrix_sdk::Error> {
        let mappings = self.inner.direct_mappings_next_tick.read().clone();

        for mapped_room in mappings.values() {
            self.compute_mapping(mapped_room).await.unwrap();
        }

        Ok(())
    }

    pub async fn set_fully_initialized(&mut self, value: bool) {
        self.inner.fully_initialized.store(value, Relaxed);
        if value {
            self.verify_mappings().await.unwrap();
        }
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
                out.push(MappingDiff::RemovedMapping(user_id.clone(), room_id.clone()));
            }
        }

        out
    }

    pub(crate) async fn handle_direct_mappings_update(&self, content: DirectMappingsEventContent) -> Result<(), anyhow::Error> {
        let mut current_mappings = self.inner.direct_mappings_next_tick.write();
        let mut diffs = Self::compute_mappings_diff(&current_mappings, &content.mappings);

        diffs.drain(..).for_each(|mut diff| {
            diff.apply(&mut current_mappings)
        });

        Ok(())
    }

    pub(crate) async fn handle_directs_update(&self, new_directs: DirectEventContent) -> Result<Vec<DirectDiff>, anyhow::Error> {
        let diffs = {
            let mut old_directs = self.inner.directs.lock().unwrap();
             old_directs.compute_diff(&new_directs)
        };

        for diff in diffs.iter() {
            match diff {
                DirectDiff::RoomRemoved(user_id, room_id) => {

                    let found_room = {
                        self.inner.direct_mappings_next_tick.read().get(user_id).cloned()
                    };

                    if let Some(found_room) = found_room {
                        if(&found_room == room_id) {
                            match self.matrix_client.get_room(&found_room) {
                                None => {
                                    MappingDiff::RemovedMapping(user_id.clone(), room_id.clone()).apply(&mut self.inner.direct_mappings_next_tick.write());
                                }
                                Some(room) => {
                                    self.compute_mapping(room.room_id()).await?;
                                }
                            }
                        }
                    }
                }
                DirectDiff::RoomAdded(user_id, room_id) => {
                    let found_room = {
                        self.inner.direct_mappings_next_tick.read().get(user_id).cloned()
                    };

                    if found_room.is_none() {
                        self.compute_mapping(&room_id).await?;
                    }

                }
            }
        }

        let mut old_directs = self.inner.directs.lock().unwrap();
        *old_directs = new_directs;

        Ok(diffs)
    }

    pub async fn compute_mapping(&self, room: &RoomId) -> Result<(), anyhow::Error> {
        if let Some(mut diff) = self.compute_mapping_diff(room).await? {
            diff.apply_as_ref(&mut self.inner.direct_mappings_next_tick.write());
        }
        Ok(())
    }

    async fn compute_mapping_diff(&self, room_id_to_check: &RoomId) -> Result<Option<MappingDiff>, anyhow::Error> {

        debug!("{} Evaluate mapping for room: {}", LOG_LABEL, room_id_to_check);

        let mapping_by_room_id = self.inner
            .direct_mappings_next_tick
            .read()
            .iter()
            .find(|(_, mapped_room_id)| *mapped_room_id == room_id_to_check)
            .map(|(mapped_user_id, mapped_room_id)| (mapped_user_id.clone(), mapped_room_id.clone()));

        debug!("{} Looking for mapping for room_id: {:?}", LOG_LABEL, &mapping_by_room_id);

        if let Some((mapped_user_id, mapped_room_id)) = mapping_by_room_id {
            debug!("{} Mapping by room_id found: {} - {}", LOG_LABEL, &mapped_user_id, &mapped_room_id);
            return self.check_if_mapping_has_changed(&mapped_user_id, &mapped_room_id).await;
        }

        debug!("{} Mapping by room_id not found.", LOG_LABEL);
        let maybe_room = self.matrix_client.get_room(&room_id_to_check);
        if let None =  maybe_room {
            debug!("{} Room {} not known by client, aborting...", LOG_LABEL, room_id_to_check);
            return Ok(None);
        }

        let room = maybe_room.unwrap();
        match room.get_1o1_direct_target().await? {
            None => {
                debug!("{} Room is not o1o direct: {}, aborting...", LOG_LABEL, room_id_to_check);
            },
            Some(direct_target) => {
                debug!("{} Room is o1o direct: {}, target: {}", LOG_LABEL, room_id_to_check, &direct_target);

                let maybe_user_mapping = {
                    self.inner.direct_mappings_next_tick.read().get(&direct_target).cloned()
                };

                match maybe_user_mapping {
                    None => {
                        debug!("{} user mapping not found for target: {}, look for canonical dm_room", LOG_LABEL, &direct_target);
                        if let Some(found_mapping) = self.matrix_client.get_canonical_dm_room_id(&direct_target).await? {
                            debug!("{} found canonical dm_room: {} for target: {}", LOG_LABEL, &found_mapping, &direct_target);
                            debug!("{} new mapping {} - {}", LOG_LABEL, &direct_target, &found_mapping);
                            return Ok(Some(MappingDiff::NewMapping(direct_target, found_mapping)));
                        }
                    }
                    Some(mapped_room) => {
                        debug!("{} user mapping found for target: {} room: {}", LOG_LABEL, &direct_target, &mapped_room);
                        return self.check_if_mapping_has_changed(&direct_target, &mapped_room).await
                    }
                }
            }
        }
        Ok(None)
    }

    async fn check_if_mapping_has_changed(&self, user_id: &UserId, room_to_compare: &RoomId) -> Result<Option<MappingDiff>, anyhow::Error> {
        debug!("{} Check if mapping has changed for user: {} - room: {}", LOG_LABEL, user_id, room_to_compare);
        match self.matrix_client.get_canonical_dm_room_id(&user_id).await? {
            None => {
                debug!("{} Removed Mapping: {} - {}", LOG_LABEL, user_id, room_to_compare);
                return Ok(Some(MappingDiff::RemovedMapping(user_id.to_owned(), room_to_compare.to_owned())));
            }
            Some(found) => {
                debug!("{} Found new room: {} - {}", LOG_LABEL, user_id, &found);
                if(&found != &room_to_compare) {
                    debug!("{} Update Mapping: {} - {}", LOG_LABEL, user_id, &found);
                    return Ok(Some(MappingDiff::UpdatedMapping(user_id.to_owned(), found)));
                }
            }
        }

        Ok(None)
    }

    pub(crate) fn diff(&self) -> Vec<MappingDiff>{
        Self::compute_mappings_diff(&self.inner.direct_mappings.read(), &self.inner.direct_mappings_next_tick.read())
    }

    pub async fn commit_pending_mappings(&mut self) -> Vec<MappingDiff> {

        let effective_diff = self.diff();

        let old_state = {
            let mut old_state = self.inner.direct_mappings.write();
            mem::replace(&mut *old_state, self.inner.direct_mappings_next_tick.read().clone())
        };

        if self.inner.mappings_sender.receiver_count() > 0 {
            effective_diff.iter().for_each(|diff| {
                let _ = self.inner.mappings_sender.send(diff.clone());
            });
        }

        if self.inner.fully_initialized.load(Ordering::Relaxed) {
            if let Err(err) = Self::save_mappings_to_account_data(&self.matrix_client, &old_state, &effective_diff).await{
                error!("{} Could not save Direct Mappings to Account Data: {}", LOG_LABEL, err);
            }
        }

        effective_diff
    }

    async fn save_mappings_to_account_data(client: &Client, mappings: &DirectMappingsHashMap, diffs: &Vec<MappingDiff>) -> Result<(), matrix_sdk::Error> {

        if diffs.is_empty() {
            info!("{} No mappings to save...", LOG_LABEL);
            return Ok(());
        }

        let room_mappings_from_server = client.account()
            .fetch_account_data(GlobalAccountDataEventType::from("org.tachyon.direct_mappings")).await?
            .map(|raw| raw.deserialize_as::<DirectMappingsEventContent>());


        let mut content = match room_mappings_from_server {
            None | Some(Err(_)) => {
                let mut content = DirectMappingsEventContent::default();
                content.mappings = mappings.clone();
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

        debug!("{} Saved {} direct mappings to account data", LOG_LABEL, mappings_count);

        Ok(())
    }



}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::str::FromStr;
    use chrono::Local;
    use env_logger::Builder;
    use log::{debug, LevelFilter};
    use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
    use matrix_sdk::test_utils::logged_in_client_with_server;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};
    use crate::matrix::directs::direct_service::{DirectService, MappingDiff, RoomMapping};
    use std::io::{BufReader, Write};
    use std::path::{Path, PathBuf};
    use anyhow::{anyhow, Error};
    use matrix_sdk::config::SyncSettings;
    use matrix_sdk::ruma::events::direct::DirectEventContent;

    #[tokio::test]
    async fn dm_invite_received() {

        log_print_panics::init();
        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {} @ {}:{}",
                    Local::now().format("%d-%m-%YT%H:%M:%S%.3f"),
                    record.level(),
                    record.args(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                )
            })
            .target(env_logger::Target::Stdout)
            .filter(Some("v2") , LevelFilter::Debug)
            .filter(Some("tachyon") , LevelFilter::Debug)
            .filter(Some("msnp") , LevelFilter::Debug)
            .filter(Some("matrix-sdk"), LevelFilter::Warn)
            .filter(Some("yaserde"), LevelFilter::Warn)
            .filter(None, LevelFilter::Trace)
            .init();


        let json_resolver = JsonResourceResolver::new("dm_invite_received");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let user_id = client.user_id().unwrap();
        let inviter_id = OwnedUserId::from_str("@inviter:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync_invited.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

            client.sync_once(Default::default()).await.unwrap();
            direct_service.compute_mapping(&room_id).await.unwrap();
            let _ = direct_service.commit_pending_mappings().await;
            let mapping = direct_service.get_mapping_for_room(&room_id);
            assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);

            client.sync_once(Default::default()).await.unwrap();
            direct_service.compute_mapping(&room_id).await.unwrap();
            let _ = direct_service.commit_pending_mappings().await;
            let mapping = direct_service.get_mapping_for_room(&room_id);

            assert!(matches!(mapping, RoomMapping::Canonical(ref user, ref room) if user == &inviter_id && room == &room_id), "Expected CanonicalMapping, got {:?}", mapping);
    }

    //Synapse doesnt put the is_direct flag when there is more that 2 people in the room, could be different for other implementations
    #[tokio::test]
    async fn invite_received() {

        let json_resolver = JsonResourceResolver::new("invite_received");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());


        let user_id = client.user_id().unwrap();
        let inviter_id = OwnedUserId::from_str("@inviter:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();


        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync_invited.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_empty_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;


        client.sync_once(Default::default()).await.unwrap();
        direct_service.compute_mapping(&room_id).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);
        assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);

        client.sync_once(Default::default()).await.unwrap();
        direct_service.compute_mapping(&room_id).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);
        assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);
    }

    #[tokio::test]
    async fn dm_invite_received_but_room_was_not_o1o() {
        let json_resolver = JsonResourceResolver::new("dm_invite_received_but_room_was_not_o1o");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let inviter_id = OwnedUserId::from_str("@inviter:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync_invited.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        let thirdwheel = OwnedUserId::from_str("@thirdwheel:test").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_sync_joined.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_members_with_thirdwheel.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        //Got the invite, marked as direct
        client.sync_once(Default::default()).await.unwrap();
        client.sync_once(Default::default()).await.unwrap();

        direct_service.compute_mapping(&room_id).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);

        assert!(matches!(mapping, RoomMapping::Canonical(ref user, ref room) if user == &inviter_id && room == &room_id), "Expected CanonicalMapping, got {:?}", mapping);


        client.sync_once(Default::default()).await.unwrap();
        direct_service.compute_mapping(&room_id).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);

        assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);

    }

    #[tokio::test]
    async fn dm_invite_sent() {

        let json_resolver = JsonResourceResolver::new("dm_invite_sent");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let invitee_id = OwnedUserId::from_str("@invitee:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync_invitee.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);
        assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);

        assert!(matches!(mapping, RoomMapping::Canonical(ref user, ref room) if user == &invitee_id && room == &room_id), "Expected CanonicalMapping, got {:?}", mapping);
    }

    #[tokio::test]
    async fn dm_room_both_joined() {
        let json_resolver = JsonResourceResolver::new("dm_room_both_joined");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);
        assert!(matches!(mapping, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;
        let mapping = direct_service.get_mapping_for_room(&room_id);

        assert!(matches!(mapping, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room_id), "Expected CanonicalMapping, got {:?}", mapping);

    }

    #[tokio::test]
    async fn dm_room_other_leaves() {
        let json_resolver = JsonResourceResolver::new("dm_room_other_leaves");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room_id = OwnedRoomId::from_str("!r00m:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_sync_other_leaves.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;


        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping = direct_service.get_mapping_for_room(&room_id);

        assert!(matches!(mapping, RoomMapping::Canonical(ref user, ref room) if
user == &other_id && room == &room_id), "Expected CanonicalMapping, got {:?}", mapping);
    }

    #[tokio::test]
    async fn multiple_dm_rooms() {

        let json_resolver = JsonResourceResolver::new("multiple_dm_rooms");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room1_id = OwnedRoomId::from_str("!r00m1:localhost").unwrap();
        let room2_id = OwnedRoomId::from_str("!r00m2:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room1_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members_room1.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room2_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_members_room2.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("05_sync_invalidate_first_room.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room1_id), "Expected CanonicalMapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room2_id), "Expected CanonicalMapping, got {:?}", mapping2);
    }

    #[tokio::test]
    async fn multiple_dm_rooms_reversed_evaluation() {

        let json_resolver = JsonResourceResolver::new("multiple_dm_rooms");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room1_id = OwnedRoomId::from_str("!r00m1:localhost").unwrap();
        let room2_id = OwnedRoomId::from_str("!r00m2:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room1_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members_room1.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room2_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_members_room2.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("05_sync_invalidate_first_room.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room1_id), "Expected CanonicalMapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room2_id), "Expected CanonicalMapping, got {:?}", mapping2);
    }

    #[tokio::test]
    async fn refresh_on_m_direct() {

        log_print_panics::init();
        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {} @ {}:{}",
                    Local::now().format("%d-%m-%YT%H:%M:%S%.3f"),
                    record.level(),
                    record.args(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                )
            })
            .target(env_logger::Target::Stdout)
            .filter(Some("v2") , LevelFilter::Debug)
            .filter(Some("tachyon") , LevelFilter::Debug)
            .filter(Some("msnp") , LevelFilter::Debug)
            .filter(Some("matrix-sdk"), LevelFilter::Warn)
            .filter(Some("yaserde"), LevelFilter::Warn)
            .filter(None, LevelFilter::Trace)
            .init();



        let json_resolver = JsonResourceResolver::new("refresh_on_m_direct");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room1_id = OwnedRoomId::from_str("!r00m1:localhost").unwrap();
        let room2_id = OwnedRoomId::from_str("!r00m2:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room1_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members_room1.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room2_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_members_room2.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        //Only rooms, no m.directs
        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        //Only m.direct global account data
        client.sync_once(Default::default()).await.unwrap();
        let direct_event_content = {
            let mut map = BTreeMap::new();
            map.insert(OwnedUserId::from_str("@other:localhost").unwrap().into(), vec!(OwnedRoomId::from_str("!r00m1:localhost").unwrap(), OwnedRoomId::from_str("!r00m2:localhost").unwrap()));
            DirectEventContent{
                0: map,
            }
        };
        direct_service.handle_directs_update(direct_event_content).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room1_id), "Expected CanonicalMapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);
    }

    #[tokio::test]
    async fn refresh_on_m_direct_removal() {

        log_print_panics::init();
        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {} @ {}:{}",
                    Local::now().format("%d-%m-%YT%H:%M:%S%.3f"),
                    record.level(),
                    record.args(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                )
            })
            .target(env_logger::Target::Stdout)
            .filter(Some("v2") , LevelFilter::Debug)
            .filter(Some("tachyon") , LevelFilter::Debug)
            .filter(Some("msnp") , LevelFilter::Debug)
            .filter(Some("matrix-sdk"), LevelFilter::Warn)
            .filter(Some("yaserde"), LevelFilter::Warn)
            .filter(None, LevelFilter::Trace)
            .init();



        let json_resolver = JsonResourceResolver::new("refresh_on_m_direct_removal");
        let (client, server) = logged_in_client_with_server().await;
        let mut direct_service = DirectService::default(client.clone());

        let own_user_id = client.user_id().unwrap();
        let other_id = OwnedUserId::from_str("@other:localhost").unwrap();
        let room1_id = OwnedRoomId::from_str("!r00m1:localhost").unwrap();
        let room2_id = OwnedRoomId::from_str("!r00m2:localhost").unwrap();

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("01_sync.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("02_sync_m_direct_received.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room1_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("03_members_room1.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/_matrix/client/r0/rooms/{}/members", &room2_id)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("04_members_room2.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/r0/sync"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json_resolver.read_json("05_sync_m_direct_removal.json"))
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        //Only rooms, no m.directs
        client.sync_once(Default::default()).await.unwrap();
        let _ = direct_service.compute_mapping(&room2_id).await;
        let _ = direct_service.compute_mapping(&room1_id).await;
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        //Only m.direct global account data
        client.sync_once(Default::default()).await.unwrap();
        let direct_event_content = {
            let mut map = BTreeMap::new();
            map.insert(OwnedUserId::from_str("@other:localhost").unwrap().into(), vec!(OwnedRoomId::from_str("!r00m1:localhost").unwrap(), OwnedRoomId::from_str("!r00m2:localhost").unwrap()));
            DirectEventContent{
                0: map,
            }
        };
        direct_service.handle_directs_update(direct_event_content).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room1_id), "Expected CanonicalMapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping2);

        //Removal of one account data
        client.sync_once(Default::default()).await.unwrap();
        let direct_event_content = {
            let mut map = BTreeMap::new();
            map.insert(OwnedUserId::from_str("@other:localhost").unwrap().into(), vec!(OwnedRoomId::from_str("!r00m2:localhost").unwrap()));
            DirectEventContent{
                0: map,
            }
        };
        direct_service.handle_directs_update(direct_event_content).await.unwrap();
        let _ = direct_service.commit_pending_mappings().await;

        let mapping1 = direct_service.get_mapping_for_room(&room1_id);
        let mapping2 = direct_service.get_mapping_for_room(&room2_id);
        assert!(matches!(mapping1, RoomMapping::Group), "Expected Group Mapping, got {:?}", mapping1);
        assert!(matches!(mapping2, RoomMapping::Canonical(ref user, ref room) if user == &other_id && room == &room2_id), "Expected CanonicalMapping, got {:?}", mapping2);



    }




    pub struct JsonResourceResolver {
        test_name: String
    }

    impl JsonResourceResolver {
        pub fn new(test_name: &str) -> Self {
            Self {
                test_name: test_name.to_string()
            }
        }

        pub fn read_json(&self, file_name: &str) -> serde_json::Value {
            let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "test", "matrix", "directs", &self.test_name, file_name].iter().collect();
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        }

    }



}