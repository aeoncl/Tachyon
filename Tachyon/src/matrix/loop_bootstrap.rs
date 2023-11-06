use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use log::kv::Source;
use matrix_sdk::{Client, Room, ruma};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::api::client::filter::{FilterDefinition, RoomFilter};
use matrix_sdk::ruma::api::client::sync::sync_events::v3::Filter;
use matrix_sdk::ruma::events::direct::DirectEvent;
use matrix_sdk::ruma::events::{GlobalAccountDataEvent, GlobalAccountDataEventType};
use matrix_sdk::ruma::events::presence::PresenceEvent;
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent;
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use matrix_sdk::ruma::OwnedMxcUri;
use matrix_sdk::ruma::presence::PresenceState;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use crate::AB_LOCATOR;
use crate::matrix::{presence_event_handler, stripped_room_member_event_handler, sync_room_message_event_handler};
use crate::matrix::sync_room_member_event_handler::handle_sync_room_member_event;
use crate::matrix::sync_typing_event_handler::handle_sync_typing_event;
use crate::models::msn_user::MSNUser;
use crate::models::notification::events::notification_event::{NotificationEvent, NotificationEventFactory};
use crate::models::notification::msn_client::MSNClient;
use crate::models::tachyon_error::TachyonError;
use crate::repositories::ab_locator::ABLocator;
use crate::repositories::msn_user_repository::MSNUserRepository;

#[derive(Debug, Clone)]
struct WLMatrixContext {
    msn_client: MSNClient,
    dedup: DedupGrimoire
}

#[derive(Debug, Clone)]
pub struct DedupGrimoire {
    display_name : Arc<Mutex<String>>,
    display_pic_mxid : Arc<Mutex<String>>,
}

impl DedupGrimoire {
    pub fn new() -> Self {
        DedupGrimoire{
            display_name: Arc::new(Mutex::new(String::new())),
            display_pic_mxid: Arc::new(Mutex::new(String::new()))
        }
    }

    pub fn get_display_picture(&self) -> String {
        self.display_pic_mxid.lock().expect("dedup presence token not to be poisoned").clone()
    }

    pub fn set_display_picture(&mut self, dedup_token: String) {
        let mut lock = self.display_pic_mxid.lock().expect("dedup presence token not to be poisoned");
        *lock = dedup_token;
    }

    pub fn get_display_name(&self) -> String {
        self.display_name.lock().expect("dedup presence token not to be poisoned").clone()
    }

    pub fn set_display_name(&mut self, dedup_token: String) {
        let mut lock = self.display_name.lock().expect("dedup presence token not to be poisoned");
        *lock = dedup_token;
    }

}

fn get_sync_settings() -> SyncSettings {
    let mut filters = FilterDefinition::default();
    let mut room_filters = RoomFilter::default();
    room_filters.include_leave = true;
    filters.room = room_filters;
    return SyncSettings::new().timeout(Duration::from_secs(5)).filter(Filter::FilterDefinition(filters)).set_presence(PresenceState::Online);
}

fn register_events(matrix_client: &Client, msn_client: MSNClient) {
    matrix_client.add_event_handler_context(WLMatrixContext { msn_client, dedup: DedupGrimoire::new() });

    matrix_client.add_event_handler({
        |ev: PresenceEvent, client: Client, context: Ctx<WLMatrixContext>| async move {
            let user_repo = MSNUserRepository::new(client.clone());
            presence_event_handler::handle_presence_event(ev, client, context.msn_client.clone(), user_repo).await;
        }
    });

    matrix_client.add_event_handler({
        |ev: StrippedRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {

            let user_repo = MSNUserRepository::new(client.clone());
            let ab_sender = AB_LOCATOR.get_sender();
            let notify_ab = stripped_room_member_event_handler::handle_stripped_room_member_event(ev, room, client, context.msn_client.clone(), user_repo, ab_sender).await;
            if notify_ab {
                context.msn_client.on_notify_ab_update();
            }
        }
    });


    matrix_client.add_event_handler({
        |ev: SyncRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            let ab_sender = AB_LOCATOR.get_sender();
            let user_repo = MSNUserRepository::new(client.clone());

            handle_sync_room_member_event(ev, room, client, context.msn_client.clone(), ab_sender, user_repo, context.dedup.clone()).await;
        }
    });

    matrix_client.add_event_handler({
        |ev: SyncTypingEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            let user_repo = MSNUserRepository::new(client.clone());
            handle_sync_typing_event(ev, room, context.msn_client.clone(), user_repo).await;
        }
    });

    matrix_client.add_event_handler({
        |ev: SyncRoomMessageEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            sync_room_message_event_handler::handle_sync_room_message_event(ev, room, client, context.msn_client.clone()).await;
        }
    });


}


pub fn listen(matrix_client: Client, msn_client: MSNClient) -> oneshot::Sender<()> {
    register_events(&matrix_client, msn_client);
    let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();

    let _result = tokio::spawn(async move {
        let mut settings = get_sync_settings();
        let mut retry_count = 0;
        let max_retry_count = 3;

        let sync_token = matrix_client.sync_token().await;

        log::info!("WLMatrix Sync - Preparing Initial Sync");
        if let Some(token) = sync_token {
            log::info!("WLMatrix Sync - Token loaded: {}", &token);
            settings = settings.token(token);
        }


        //   matrix_client.sync_stream(sync_settings);

        loop {
            tokio::select! {
                sync_result = matrix_client.sync_once(settings.clone()) => {
                    if let Ok(sync_result) = sync_result {
                        log::info!("WLMatrix Sync - next batch: {}", &sync_result.next_batch);

                        settings = settings.token(sync_result.next_batch);
                        retry_count = 0;
                    } else {
                        if retry_count < max_retry_count {
                            retry_count += 1;
                        } else {
                            break;
                                //TODO when we break out of the sync loop (because an error) we should tell the client & all it's switchboards to disconnect
                        }
                    }
                },
                _stop_signal = &mut stop_receiver => {
                    break;
                },
            }
        }
    });
    return stop_sender;
}