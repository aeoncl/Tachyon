use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::events::AnySyncTimelineEvent::MessageLike;
use matrix_sdk::ruma::events::{AnySyncTimelineEvent, MessageLikeEventType, OriginalSyncMessageLikeEvent};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::presence::PresenceState;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use crate::notification::client_store::ClientStoreFacade;



pub async fn start_sync_task(client: Client, not_sender: Sender<NotificationServerCommand>, client_store: ClientStoreFacade, kill_signal: broadcast::Receiver<()>) {

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

    for (key, room) in response.rooms.join {

       let event =  room.timeline.events.get(0).unwrap();

       let deser = event.event.deserialize_as::<OriginalSyncMessageLikeEvent<RoomMessageEventContent>>();
        if let Ok(event) = deser {

        }



    }






}