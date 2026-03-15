use crate::matrix::handlers::context::TachyonContext;
use crate::matrix::handlers::{self, register_event_handlers};
use crate::tachyon::tachyon_client::TachyonClient;
use futures::StreamExt;
use log::{debug, error, info};
use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::api::client::error::ErrorKind;
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::{
    AccountData, ListFilters, RoomSubscription, ToDevice, Typing, E2EE,
};
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::direct::{DirectEvent, DirectEventContent};
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::events::{GlobalAccountDataEventType, StateEventType};
use matrix_sdk::ruma::{assign, OwnedRoomId, RoomId, UInt, UserId};
use matrix_sdk::sync::RoomUpdates;
use matrix_sdk::{
    Client, Error, Room, SlidingSync, SlidingSyncList, SlidingSyncListBuilder, SlidingSyncMode,
};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};

use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::error::SendTimeoutError;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::NotServer;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::presence_status::PresenceStatus;

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

fn create_room_list() -> SlidingSyncListBuilder {
    SlidingSyncList::builder("all_rooms")
        .required_state(REQUIRED_STATE.iter().map(|(key, val)| (key.to_owned(), val.to_string())).collect())
        .sync_mode(SlidingSyncMode::new_growing(20))
}

pub async fn build_sliding_sync(matrix_client: &Client) -> Result<SlidingSync, anyhow::Error> {
    let sliding_sync_builder = matrix_client
        .sliding_sync("everything_list")?
        .add_list(create_room_list())
        .with_all_extensions()
        .share_pos();

    Ok(sliding_sync_builder.build().await?)
}

pub fn sync(client_data: TachyonClient, kill_signal: Receiver<()>) {
    let client = client_data.matrix_client();
    let sliding_sync = client_data.sliding_sync();
    let updates_recv = client.subscribe_to_all_room_updates();

    spawn_sync_task(client_data, sliding_sync, updates_recv, kill_signal, true);
}

fn spawn_sync_task(
    client_data: TachyonClient,
    sliding_sync: SlidingSync,
    mut updates_recv: Receiver<RoomUpdates>,
    mut kill_signal: Receiver<()>,
    mut first_sync_of_session: bool,
) {
    tokio::spawn(async move {
        info!("Initializing Sliding Sync...");
        let matrix_client = client_data.matrix_client();
        register_event_handlers(&matrix_client, client_data.clone());

        let mut initial_sync = true;

        info!("Starting Sliding Sync...");
        let mut sync_stream = Box::pin(sliding_sync.sync());
        'main: loop {
            tokio::select! {
                _ = kill_signal.recv() => {
                    info!("Gracefully exit sync loop...");
                    if let Err(err) = sliding_sync.stop_sync() {
                        error!("Error stopping sync loop: {:?}", err);
                    }
                    break 'main;
                }
                sync_response = sync_stream.next() => {
                    match sync_response {
                        Some(Ok(update_summary)) => {
                            info!("Received Sliding Sync stream response with pos: {:?}", &update_summary);

                            match updates_recv.recv().await {
                                Ok(room_updates) => {
                                    println!("{:?}", &room_updates);
                                    if first_sync_of_session  {
                                        first_sync_of_session = false;
                                        handle_first_sync(&client_data).await.unwrap();
                                    }
                                    
                                }
                                Err(err) => {
                                    error!("Error receiving RoomUpdates: {:?}", err);
                                }
                            }

                            handle_addressbook_notifications(&client_data).await.unwrap();
                            initial_sync = false
                        }
                        Some(Err(err)) => {
                            if err.client_api_error_kind() == Some(&ErrorKind::UnknownPos) {
                                info!("Unknown pos, re-syncing...");
                                spawn_sync_task(client_data, sliding_sync.clone(), updates_recv, kill_signal, first_sync_of_session);
                                break 'main;
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

async fn handle_addressbook_notifications(client_data: &TachyonClient) -> Result<(), SendTimeoutError<NotificationServerCommand>> {

    let update_required = {
        let contact_holder = client_data.soap_holder().contacts.lock().unwrap();
        let member_holder = client_data.soap_holder().memberships.lock().unwrap();
        contact_holder.len() > 0 || member_holder.len() > 0
    };

    if update_required {
        let user = client_data.own_user().unwrap();
        client_data.notification_handle().send(NotificationServerCommand::NOT(NotServer {
            payload: NotificationFactory::get_abch_updated(&user.uuid, user.get_email_address()),
        })).await
    } else {
        Ok(())
    }
}

async fn handle_first_sync(client_data: &TachyonClient) -> Result<(), anyhow::Error> {
    let me = client_data.own_user()?;
    let ticket_token = client_data.ticket_token();
    let notification_handle = client_data.notification_handle();

    // This is sent to make the client pass the logon screen. Timeout of the logon screen is 1 minute.
    let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
        sender: "Hotmail".to_string(),
        display_name: DisplayName::new_from_ref("Hotmail"),
        payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(
            &me.uuid.get_puid(),
            me.get_email_address(),
            &ticket_token,
        )),
    });

    notification_handle.send(initial_profile_msg).await?;

    //Todo fetch endpoint data
    let endpoint_data = b"<Data></Data>";
    notification_handle
        .send(NotificationServerCommand::RAW(RawCommand::with_payload(
            &format!("UBX 1:{}", &me.get_email_address().as_str()),
            endpoint_data.to_vec(),
        )))
        .await?;
    Ok(())
}
