use std::fmt::format;
use chrono::{DateTime, Local, TimeZone};
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::api::client::filter::RoomEventFilter;
use matrix_sdk::ruma::api::Direction;
use matrix_sdk::ruma::events::AnySyncTimelineEvent::MessageLike;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent, AnyTimelineEvent, MessageLikeEvent, MessageLikeEventType, OriginalSyncMessageLikeEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::MessageLikeEvent::Original;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::ruma::{UInt, uint};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::oim::{MetaData, MetadataMessage, OIM};
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::raw_msg_payload::factories::MsgPayloadFactory;
use crate::matrix::oim::handle_oims;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::shared::identifiers::MatrixIdCompatible;

pub async fn start_sync_task(client: Client, notif_sender: Sender<NotificationServerCommand>, mut client_data: ClientData, kill_signal: broadcast::Receiver<()>) {

    let sync_token = client.sync_token().await;


    let mut settings = SyncSettings::new().set_presence(PresenceState::Offline);

    if let Some(sync_token) = sync_token.as_ref() {
        settings = settings.token(sync_token);
    }

    //TODO handle contact list & address book -> Keep syncing

    let response = client.sync_once(settings).await.unwrap();

    tokio::spawn(async move{
        handle_oims(client.clone(), response.clone(), client_data.clone(), notif_sender.clone(), sync_token).await.unwrap();
    });

}

