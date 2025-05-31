use crate::matrix::directs::direct_service::{DirectService, MappingDiff};
use crate::matrix::utils::EventDeduplicator;
use crate::notification::client_store::{AddressBookContact, ClientData};
use crate::shared::identifiers::MatrixIdCompatible;
use futures_util::FutureExt;
use log::{info, warn};
use matrix_sdk::deserialized_responses::TimelineEventKind;
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::EventId;
use matrix_sdk::sync::{RoomUpdates, Timeline};
use matrix_sdk::Room;
use matrix_sdk_ui::sync_service::SyncService;
use msnp::msnp::models::contact::Contact;
use msnp::shared::models::capabilities::Capabilities::MSN8User;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::soap::abch::msnab_datatypes::ContactType;
use std::any::{Any, TypeId};
use std::collections::HashSet;

const LOG_LABEL: &str = "Handlers::DirectMappings |";




pub async fn handle_direct_mappings_room_updates(mut room_updates: RoomUpdates, client_data: ClientData) -> Result<Vec<MappingDiff>, anyhow::Error> {

        let mut direct_service = client_data.get_direct_service();
        let matrix_client = client_data.get_matrix_client();

        for (room_id, room_update) in room_updates.joined.into_iter() {
            if state_can_affect_mappings(room_update.state) || timeline_can_affect_mappings(&room_update.timeline) {
                direct_service.compute_mapping(&room_id).await.unwrap();
            }
        }

        for (room_id, room_update) in room_updates.left.into_iter() {
            if state_can_affect_mappings(room_update.state) || timeline_can_affect_mappings(&room_update.timeline) {
                direct_service.compute_mapping(&room_id).await.unwrap();
            }

        }

        for (room_id, room_update) in room_updates.invited.into_iter() {
            if stripped_state_can_affect_mappings(room_update.invite_state.events) {
                direct_service.compute_mapping(&room_id).await.unwrap();
            }
        }

        for (room_id, room_update) in room_updates.knocked.into_iter() {
            if stripped_state_can_affect_mappings(room_update.knock_state.events) {
                direct_service.compute_mapping(&room_id).await.unwrap();
            }
        }


    let diffs = direct_service.commit_pending_mappings().await;

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
