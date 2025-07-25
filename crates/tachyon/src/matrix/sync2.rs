use core::sync;
use std::collections::HashSet;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use log::{debug, error, info, warn};
use crate::notification::client_store::ClientData;
use matrix_sdk::{Client, Error, Room, SlidingSync, SlidingSyncList, SlidingSyncListBuilder, SlidingSyncMode};
use matrix_sdk::crypto::types::events::olm_v1::AnyDecryptedOlmEvent;
use matrix_sdk::deserialized_responses::{DecryptedRoomEvent, MemberEvent, RawAnySyncOrStrippedState, TimelineEventKind};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::{AccountData, ListFilters, RoomSubscription, ToDevice, Typing, E2EE};
use matrix_sdk::ruma::events::direct::{DirectEvent, DirectEventContent};
use matrix_sdk::ruma::events::{AnyMessageLikeEvent, AnyStrippedStateEvent, AnySyncMessageLikeEvent, AnySyncStateEvent, AnySyncTimelineEvent, GlobalAccountDataEventType, StateEventType, SyncMessageLikeEvent};
use matrix_sdk::ruma::{assign, OwnedRoomId, OwnedUserId, RoomId, UInt, UserId};
use matrix_sdk::ruma::api::client::error::ErrorKind;
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::sleep::sleep;
use matrix_sdk::sliding_sync::Bound;
use matrix_sdk::sync::RoomUpdates;
use matrix_sdk_ui::sync_service::{self, SyncService};
use matrix_sdk_ui::timeline::RoomExt;
use tokio::sync::broadcast::Receiver;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::NotServer;
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::matrix::contacts::contact_handler::handle_contacts_room_updates;
use crate::matrix::directs::direct_extensions::{DirectDiff, TachyonDirectAccountDataContent};
use crate::matrix::directs::direct_handler;
use crate::matrix::directs::direct_service::{DirectMappingsEvent, DirectMappingsEventContent, DirectService, MappingDiff};

#[derive(Clone)]
pub struct TachyonContext {
    client_data: ClientData
}

const REQUIRED_STATE: &[(StateEventType, &str)] = &[
    (StateEventType::RoomName, ""),
    (StateEventType::RoomEncryption, ""),
    (StateEventType::RoomMember, "$LAZY"),
    (StateEventType::RoomMember, "$ME"),
    (StateEventType::RoomTopic, ""),
    (StateEventType::RoomCanonicalAlias, ""),
    (StateEventType::RoomPowerLevels, ""),
    (StateEventType::CallMember, "*"),
    (StateEventType::RoomJoinRules, ""),
    // Those two events are required to properly compute room previews.
    (StateEventType::RoomCreate, ""),
    (StateEventType::RoomHistoryVisibility, ""),
    // Required to correctly calculate the room display name.
    (StateEventType::MemberHints, ""),
];

const DM_REQUIRED_STATE: &[(StateEventType, &str)] = &[
    (StateEventType::RoomName, ""),
    (StateEventType::RoomEncryption, ""),
    (StateEventType::RoomMember, "*"),
    (StateEventType::RoomTopic, ""),
    (StateEventType::RoomCanonicalAlias, ""),
    (StateEventType::RoomPowerLevels, ""),
    (StateEventType::CallMember, "*"),
    (StateEventType::RoomJoinRules, ""),
    // Those two events are required to properly compute room previews.
    (StateEventType::RoomCreate, ""),
    (StateEventType::RoomHistoryVisibility, ""),
    // Required to correctly calculate the room display name.
    (StateEventType::MemberHints, ""),
];

async fn get_mandatory_rooms_for_initial_sync<'a>(room_mappings: &'a Option<Result<DirectMappingsEventContent, serde_json::Error>>, directs: &'a Option<Result<DirectEventContent, serde_json::Error>>) -> Result<Vec<&'a RoomId>, Error> {

    let mut out : HashSet<&RoomId> = HashSet::new();

    if let Some(room_mappings) = room_mappings {
        match room_mappings {
            Ok(room_mappings) => {
                for current in room_mappings.get_room_ids().drain(..) {
                    out.insert(current);
                }
            }
            Err(e) => {
                log::error!("Malformed room mappings received, ignoring. {}", e);
            }
        }
    }

    if let Some(directs) = directs {
        match directs {
            Ok(directs) => {
                let mut directs: Vec<&RoomId> = directs.values().flatten().map(|room_id| room_id.as_ref()).collect();
                for current in directs.drain(..) {
                    out.insert(current);
                }
            }
            Err(e) => {
                log::error!("Malformed directs received, ignoring. {}", e);
            }
        }
    }

    Ok(Vec::from_iter(out.drain()))
}

fn create_room_list() -> SlidingSyncListBuilder {
    SlidingSyncListBuilder::new("all_rooms")
        .timeline_limit(1)
        .required_state(REQUIRED_STATE.iter().map(|(key, val)| (key.to_owned(), val.to_string())).collect())
        .sync_mode(SlidingSyncMode::new_growing(20))
        .filters(Some(assign!(ListFilters::default(), {
            is_invite: None,
            not_room_types: vec![RoomTypeFilter::Space],
        })))

}

pub async fn build_sliding_sync(matrix_client: &Client) -> Result<SlidingSync, anyhow::Error> {

    let sliding_sync_builder = matrix_client.sliding_sync("everything_list")?
        .add_list(
            create_room_list()
        )
        .with_to_device_extension(
            assign!(ToDevice::default(), { enabled: Some(true)}),
        )
        .with_e2ee_extension(assign!(E2EE::default(), { enabled: Some(true)}))
        .with_account_data_extension(assign!(AccountData::default(), { enabled: Some(true)}))
        .with_typing_extension(assign!(Typing::default(), { enabled: Some(true)}))
        .share_pos();

    Ok(sliding_sync_builder.build().await?)
}

pub fn sync(client_data: ClientData, kill_signal: Receiver<()>){

    let client = client_data.get_matrix_client();
    let sliding_sync = client_data.get_sliding_sync();
    let updates_recv = client.subscribe_to_all_room_updates();

    spawn_sync_task(client_data, sliding_sync, updates_recv, kill_signal, true);

}

async fn setup_sliding_sync_room_subscriptions(sliding_sync: &SlidingSync, client: &Client) -> Result<(), anyhow::Error> {

    let maybe_direct_mappings = client.account()
        .fetch_account_data(GlobalAccountDataEventType::from("org.tachyon.direct_mappings")).await?
        .map(|raw| raw.deserialize_as::<DirectMappingsEventContent>());

    let maybe_direct_event = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await?.map(|raw| raw.deserialize_as::<DirectEventContent>());

    let rooms_to_watch = get_mandatory_rooms_for_initial_sync(&maybe_direct_mappings, &maybe_direct_event).await?;

    let subscription = assign!(RoomSubscription::default(), {
        required_state: DM_REQUIRED_STATE.iter().map(|(key, val)| (key.to_owned(), val.to_string())).collect(),
        timeline_limit: UInt::new_wrapping(10),
        include_heroes: Some(true),
            });

    let room_refs: Vec<&RoomId> = rooms_to_watch.iter().map(|r| r.as_ref()).collect();
    sliding_sync.subscribe_to_rooms(&room_refs, Some(subscription), false);

    Ok(())

}

fn spawn_sync_task(client_data: ClientData, sliding_sync: SlidingSync, mut updates_recv: Receiver<RoomUpdates>, mut kill_signal: Receiver<()>, mut first_sync_of_session: bool) {
    tokio::spawn(async move {
        info!("Initializing Sliding Sync...");
        let matrix_client = client_data.get_matrix_client();

        matrix_client.add_event_handler_context(TachyonContext{ client_data: client_data.clone() });
        
            matrix_client.add_event_handler(|event: DirectMappingsEvent, context: Ctx<TachyonContext> | async move {
                let direct_service = context.client_data.get_direct_service();
                direct_service.handle_direct_mappings_update(event.content).await.unwrap();
            });

            matrix_client.add_event_handler(|event: DirectEvent, context: Ctx<TachyonContext>, client: Client | async move {
                let direct_service = context.client_data.get_direct_service();
                let direct_diffs = direct_service.handle_directs_update(event.content).await.unwrap();

                // TODO: ask wassup with this ?
                // for direct_diff in direct_diffs {
                //     if let DirectDiff::RoomAdded(user_id, room_id) = direct_diff {
                //         if let Some(room) = client.get_room(&room_id) {
                //             //Remark room as direct room (not supported for now i believe by the sdk
                //         }
                //     }
                // }


            });
        
        info!("Fetching room subscriptions...");
        if let Err(err) = setup_sliding_sync_room_subscriptions(&sliding_sync, &matrix_client).await {
            error!("Error setting up sliding sync room subscriptions: {:?}", err);
            return;
        }

        let mut initial_sync = true;

        info!("Starting Sliding Sync...");
        let mut sync_stream = Box::pin(sliding_sync.sync());
        loop {
            tokio::select! {
                _ = kill_signal.recv() => {
                    info!("Gracefully exit sync loop...");
                    if let Err(err) = sliding_sync.stop_sync() {
                        error!("Error stopping sync loop: {:?}", err);
                    }
                    break;
                }
                sync_response = sync_stream.next() => {
                    match sync_response {
                        Some(Ok(update_summary)) => {
                            if initial_sync {
                                // Tell the FindContacts & FindMemberships to require full sync
                            }

                            info!("Received Sliding Sync stream response with pos: {:?}", &update_summary);

                            match updates_recv.recv().await {
                                Ok(room_updates) => {
                                    if(first_sync_of_session) {
                                        first_sync_of_session = false;
                                        handle_first_sync(&client_data).await.unwrap();
                                    }
                                    
                                    handle_room_updates(&client_data, room_updates).await;
                                }
                                Err(err) => {
                                    error!("Error receiving RoomUpdates: {:?}", err);
                                }
                            }

                            initial_sync = false
                        }
                        Some(Err(err)) => {
                            if err.client_api_error_kind() == Some(&ErrorKind::UnknownPos) {
                                info!("Unknown pos, re-syncing...");
                                spawn_sync_task(client_data, sliding_sync.clone(), updates_recv, kill_signal, first_sync_of_session);
                                break;
                            } else {
                                error!("Error in sync stream: {:?}", err);
                            }
                        }
                        _ => {
                            error!("Unexpected sync stream response");
                        }
                    }
                }

            }


        }

    });
}

async fn handle_room_updates(client_data: &ClientData, room_updates: RoomUpdates) {

    let own_user_id = client_data.get_matrix_client().user_id().unwrap().to_owned();

    let diffs = direct_handler::handle_direct_mappings_room_updates(room_updates.clone(), client_data.clone()).await.unwrap();

    //TODO: This is not enough, custom behaviour needs to happen in the ContactService to remove old mapping before adding new ones.
    let events_to_reevaluate = load_mapping_diff_events(client_data, diffs, &own_user_id).await.unwrap();
    debug!("events_to_reevaluate: {:?}", events_to_reevaluate);
    handle_contacts_room_updates(room_updates, client_data.clone(), events_to_reevaluate).await;

    let contact_service = client_data.get_contact_service();

    let contact_len = {
        contact_service.inner.pending_contacts.lock().unwrap().len()
    };

    let member_len = {
        contact_service.inner.pending_members.lock().unwrap().len()
    };

    let circle_len = {
        contact_service.inner.pending_circles.lock().unwrap().len()
    };

    if contact_len > 0 || member_len > 0 || circle_len > 0{
        let user = client_data.get_user_clone().unwrap();
        let _ = client_data.get_notification_handle().send(NotificationServerCommand::NOT(NotServer {
            payload: NotificationFactory::get_abch_updated(&user.uuid, user.get_email_address()),
        })).await;
    }


    }

#[derive(Debug)]
pub struct MappingDiffEvents {
    pub room_id: OwnedRoomId,
    pub events: Vec<RawAnySyncOrStrippedState>
}

impl MappingDiffEvents {
    fn new(room_id: OwnedRoomId) -> Self {
        Self {
            room_id,
            events: Vec::new()
        }
    }
}

async fn load_mapping_diff_events(client_data: &ClientData, diffs: Vec<MappingDiff>, me: &UserId) -> Result<Vec<MappingDiffEvents>, anyhow::Error> {
    let mut events_to_handle: Vec<MappingDiffEvents> = Vec::new();

    for diff in diffs.into_iter() {
        let user_id = diff.user_id();
        let room_id = diff.room_id();

        let matrix_client = client_data.get_matrix_client();
        let room = matrix_client.get_room(&room_id).unwrap();

        let mut mapping_diff_events = MappingDiffEvents::new(room_id.to_owned());
        if let Some(event) = room.get_state_event(StateEventType::RoomMember, &user_id.as_str()).await? {
            mapping_diff_events.events.push(event);
        }

        if let Some(event) = room.get_state_event(StateEventType::RoomMember, &me.as_str()).await? {
            mapping_diff_events.events.push(event);
        }

        events_to_handle.push(mapping_diff_events);
    }

    Ok(events_to_handle)
}

async fn handle_first_sync(client_data: &ClientData) -> Result<(), anyhow::Error> {

    let me = client_data.get_user_clone()?;
    let ticket_token = client_data.get_ticket_token();
    let notification_handle = client_data.get_notification_handle();

    let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
        sender: "Hotmail".to_string(),
        display_name: "Hotmail".to_string(),
        payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(&me.uuid.get_puid(), me.get_email_address(), &ticket_token))
    });

    notification_handle.send(initial_profile_msg).await?;


    //Todo fetch endpoint data
    let endpoint_data = b"<Data></Data>";
    notification_handle.send(NotificationServerCommand::RAW(RawCommand::with_payload(&format!("UBX 1:{}", &me.get_email_address().as_str()), endpoint_data.to_vec()))).await?;

    Ok(())
}



