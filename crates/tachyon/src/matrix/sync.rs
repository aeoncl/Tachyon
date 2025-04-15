use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::future::Future;
use std::mem;
use std::time::Duration;

use anyhow::{anyhow, Error};
use futures_core::Stream;
use lazy_static_include::syn::__private::quote::__private::ext::RepToTokensExt;
use log::{debug, info};
use matrix_sdk::{Client, LoopCtrl, Room};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::deserialized_responses::{TimelineEvent, TimelineEventKind};
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::{OwnedMxcUri, OwnedRoomId, OwnedUserId};
use matrix_sdk::ruma::api::client::filter::FilterDefinition;
use matrix_sdk::ruma::api::client::sync::sync_events::v3::Filter;
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, AnyMessageLikeEvent};
use matrix_sdk::ruma::events::direct::DirectEvent;
use matrix_sdk::ruma::events::GlobalAccountDataEventType::IgnoredUserList;
use matrix_sdk::ruma::events::ignored_user_list::IgnoredUserListEvent;
use matrix_sdk::ruma::events::presence::PresenceEvent;
use matrix_sdk::ruma::events::room::member::{RoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::sync::{JoinedRoomUpdate, SyncResponse, Timeline};
use matrix_sdk_ui::sync_service::{SyncService, SyncServiceBuilder};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::NotServer;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::presence_status::PresenceStatus;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{BaseMember, ContactType, ContactTypeEnum, MemberState};

use crate::matrix::memberships::{handle_joined_room_member_event, handle_memberships};
use crate::matrix::msn_user_resolver::{avatar_to_msn_obj, get_avatar_bytes, resolve_msn_user_from_presence_event};
use crate::matrix::oim::handle_oims;
use crate::notification::client_store::ClientData;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::PresenceStateCompatible;

#[derive(Clone)]
struct TachyonContext {
    notif_sender: Sender<NotificationServerCommand>,
    client_data: ClientData
}


/*pub async fn sync(client_data: &ClientData) -> Result<(), anyhow::Error> {

    let matrix_client = client_data.get_matrix_client();
    let initial_sync = matrix_client.sync_once(SyncSettings::default()).await?;




    let mut sync_stream = Box::pin(matrix_client.sync_stream(SyncSettings::default().token(initial_sync.next_batch)).await);

    while let Some(Ok(mut sync_response)) = sync_stream.next().await {

        while let Some((room_id, roomUpdate)) = sync_response.rooms.join.pop_first() {
            let room = matrix_client.get_room(&room_id).expect("Room to be present when receiving an update for it.");
            handle_joined_room_update(room_id, roomUpdate, room, sync_response.next_batch.clone(), false).await.unwrap();
        }
    }

    ()
}



pub async fn handle_joined_room_update(room_id: OwnedRoomId, update: JoinedRoomUpdate, room: Room, next_batch: String, initial: bool) -> Result<(), anyhow::Error> {


    let mut next_start_token = update.timeline.prev_batch.clone();

    while next_start_token.is_some() && next_start_token.unwrap() != next_batch {
        //Fetch missed messages

        let message_options = MessagesOptions::forward().from(next_start_token.as_ref().unwrap()).to(&next_batch);
        let messages_response = room.messages(message_options).await.unwrap();


            //TODO Do stuff with the messages
        next_start_token = messages_response.end
    }





    todo!()
}
*/

pub async fn initial_sync(tr_id: u128, client_data: &ClientData) -> Result<(Vec<IlnServer>, Vec<NotServer>), anyhow::Error> {

    let me_msn_user = client_data.get_user_clone()?;
    let client = client_data.get_matrix_client();



    let mut sync_token = client.sync_token().await;
    
    let mut settings = SyncSettings::new().set_presence(PresenceState::Offline).timeout(Duration::from_secs(240));
    if let Some(sync_token) = sync_token.as_ref() {
        settings = settings.token(sync_token);
    } else {
        //If it's our first ever sync, lazy load members !
        settings = settings.filter(Filter::FilterDefinition(FilterDefinition::with_lazy_loading()));
    }

    let response = client.sync_once(settings.clone()).await?;

    let (mut contacts, mut memberships, mut circle_members) = handle_memberships(client, &response).await?;

    let mut notifications = Vec::new();

    if !contacts.is_empty() || !memberships.is_empty() {
        {
            let mut contacts_mtx = client_data.inner.soap_holder.contacts.lock().unwrap();
            let mut memberships_mtx = client_data.inner.soap_holder.memberships.lock().unwrap();
            contacts_mtx.append(&mut contacts);
            memberships_mtx.append(&mut memberships);

            notifications.push(NotServer {
                payload: NotificationFactory::get_abch_updated(&me_msn_user.uuid, me_msn_user.get_email_address().as_str()),
            });

        }
    }

    if !circle_members.is_empty() {
        for (circle_id, mut members) in circle_members.drain() {

            match client_data.inner.soap_holder.circle_contacts.get_mut(&circle_id) {
                None => {
                    client_data.inner.soap_holder.circle_contacts.insert(circle_id.clone(), members);
                }
                Some(mut circle_members) => {
                    circle_members.append(&mut members);
                }
            }

            notifications.push(NotServer {
                payload: NotificationFactory::get_circle_updated(&me_msn_user.uuid, me_msn_user.get_email_address().as_str(), &circle_id)
            });

        }
    }

    let iln = handle_initial_presence(tr_id, response.presence, client_data).await?;

    Ok((iln, notifications))
}


pub async fn handle_initial_presence(tr_id: u128, presence: Vec<Raw<PresenceEvent>>, client_data: &ClientData) -> Result<Vec<IlnServer>, anyhow::Error> {
    let mut out = Vec::with_capacity(presence.len());

    for current in presence {
        let presence_event = current.deserialize()?;
        let msn_user = resolve_msn_user_from_presence_event(presence_event, client_data).await;

        let target_user = msn_user.get_network_id_email();
        let display_name = msn_user.compute_display_name().to_string();
        let iln = IlnServer{
            tr_id,
            presence_status: msn_user.status,
            target_user,
            via: None,
            display_name,
            client_capabilities: msn_user.capabilities,
            avatar: msn_user.display_picture,
            badge_url: None,
        };

        out.push(iln);
    }
   Ok(out)
}

pub async fn start_sync_task(client: Client, notif_sender: Sender<NotificationServerCommand>, mut client_data: ClientData, mut kill_signal: broadcast::Receiver<()>) {

    let me_msn_user = client_data.get_user_clone().expect("to be here");

    let mut sync_token = client.sync_token().await;

    let filter = FilterDefinition::with_lazy_loading();

    let mut settings = SyncSettings::new().set_presence(PresenceState::Offline).timeout(Duration::from_secs(240)).filter(Filter::FilterDefinition(filter));

    if let Some(sync_token) = sync_token.as_ref() {
        settings = settings.token(sync_token);
    }



    client.add_event_handler_context(TachyonContext { notif_sender: notif_sender.clone(), client_data: client_data.clone() });


    client.add_event_handler({ |event: SyncRoomMemberEvent, room: Room, client: Client, context: Ctx<TachyonContext>| async move {
        let client_data = &context.client_data;

        let mut contacts = Vec::new();
        let mut memberships = VecDeque::new();
        let mut circle_members: HashMap<String, Vec<ContactType>> = HashMap::new();

        let me = client.user_id().expect("to be here");

        handle_joined_room_member_event(&event, &room, me, &client, &mut contacts, &mut memberships, &mut circle_members).await;

        if !contacts.is_empty() || !memberships.is_empty() {
            {
                let mut contacts_mtx = client_data.inner.soap_holder.contacts.lock().unwrap();
                let mut memberships_mtx = client_data.inner.soap_holder.memberships.lock().unwrap();
                contacts_mtx.append(&mut contacts);
                memberships_mtx.append(&mut memberships);
            }
        }

        if !circle_members.is_empty() {
            for (circle_id, mut members) in circle_members.drain() {
                match client_data.inner.soap_holder.circle_contacts.get_mut(&circle_id) {
                    None => {
                        client_data.inner.soap_holder.circle_contacts.insert(circle_id.clone(), members);
                    }
                    Some(mut circle_members) => {
                        circle_members.append(&mut members);
                    }
                }
            }
        }

    }});


    // client.add_event_handler({ |event: DirectEvent, client: Client, context: Ctx<TachyonContext>| async move{
    //
    //
    // }});

    let mut is_first_iteration = true;

    loop {
        tokio::select! {
            response = client.sync_once(settings.clone()) => {

                let response = response.unwrap();

                debug!("---New Sync---: from: {:?} to: {}", &sync_token, &response.next_batch);

                settings = settings.token(&response.next_batch);

        if is_first_iteration {

            is_first_iteration = false;
            let client_cloned = client.clone();
            let response_cloned = response.clone();
            let client_data_cloned = client_data.clone();
            let notif_sender_cloned = notif_sender.clone();
            let sync_token_clone = sync_token.clone();

            // tokio::spawn(async move{
            //     handle_oims(client_cloned, response_cloned, client_data_cloned, notif_sender_cloned, sync_token_clone).await
            // });

        }



        let client_cloned = client.clone();
        let response_cloned = response.clone();
        let client_data_cloned = client_data.clone();
        let notif_sender_cloned = notif_sender.clone();

      // handle_memberships(client_cloned, response_cloned, client_data_cloned, notif_sender_cloned).await;


            info!("Synced finished....");
            info!("Dispatching Notifications...");

                let send_ab_notify = {
                    let contacts_mtx = client_data.inner.soap_holder.contacts.lock().unwrap();
                    let memberships_mtx = client_data.inner.soap_holder.memberships.lock().unwrap();
                    !contacts_mtx.is_empty() || !memberships_mtx.is_empty()
                };

            if send_ab_notify {
                  //TODO make this less shit later
                notif_sender.send(NotificationServerCommand::NOT(NotServer {
                    payload: NotificationFactory::get_abch_updated(&me_msn_user.uuid, me_msn_user.get_email_address().as_str()),
                    })).await;
            }

            {
                for circle_id in client_data.inner.soap_holder.circle_contacts.iter().map(|m| m.key().clone()) {
                     notif_sender.send(NotificationServerCommand::NOT(NotServer {
                            payload: NotificationFactory::get_circle_updated(&me_msn_user.uuid, me_msn_user.get_email_address().as_str(), &circle_id)
                    })).await;
                }
            }


            sync_token = Some(response.next_batch.clone());

            },
            kill_signal = kill_signal.recv() => {
                debug!("Matrix loop stopped gracefully");
                break;
            }

        }
    }
}