use crate::matrix::handlers::context::TachyonContext;
use crate::matrix::handlers::{self, register_event_handlers};
use crate::tachyon::client::tachyon_client::TachyonClient;
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
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendTimeoutError;
use tokio::task::JoinHandle;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayloadType};
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
    (StateEventType::RoomTombstone, ""),
];

fn create_room_list() -> SlidingSyncListBuilder {
    SlidingSyncList::builder("all_sync")
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

pub async fn sync(tachyon_client: TachyonClient, matrix_client: Client, kill_signal_snd: Sender<()>, kill_signal_rcv: Receiver<()>) -> JoinHandle<()> {
    let sliding_sync = build_sliding_sync(&matrix_client).await.unwrap();
    let updates_recv = matrix_client.subscribe_to_all_room_updates();

    spawn_sync_task(tachyon_client, matrix_client, sliding_sync, updates_recv, kill_signal_snd, kill_signal_rcv)
}

fn spawn_sync_task(
    tachyon_client: TachyonClient,
    matrix_client: Client,
    sliding_sync: SlidingSync,
    mut updates_recv: Receiver<RoomUpdates>,
    client_shutdown_snd: Sender<()>,
    mut client_shutdown_rcv: Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!("Initializing Sliding Sync...");
        let (handler_drop_guards, context_drop_guard) = register_event_handlers(&matrix_client, tachyon_client.clone());

        info!("Starting Sliding Sync...");
        let (error_tx, mut error_rx) = mpsc::channel::<ErrorKind>(1);

        let mut sync_handle = tokio::spawn({
            let sliding_sync = sliding_sync.clone();

            async move {
                let mut sync_stream = Box::pin(sliding_sync.sync());
                loop {
                    match sync_stream.next().await {
                        Some(Ok(update_summary)) => {
                            info!("Received Sliding Sync stream response with pos: {:?}", &update_summary);
                        }
                        Some(Err(err)) => {
                            if let Some(error_kind) = err.client_api_error_kind() {
                                error!("Error in sync stream: {:?}", error_kind);
                                let _ = error_tx.send(error_kind.clone()).await;
                            } else {
                                error!("Error in sync stream: {:?}", err);
                            }
                            break;
                        }
                        None => {
                            error!("Sync stream ended unexpectedly");
                            break;
                        }
                    }
                }
            }
        });

        let mut restart_sync = false;
        loop {
            tokio::select! {
                _ = client_shutdown_rcv.recv() => {
                    info!("Gracefully exit sync loop...");
                    if let Err(err) = sliding_sync.stop_sync() {
                        error!("Error stopping sync loop: {:?}", err);
                    }
                    sync_handle.abort();
                    break;
                }
                room_update = updates_recv.recv() => {
                    match room_update {
                        Ok(room_updates) => {
                            println!("{:?}", &room_updates);

                            if let Err(e) = handle_addressbook_notifications(&tachyon_client).await {
                                error!("Error handling addressbook notifications: {:?}", e);
                            }
                        }
                        Err(err) => {
                            error!("Error receiving RoomUpdates: {:?}", err);
                        }
                    }
                }
                error_kind = error_rx.recv() => {
                    if let Some(ErrorKind::UnknownPos) = error_kind {
                        info!("Unknown pos detected, re-syncing...");
                        sync_handle.abort();
                        restart_sync = true;
                        break;
                    } else {
                        error!("Unrecoverable sync error: {:?}", error_kind);
                        let _ = client_shutdown_snd.send(());
                        sync_handle.abort();
                        break;
                    }
                }
                _ = &mut sync_handle => {
                    info!("Sync handle completed");
                    break;
                }
            }
        }

        if restart_sync {
            drop(handler_drop_guards);
            drop(context_drop_guard);
            spawn_sync_task(
                tachyon_client,
                matrix_client,
                sliding_sync.clone(),
                updates_recv,
                client_shutdown_snd,
                client_shutdown_rcv
            );
        }

        info!("Sync task finished");
    })
}

async fn handle_addressbook_notifications(client_data: &TachyonClient) -> Result<(), SendTimeoutError<NotificationServerCommand>> {

    let update_required = {
        let contact_holder = client_data.soap_holder().contacts.lock().unwrap();
        let member_holder = client_data.soap_holder().memberships.lock().unwrap();
        contact_holder.len() > 0 || member_holder.len() > 0
    };

    if update_required {
        let user = client_data.own_user();
        client_data.notification_handle().send(NotificationServerCommand::NOT(NotServer {
            payload: NotificationPayloadType::Normal(NotificationFactory::get_abch_updated(&user.uuid, user.get_email_address())),
        })).await
    } else {
        Ok(())
    }
}
