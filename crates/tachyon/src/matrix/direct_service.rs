use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::Ref;
use futures_util::StreamExt;
use log::{debug, error, info};
use matrix_sdk::{Client, Room, RoomMemberships, SlidingSyncBuilder, SlidingSyncListBuilder};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::direct::{DirectEvent, DirectEventContent};
use matrix_sdk::ruma::events::room::member::{RoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::ListFilters;
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, GlobalAccountDataEventType, StaticEventContent};
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::events::room::create::RoomCreateEvent;
use matrix_sdk::sync::RoomUpdates;
use matrix_sdk_ui::{RoomListService, Timeline};
use matrix_sdk_ui::timeline::RoomExt;
use serde::{Deserialize, Serialize};
use msnp::soap::abch::sharing_service::find_membership::response::Memberships;
use crate::matrix::directs::{OneOnOneDmClient, OneOnOneDmRoom};
use crate::matrix::sync2::TachyonContext;

#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.tachyon.direct_mappings", kind = GlobalAccountData)]
pub struct DirectMappingsEventContent {
    // Map from UserId to RoomId for direct chats
    pub mappings: HashMap<OwnedUserId, OwnedRoomId>,
}

type DirectMappingsHashMap = HashMap<OwnedUserId, OwnedRoomId>;
type DirectMappingsDashMap = Arc<DashMap<OwnedUserId, OwnedRoomId>>;

pub struct DirectService {
    matrix_client: Client,
    direct_mappings: DirectMappingsDashMap,
}

enum MappingDiff {
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
}

impl DirectService {

    pub fn new(matrix_client: Client) -> Self {
        Self {
            matrix_client,
            direct_mappings: Arc::new(Default::default()),
        }
    }

    pub async fn init(&self) -> Result<(), matrix_sdk::Error> {

        //self.matrix_client.force_update_rooms_with_fresh_m_direct().await?;


        //let mut list = SlidingSyncListBuilder::new("directs").filters()

        //set is_direct as flag when it'exists
        //let test2= SlidingSyncListBuilder::filters(test, Some(ListFilters { is_invite: Some(true), not_room_types: vec![RoomTypeFilter::Space] }))
        //    .required_state(vec![(RoomMemberEvent::event_type(), "*".to_owned()), (RoomCreateEvent::event_type(), "*".to_owned())])
        //    .timeline_limit(20);



        self.init_account_data_from_cache().await;

        self.register_event_handlers();

        Ok(())
    }

    async fn init_account_data_from_cache(&self) {
        if let Ok(Some(raw_direct)) = self.matrix_client.account().account_data::<DirectMappingsEventContent>().await {
            if let Ok(mut content) = raw_direct.deserialize() {
                for (key, content) in content.mappings.drain() {
                    self.direct_mappings.insert(key, content);
                }
            }
        }
    }

    fn compute_mappings_diff(&self, new_mappings: &DirectMappingsHashMap) -> Vec<MappingDiff> {
        let mut out = Vec::new();

        for (new_mapping_user, new_mapping_room) in new_mappings.iter() {

            match self.direct_mappings.get(new_mapping_user) {
                None => {
                    // New mapping
                    out.push(MappingDiff::NewMapping(new_mapping_user.clone(), new_mapping_room.clone()));
                }
                Some(existing_mapping) => {
                    if existing_mapping.value() != new_mapping_room {
                        // Updated mapping
                        out.push(MappingDiff::UpdatedMapping(new_mapping_user.clone(), new_mapping_room.clone()));
                    }
                }
            }
        }

        for item in self.direct_mappings.iter() {
            if !new_mappings.contains_key(item.key()) {
                // Removed mapping
                out.push(MappingDiff::RemovedMapping(item.key().clone()));
            }
        }

        out
    }

    fn register_event_handlers(&self) {



        let directs_clone = self.direct_mappings.clone();

        self.matrix_client.add_event_handler(|event: DirectMappingsEvent | async move {
            let directs = directs_clone;

            info!("Received org.tachyon.direct_mappings event with {} mappings", event.content.mappings.len());
            directs.clear();

            for (user_id, room_id) in event.content.mappings {
                directs.insert(user_id, room_id);
            }
            debug!("Updated direct mappings from account data event");

        });

        let client_cloned = self.matrix_client.clone();
        let direct_mappings_cloned = self.direct_mappings.clone();
        self.matrix_client.add_updates_handler( |room_updates: RoomUpdates| async move {



            let mut diffs = Vec::new();

            for (room_id, update) in room_updates.joined {
                let maybe_mapping = self.direct_mappings.iter().find(|e| e.value() == &room_id);
                let room = client_cloned.get_room(&room_id).unwrap();
                //Check if update contains room member events

                //Update Existing Mapping if room is no longer a 1o1 direct
                if maybe_mapping.is_some() && !room.is_room_1o1_direct().await.unwrap(){
                    // We have a mapping for this room, it's no longer a 101 dm
                    let direct_target = maybe_mapping.unwrap().key().clone();

                    match client_cloned.get_canonical_dm_room(direct_target.as_ref()).await.unwrap() {
                        None => {
                            diffs.push(MappingDiff::RemovedMapping(direct_target));
                        }
                        Some(room) => {
                            diffs.push(MappingDiff::UpdatedMapping(direct_target, room));
                        }
                    }
                }

                //No mapping yet for this room
                if maybe_mapping.is_none() && room.is_direct().await.unwrap() {
                    if let Some(direct_target) = room.get_1o1_direct_target().await.unwrap() {
                        if let Some(room) = client_cloned.get_canonical_dm_room(direct_target.as_ref()).await.unwrap() {
                            diffs.push(MappingDiff::NewMapping(direct_target, room));
                        }
                    }
                }
            }



            //TODO Handle other room types

            Self::save_mappings_to_account_data(client_cloned, direct_mappings_cloned, diffs).await.unwrap();
            
            
            

        });
    }

    async fn save_mappings_to_account_data(client: Client, direct_mappings: DirectMappingsDashMap, diffs: Vec<MappingDiff>) -> Result<(), matrix_sdk::Error> {
        let room_mappings_from_server = client.account()
            .fetch_account_data(GlobalAccountDataEventType::from("org.tachyon.direct_mappings")).await?
            .map(|raw| raw.deserialize_as::<DirectMappingsEventContent>());


        let mut content = match room_mappings_from_server {
            None | Some(Err(_)) => {
                let mut content = DirectMappingsEventContent::default();
                for entry in direct_mappings.iter() {
                    content.mappings.insert(entry.key().clone(), entry.value().clone());
                }
                content
            }
            Some(Ok(existing_mappings)) => {
                existing_mappings
            }
        };
        
        
        
        
        for diff in diffs {
            diff.apply(&mut content.mappings);
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