use std::fmt::format;
use chrono::{DateTime, Local, TimeZone};
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::events::AnySyncTimelineEvent::MessageLike;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent, MessageLikeEventType, OriginalSyncMessageLikeEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::MessageLikeEvent::Original;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::presence::PresenceState;
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

    if let Some(sync_token) = sync_token {
        settings = settings.token(sync_token);
    }

    //handle OIMs (room messages) -> Unregister after sync loop;
    //Add oims to the client store
    //Send xml notification with too large
    //SOAP endpoints to retrieve XML & message contents via event id


    //handle contact list & address book -> Keep syncing

    let response = client.sync_once(settings).await.unwrap();

    for (key, noti) in response.notifications {

    }


    let mut md = MetaData {
        ..Default::default()
    };

    for (key, room) in response.rooms.join {


        for event in room.timeline.events {

            if let Ok(AnySyncTimelineEvent::MessageLike(e)) = event.event.deserialize() {
                match &e {
                    AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(original_event)) => {
                        if let MessageType::Text(text) = &original_event.content.msgtype {

                            println!("DEBUG: {}", text.body);
                            let timestamp = DateTime::from_timestamp_millis(original_event.origin_server_ts.0.into()).unwrap().naive_local();
                            let message = MetadataMessage::new(Local.from_local_datetime(&timestamp).unwrap(), EmailAddress::from_user_id(&original_event.sender), "blabla".into(), format!("{};{}", &key, &original_event.event_id), 0);
                            md.messages.push(message);
                        }
                    }
                    _ => {

                    }
                }
                println!("{:?}", e);
                let test = 0;
            }

        }





    }

    notif_sender.send(NotificationServerCommand::MSG(MsgServer {
        sender: "Hotmail".to_string(),
        display_name: "Hotmail".to_string(),
        payload: MsgPayload::Raw(MsgPayloadFactory::get_initial_mail_data_notification(md))
    })).await.unwrap();



}