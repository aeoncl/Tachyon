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
use msnp::shared::models::oim::{MetaData, MetadataMessage};
use msnp::shared::payload::raw_msg_payload::factories::MsgPayloadFactory;
use crate::notification::client_store::ClientStoreFacade;
use crate::shared::identifiers::MatrixIdCompatible;


pub async fn start_sync_task(client: Client, notif_sender: Sender<NotificationServerCommand>, client_store: ClientStoreFacade, kill_signal: broadcast::Receiver<()>) {

    let sync_token = client.sync_token().await;

    let mut settings = SyncSettings::new().set_presence(PresenceState::Offline);

    if let Some(sync_token) = sync_token.as_ref() {
        settings = settings.token(sync_token);
    }

    //handle OIMs (room messages) -> Unregister after sync loop;
    //Add oims to the client store
    //Send xml notification with too large
    //SOAP endpoints to retrieve XML & message contents via event id


    //handle contact list & address book -> Keep syncing

    let response = client.sync_once(settings).await.unwrap();




    for (key, room) in response.rooms.join {

        if let Some(prev_batch) = room.timeline.prev_batch {
            //We missed some events
            if let Some(room) = client.get_room(&key) {


                let mut config = MessagesOptions::forward();
                config.from = Some(prev_batch);
                config.to = sync_token.clone();
                config.limit = uint!(100);
                config.filter = RoomEventFilter::empty();

                while {


                    let messages = room.messages(config).await.unwrap();


                    config = MessagesOptions::forward();
                    config.from = messages.end;
                    config.to = sync_token.clone();
                    config.limit = uint!(100);
                    config.filter = RoomEventFilter::empty();


                    config.from != None
                } {};




            };
        };

        let mut client_data = client_store.get_client_data(&client.access_token().expect("token to be present")).unwrap();

        let mut oims = Vec::new();

        for event in room.timeline.events {

            if let Ok(AnySyncTimelineEvent::MessageLike(e)) = event.event.deserialize() {
                println!("{:?}", e);
                oims.push(e);
                let test = 0;
            }

        }


        client_data.add_oims(oims);

    }

    notif_sender.send(NotificationServerCommand::MSG(MsgServer {
        sender: "Hotmail".to_string(),
        display_name: "Hotmail".to_string(),
        payload: MsgPayload::Raw(MsgPayloadFactory::get_initial_mail_data_too_large_notification())
    })).await.unwrap();



}