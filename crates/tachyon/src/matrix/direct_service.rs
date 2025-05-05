use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use log::{debug, error, info};
use matrix_sdk::{Client, Room, RoomMemberships, SlidingSyncBuilder, SlidingSyncListBuilder};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::direct::DirectEvent;
use matrix_sdk::ruma::events::room::member::{RoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::ListFilters;
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, GlobalAccountDataEventType, StaticEventContent};
use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::events::room::create::RoomCreateEvent;
use matrix_sdk::sync::RoomUpdates;
use matrix_sdk_ui::RoomListService;
use serde::{Deserialize, Serialize};
use msnp::soap::abch::sharing_service::find_membership::response::Memberships;
use crate::matrix::directs::{OneOnOneDmClient, OneOnOneDmRoom};

#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.tachyon.direct_mappings", kind = GlobalAccountData)]
pub struct DirectMappingsEventContent {
    // Map from UserId to RoomId for direct chats
    pub mappings: HashMap<OwnedUserId, OwnedRoomId>,
}

pub struct DirectService {
    matrix_client: Client,
    directs: Arc<DashMap<OwnedUserId, OwnedRoomId>>,
}

impl DirectService {

    pub fn new(matrix_client: Client) -> Self {
        Self {
            matrix_client,
            directs: Arc::new(Default::default()),
        }
    }

    pub async fn init(&self) -> Result<(), matrix_sdk::Error> {

        //self.matrix_client.force_update_rooms_with_fresh_m_direct().await?;


        //let mut list = SlidingSyncListBuilder::new("directs").filters()

        //set is_direct as flag when it'exists
        //let test2= SlidingSyncListBuilder::filters(test, Some(ListFilters { is_invite: Some(true), not_room_types: vec![RoomTypeFilter::Space] }))
        //    .required_state(vec![(RoomMemberEvent::event_type(), "*".to_owned()), (RoomCreateEvent::event_type(), "*".to_owned())])
        //    .timeline_limit(20);



        //self.load_mappings_from_account_data().await?;

        self.register_event_handlers();

        Ok(())
    }

    fn register_event_handlers(&self) {

        let directs = self.directs.clone();
        let client = self.matrix_client.clone();

        self.matrix_client.add_updates_handler({ |room_updates: RoomUpdates, client: Client| async move {

            info!("DEBUGG: Received room updates with {} joined rooms", room_updates.joined.len());

            for (id, room) in room_updates.joined {
              //  let found : Vec<&RoomId> = directs.iter().filter(|entry| entry.value() == &id).map(|elem|  elem.value()).collect();
                //TODO handle this
            //    assert!(found.len() <= 1);
            }

        }});
        
        self.matrix_client.add_event_handler({|event: SyncRoomMemberEvent, room: Room, client: Client| async move {


            let mut save_mappings = false;

                match room.get_1o1_direct_target().await.unwrap() {
                    Some(target) => {
                        // Room is One on One
                        if let Some(canonical_dm_room) = client.get_canonical_dm_room(&target).await.unwrap() {

                            if let Some(mapping) = directs.get(&target) {
                                // We have a mapping for this user, we need to make sure it hasnt changed.
                                if mapping.value() != &canonical_dm_room {
                                    // The mapping has changed, update it
                                    directs.insert(target.clone(), room.room_id().to_owned());
                                    save_mappings = true;
                                    debug!("Updated direct mapping for {} to {}", target, room.room_id());
                                } else {
                                    // The mapping is still valid, do nothing
                                    debug!("Direct mapping for {} to {} is still valid", target, room.room_id());
                                }
                            } else {
                                // No mapping exists yet for this user, create it
                                directs.insert(target.clone(), room.room_id().to_owned());
                                save_mappings = true;
                                debug!("Created direct mapping for {} to {}", target, room.room_id());
                            }
                        }

                    },
                    None => {
                        // Room is Not One on One
                        if let Some(mapping) = directs.iter().find(|e| e.value() == room.room_id()) {
                            save_mappings = true;
                            directs.remove(mapping.key());
                        }

                    }

                }

                // if save_mappings {
                //     // Save the mappings to account data
                //     if let Err(e) = self.save_mappings_to_account_data().await {
                //         error!("Failed to save direct mappings to account data: {}", e);
                //     } else {
                //         debug!("Saved direct mappings to account data");
                //     }
                // }

        }
        }
        );


        let directs_clone = self.directs.clone();

        self.matrix_client.add_event_handler(move |event: DirectMappingsEvent | {
            let directs = directs_clone;

            async move {
                info!("Received org.tachyon.direct_mappings event with {} mappings", event.content.mappings.len());

                directs.clear();
                for (user_id, room_id) in event.content.mappings {
                    directs.insert(user_id, room_id);
                }

                debug!("Updated direct mappings from account data event");
            }
        });



    }

    async fn save_mappings_to_account_data(&self) -> Result<(), matrix_sdk::Error> {
        let mut content = DirectMappingsEventContent::default();

        // Build the mappings from our internal state
        for entry in self.directs.iter() {
                content.mappings.insert(entry.key().clone(), entry.value().clone());
        }

        let mappings_count = content.mappings.len();
        // Save to account data
        self.matrix_client.account()
            .set_account_data(
                content
            )
            .await?;

        debug!("Saved {} direct mappings to account data", mappings_count);

        Ok(())
    }



}