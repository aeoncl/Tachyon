use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use log::{debug, error, info};
use matrix_sdk::{Client, Room, RoomMemberships};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::direct::DirectEvent;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, GlobalAccountDataEventType, StaticEventContent};
use matrix_sdk::ruma::events::macros::EventContent;
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


    fn new(matrix_client: Client) -> Self {
        Self {
            matrix_client,
            directs: Arc::new(Default::default()),
        }
    }

    pub async fn init(&self) -> Result<(), matrix_sdk::Error> {

        self.matrix_client.force_update_rooms_with_fresh_m_direct().await?;

        self.load_mappings_from_account_data().await?;

        self.register_event_handlers();

        todo!()

    }

    fn register_event_handlers(&self) {

        let directs = self.directs.clone();

        self.matrix_client.add_event_handler({ move |event: SyncRoomMemberEvent, room: Room, client: Client| async move {


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

                if save_mappings {
                    // Save the mappings to account data
                    if let Err(e) = self.save_mappings_to_account_data().await {
                        error!("Failed to save direct mappings to account data: {}", e);
                    } else {
                        debug!("Saved direct mappings to account data");
                    }
                }

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

    async fn load_mappings_from_account_data(&self) -> Result<(), matrix_sdk::Error> {
        let account = self.matrix_client.account();

        if let Some(raw_content) = account.fetch_account_data(GlobalAccountDataEventType::from(DirectMappingsEventContent::TYPE)).await? {
            let content = raw_content.deserialize_as::<DirectMappingsEventContent>()?;

            for (user_id, room_id) in content.mappings {
                self.directs.insert(user_id, room_id);
            }

            debug!("Loaded {} direct mappings from account data", self.directs.len());
        } else {
            debug!("No direct mappings found in account data");
        }

        Ok(())
    }

    async fn save_mappings_to_account_data(&self) -> Result<(), matrix_sdk::Error> {
        let mut content = DirectMappingsEventContent::default();

        // Build the mappings from our internal state
        for entry in self.directs.iter() {
                content.mappings.insert(entry.key().clone(), entry.value().clone());
        }

        // Save to account data
        self.matrix_client.account()
            .set_account_data(
                content
            )
            .await?;

        debug!("Saved {} direct mappings to account data", content.mappings.len());

        Ok(())
    }



}