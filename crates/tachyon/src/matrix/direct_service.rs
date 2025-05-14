use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::Ref;
use futures_util::StreamExt;
use log::{debug, error, info};
use matrix_sdk::{Client, Room, RoomMemberships, SlidingSyncBuilder, SlidingSyncListBuilder};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::direct::{DirectEvent, DirectEventContent};
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEvent, SyncRoomMemberEvent};
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
use tokio::sync::broadcast;
use msnp::soap::abch::sharing_service::find_membership::response::Memberships;
use crate::matrix::directs::{OneOnOneDmClient, OneOnOneDmRoom};
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


type DirectMappingsHashMap = HashMap<OwnedUserId, OwnedRoomId>;
type DirectMappingsDashMap = Arc<DashMap<OwnedUserId, OwnedRoomId>>;

pub struct DirectService {
    matrix_client: Client,
    direct_mappings: DirectMappingsDashMap,
    current_diffs: Mutex<Vec<MappingDiff>>,
    mappings_sender : broadcast::Sender<MappingDiff>,
    mappings_receiver : broadcast::Receiver<MappingDiff>,


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
}

impl DirectService {

    pub fn new(matrix_client: Client) -> Self {

        let (mappings_sender, mappings_receiver) = broadcast::channel(100);

        Self {
            matrix_client,
            direct_mappings: Arc::new(Default::default()),
            current_diffs: Default::default(),
            mappings_sender,
            mappings_receiver,
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
        // self.matrix_client.add_updates_handler( |room_updates: RoomUpdates, client: Client| async move {
        //
        //
        //
        //     let mut diffs = Vec::new();
        //
        //     for (room_id, update) in room_updates.joined {
        //         let maybe_mapping = self.direct_mappings.iter().find(|e| e.value() == &room_id);
        //         let room = client_cloned.get_room(&room_id).unwrap();
        //         //Check if update contains room member events
        //
        //         //Update Existing Mapping if room is no longer a 1o1 direct
        //         if maybe_mapping.is_some() && !room.is_room_1o1_direct().await.unwrap(){
        //             // We have a mapping for this room, it's no longer a 101 dm
        //             let direct_target = maybe_mapping.unwrap().key().clone();
        //
        //             match client_cloned.get_canonical_dm_room(direct_target.as_ref()).await.unwrap() {
        //                 None => {
        //                     diffs.push(MappingDiff::RemovedMapping(direct_target));
        //                 }
        //                 Some(room) => {
        //                     diffs.push(MappingDiff::UpdatedMapping(direct_target, room));
        //                 }
        //             }
        //         }
        //
        //         //No mapping yet for this room
        //         if maybe_mapping.is_none() && room.is_direct().await.unwrap() {
        //             if let Some(direct_target) = room.get_1o1_direct_target().await.unwrap() {
        //                 if let Some(room) = client_cloned.get_canonical_dm_room(direct_target.as_ref()).await.unwrap() {
        //                     diffs.push(MappingDiff::NewMapping(direct_target, room));
        //                 }
        //             }
        //         }
        //     }
        //
        //
        //
        //     //TODO Handle other room types
        //
        //     Self::save_mappings_to_account_data(client_cloned, direct_mappings_cloned, diffs).await.unwrap();
        //
        //
        //
        //
        // });
    }

    pub async fn handle_member_event(&self, room_member_event: &SyncRoomMemberEvent, room: &Room) -> Option<MappingDiff> {

        let maybe = self.direct_mappings.iter().find(|e| e.value() == room.room_id());

        let mut out = None;

        // If room is a canonical room & another user joins or leaves, reevaluate the mapping
        if let Some(found) = maybe {
            if (room_member_event.state_key() != found.key() && room_member_event.state_key() != room.own_user_id()) || room_member_event.state_key() == room.own_user_id() {
                match room_member_event.membership() {
                    MembershipState::Ban | MembershipState::Join | MembershipState::Leave => {
                        match self.matrix_client.get_canonical_dm_room(found.key()).await.unwrap() {
                            None => {
                                out = Some(MappingDiff::RemovedMapping(found.key().clone()));
                                self.direct_mappings.remove(found.key());
                            }
                            Some(new_mapping) => {
                                out = Some(MappingDiff::UpdatedMapping(found.key().clone(), new_mapping.clone()));
                                self.direct_mappings.insert(found.key().clone(), new_mapping);

                            }
                        };
                    }
                    MembershipState::Knock | MembershipState::Invite=> {

                    }
                    _ => {}
                }
            }
        } else if room.is_direct().await.unwrap_or(false) {
            //No canonical room mapping yet, Check if we invited someone in a dm room, and if
            if room_member_event.sender() == room.own_user_id() {

                match room_member_event.membership() {
                    MembershipState::Invite => {
                        let direct_target = room.get_1o1_direct_target().await.unwrap();
                        if let Some(direct_target) = direct_target {
                            if room_member_event.state_key() == &direct_target {
                                if let Some(new_mapping) = self.matrix_client.get_canonical_dm_room(direct_target.as_ref()).await.unwrap() {
                                    out = Some(MappingDiff::NewMapping(direct_target.clone(), new_mapping.clone()));
                                    self.direct_mappings.insert(direct_target, new_mapping);
                                }
                            }
                        }
                    }
                    MembershipState::Join | MembershipState::Ban | MembershipState::Knock | MembershipState::Leave => {}
                    _ => {}
                }

            }

        }
        
        return out;
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