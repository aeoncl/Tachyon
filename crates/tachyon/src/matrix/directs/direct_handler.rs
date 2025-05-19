use std::any::{Any, TypeId};
use std::collections::HashSet;
use futures_util::FutureExt;
use log::{info, warn};
use matrix_sdk::deserialized_responses::TimelineEventKind;
use matrix_sdk::Room;
use matrix_sdk::ruma::EventId;
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::sync::{RoomUpdates, Timeline};
use matrix_sdk_ui::sync_service::SyncService;
use msnp::msnp::models::contact::Contact;
use msnp::shared::models::capabilities::Capabilities::MSN8User;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::soap::abch::msnab_datatypes::ContactType;
use crate::matrix::directs::direct_service::{DirectService, MappingDiff};
use crate::notification::client_store::{AddressBookContact, ClientData};
use crate::shared::identifiers::MatrixIdCompatible;

const LOG_LABEL: &str = "Handlers::DirectMappings |";


struct EventDeduplicator {
    events: HashSet<String>
}

impl Default for EventDeduplicator {
    fn default() -> Self {
        EventDeduplicator { events: HashSet::new() }
    }
}

impl EventDeduplicator {

    pub fn insert(&mut self, event_id: &EventId) {
        self.events.insert(event_id.to_string());
    }

    pub fn insert_once(&mut self,  event_id: &EventId) -> bool {
        if self.contains(event_id) {
            false
        } else {
            self.insert(event_id);
            true
        }
    }

    pub fn contains(&self, event_id: &EventId) -> bool {
        self.events.contains(event_id.as_str())
    }
}


pub async fn handle_direct_mappings_room_updates(mut room_updates: RoomUpdates, client_data: ClientData) -> Result<Vec<MappingDiff>, anyhow::Error> {

        let mut direct_service = client_data.get_direct_service();
        let matrix_client = client_data.get_matrix_client();
        let mut event_deduplicator = EventDeduplicator::default();

        for (room_id, room_update) in room_updates.joined.into_iter() {
            if state_can_affect_mappings(room_update.state) || timeline_can_affect_mappings(&room_update.timeline) {
                let room = matrix_client.get_room(&room_id).unwrap();
                direct_service.evaluate_mapping(&room).await.unwrap();
            }
        }

        for (room_id, room_update) in room_updates.left.into_iter() {
            if state_can_affect_mappings(room_update.state) || timeline_can_affect_mappings(&room_update.timeline) {
                let room = matrix_client.get_room(&room_id).unwrap();
                direct_service.evaluate_mapping(&room).await.unwrap();
            }

        }

        for (room_id, room_update) in room_updates.invited.into_iter() {
            if stripped_state_can_affect_mappings(room_update.invite_state.events) {
                let room = matrix_client.get_room(&room_id).unwrap();
                direct_service.evaluate_mapping(&room).await.unwrap();
            }
        }

        for (room_id, room_update) in room_updates.knocked.into_iter() {
            if stripped_state_can_affect_mappings(room_update.knock_state.events) {
                let room = matrix_client.get_room(&room_id).unwrap();
                direct_service.evaluate_mapping(&room).await.unwrap();
            }
        }


    let diffs = direct_service.apply_pending_mappings().await.unwrap();

    info!("{} found direct mappings diff: {:?}", LOG_LABEL, diffs);

    Ok(diffs)
}


fn timeline_can_affect_mappings(timeline: &Timeline) -> bool {
    for event in &timeline.events {
        if let TimelineEventKind::PlainText { event }  = &event.kind {
            if let Ok(AnySyncTimelineEvent::State(state_event)) = event.deserialize() {
                match state_event {
                    AnySyncStateEvent::RoomMember(_) | AnySyncStateEvent::RoomTombstone(_) => {
                        return true;
                    },
                    _ => {}
                }

            }
        }
    }

    return false;
}

fn state_can_affect_mappings(raw_events: Vec<Raw<AnySyncStateEvent>>) -> bool{
    for raw_event in raw_events {
        if let Ok(event) = raw_event.deserialize() {

            match event {
                AnySyncStateEvent::RoomMember(_) | AnySyncStateEvent::RoomTombstone(_) => {
                    return true;
                }
                _ => {}
            }

        }
    }

    return false;
}

fn stripped_state_can_affect_mappings(raw_events: Vec<Raw<AnyStrippedStateEvent>>) -> bool {
    for raw_event in raw_events {
        if let Ok(event) = raw_event.deserialize() {
            match event {
                AnyStrippedStateEvent::RoomMember(_) | AnyStrippedStateEvent::RoomTombstone(_) => {
                    return true;
                }
                _ => {}
            }
        }

    }

    return false;

}
