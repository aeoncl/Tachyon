use log::{debug, error};
use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState;
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent};
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::sync::RoomUpdates;
use crate::matrix::directs::direct_service::MappingDiff;
use crate::matrix::sync2::MappingDiffEvents;
use crate::matrix::utils::EventDeduplicator;
use crate::notification::client_store::ClientData;

pub async fn handle_contacts_room_updates(mut room_updates: RoomUpdates, client_data: ClientData, events_to_reevaluate: Vec<MappingDiffEvents>) {
    let contact_service = client_data.get_contact_service();
    let client = client_data.get_matrix_client();

    let mut event_deduplicator = EventDeduplicator::default();


    for diff_events in events_to_reevaluate {
        for event in diff_events.events {
            match event {
                RawAnySyncOrStrippedState::Sync(event) => {

                    if let Ok(event) = event.deserialize_as::<SyncRoomMemberEvent>() {
                        if event_deduplicator.insert_once(event.event_id()) {
                            contact_service.handle_room_member_event(event, &diff_events.room_id);
                        }
                    }

                }
                RawAnySyncOrStrippedState::Stripped(event) => {

                    if let Ok(event) = event.deserialize_as::<StrippedRoomMemberEvent>() {
                        contact_service.handle_stripped_room_member_event(event, &diff_events.room_id);
                    }

                }
            }
        }
    }

    for (room_id, update) in &room_updates.joined {
        debug!("SYNC|MEMBERSHIPS|JOIN: Handling room: {}: state count: {}", &room_id, update.state.len());

        let room = client.get_room(&room_id).expect("Room to be here");

        for event in update.timeline.events.iter().rev() {
            match event.raw().deserialize() {
                Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(room_member_event))) => {
                    if event_deduplicator.insert_once(room_member_event.event_id()) {
                        contact_service.handle_room_member_event(room_member_event, &room_id);
                    }
                },
                Ok(_) => {
                },
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Couldnt deserialize sync state event: {:?}", e);
                }
            }
        }

        if (update.timeline.limited) {

            for state_event in &update.state {
                let event = state_event.deserialize();
                match event {
                    Ok(AnySyncStateEvent::RoomMember(room_member_event)) => {
                        if event_deduplicator.insert_once(room_member_event.event_id()) {
                            contact_service.handle_room_member_event(room_member_event, &room_id);
                        }
                    },
                    Ok(_) => {
                    },
                    Err(e) => {
                        error!("SYNC|MEMBERSHIPS|JOIN: Couldnt deserialize sync state event: {:?}", e);
                    }
                }
            }

        }
    }

    for (room_id, update) in &room_updates.invited {
        debug!("SYNC|MEMBERSHIPS|INVITE: Handling room: {}: state count: {}", &room_id, update.invite_state.events.len());

        for state_event in &update.invite_state.events {

            match state_event.deserialize() {
                Ok(AnyStrippedStateEvent::RoomMember(stripped_rm_event)) => {
                    debug!("SYNC|MEMBERSHIPS|INVITE: Stripped RoomMemberEvent Received: {:?}", stripped_rm_event);
                    contact_service.handle_stripped_room_member_event(stripped_rm_event, &room_id);
                },
                Ok(other) => {}
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|INVITE: Couldnt deserialize invite room sync state event: {:?}", e);
                },

            }
        }
    }

    for (room_id, update) in &room_updates.left {

        let room = client.get_room(&room_id).expect("Room to be here");


        debug!("SYNC|MEMBERSHIPS|LEAVE: Handling room: {}: state count: {}", &room_id, update.state.len());

        for event in update.timeline.events.iter().rev() {
            match event.raw().deserialize() {
                Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(room_member_event))) => {
                    if event_deduplicator.insert_once(room_member_event.event_id()) {
                        contact_service.handle_room_member_event(room_member_event, &room_id);
                    }
                },
                Ok(_) => {
                },
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Couldnt deserialize sync state event: {:?}", e);
                }
            }
        }


        if (update.timeline.limited) {

            for state_event in &update.state {

                match state_event.deserialize() {
                    Ok(AnySyncStateEvent::RoomMember(room_member_event)) => {
                        if event_deduplicator.insert_once(room_member_event.event_id()) {
                            contact_service.handle_room_member_event(room_member_event, &room_id);
                        }
                    },
                    Ok(other) => {
                        debug!("SYNC|MEMBERSHIPS|LEAVE: Received Non Member Event: {:?}", &other);
                    }
                    Err(e) => {
                        error!("SYNC|MEMBERSHIPS|LEAVE: Couldnt deserialize SyncStateEvent: {:?}", &e);
                    },
                }

            }

        }



    }





}