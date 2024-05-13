use log::debug;
use matrix_sdk::{Client, LoopCtrl, Room};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::sync::SyncResponse;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

use msnp::msnp::notification::command::command::NotificationServerCommand;

use crate::matrix::memberships::handle_memberships;
use crate::matrix::oim::handle_oims;
use crate::notification::client_store::ClientData;

#[derive(Clone)]
struct TachyonContext {
    notif_sender: Sender<NotificationServerCommand>,
    client_data: ClientData
}

pub async fn start_sync_task(client: Client, notif_sender: Sender<NotificationServerCommand>, mut client_data: ClientData, mut kill_signal: broadcast::Receiver<()>) {


    let sync_token = client.sync_token().await;

    let mut settings = SyncSettings::new().set_presence(PresenceState::Offline);

    if let Some(sync_token) = sync_token.as_ref() {
        settings = settings.token(sync_token);
    }


    client.add_event_handler_context(TachyonContext { notif_sender: notif_sender.clone(), client_data: client_data.clone() });

    //TODO handle contact list & address book -> Keep syncing
    let mut is_first_iteration = true;

    loop {
        tokio::select! {
            response = client.sync_once(settings.clone()) => {
            let response = response.unwrap();
                settings = settings.token(&response.next_batch);

        if is_first_iteration {

            is_first_iteration = false;
            let client_cloned = client.clone();
            let response_cloned = response.clone();
            let client_data_cloned = client_data.clone();
            let notif_sender_cloned = notif_sender.clone();
            let sync_token_clone = sync_token.clone();

            tokio::spawn(async move{
                handle_oims(client_cloned, response_cloned, client_data_cloned, notif_sender_cloned, sync_token_clone).await.unwrap();
            });

        }

        let client_cloned = client.clone();
        let response_cloned = response.clone();
        let client_data_cloned = client_data.clone();
        let notif_sender_cloned = notif_sender.clone();

        tokio::spawn(async move {
            handle_memberships(client_cloned, response_cloned, client_data_cloned, notif_sender_cloned).await.unwrap();
        });

            },
            kill_signal = kill_signal.recv() => {
                debug!("Matrix loop stopped gracefully");
                break;
            }

        }
    }
}