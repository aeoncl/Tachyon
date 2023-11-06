use log::{info, warn};
use matrix_sdk::Client;
use matrix_sdk::ruma::events::presence::PresenceEvent;
use matrix_sdk::ruma::OwnedUserId;
use crate::generated::payloads::PresenceStatus;
use crate::models::notification::msn_client::MSNClient;
use crate::models::owned_user_id_traits::ToMsnAddr;
use crate::repositories::msn_user_repository::MSNUserRepository;


pub(crate) async fn handle_presence_event(ev: PresenceEvent, client: Client, msn_client: MSNClient, user_repo:MSNUserRepository) {
    if ev.sender == client.user_id().unwrap() {
        info!("received presence event for me, notifying ab profile update...");
        return;
    }

    let event_sender: &OwnedUserId = &ev.sender;
    let sender_msn_addr = event_sender.to_msn_addr();

    if let Ok(mut user) = user_repo.get_msnuser_from_userid(event_sender, false).await {
        let presence_status: PresenceStatus = ev.content.presence.clone().into();

        info!("Received Presence Event: {:?} - ev: {:?}", &presence_status, &ev);

        if PresenceStatus::FLN == presence_status {
            msn_client.on_user_disconnected(user);
        } else {
            user.set_status(presence_status);
            if let Some(display_name) = ev.content.displayname {
                user.set_display_name(display_name);
            }

            if let Some(status_msg) = ev.content.status_msg {
                user.set_psm(status_msg);
            }


            if let Some(avatar_mxc) = ev.content.avatar_url.as_ref() {
                match user_repo.get_avatar(avatar_mxc.clone()).await {
                    Ok(avatar) => {
                        user.set_display_picture(Some(user_repo.avatar_to_msn_obj(&avatar, sender_msn_addr.clone(), &avatar_mxc)));
                    }
                    Err(err) => {
                        log::error!("Couldn't download avatar: {} - {}", &avatar_mxc, err);
                    }
                }
            }

            msn_client.on_user_presence_changed(user);

        }
    } else {
        warn!("Could not find user in repo (presence) {}", &event_sender);
    }
}