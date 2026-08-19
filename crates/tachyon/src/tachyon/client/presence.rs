use log::error;
use matrix_sdk::StoreError;
use ruma::events::presence::PresenceEvent;
use ruma::presence::PresenceState;
use ruma::serde::Raw;
use ruma::UserId;
use msnp::shared::models::presence_status::PresenceStatus;
use crate::tachyon::client::tachyon_client::TachyonClient;

const DEFAULT_PRESENCE: PresenceStatus = PresenceStatus::HDN;

impl TachyonClient {



    pub async fn get_presence(&self, user_id: &UserId) -> PresenceStatus {

       match self.matrix_client().state_store().get_presence_event(user_id).await {
           Ok(Some(presence)) => {
                match presence.deserialize() {
                    Ok(presence) => map_presence_to_msn(&presence.content.presence),
                    Err(err) => {
                        error!("Could not deserialize presence event for user: {} - {}", user_id, err);
                        DEFAULT_PRESENCE
                    }
                }
           }
           Ok(None) => DEFAULT_PRESENCE,
           Err(e) => {
               error!("Could not fetch presence event from state store for user : {} = {}", user_id, e);
               DEFAULT_PRESENCE
           }
       }


    }

}

fn map_presence_to_msn(presence_state: &PresenceState) -> PresenceStatus {
    match presence_state {
        PresenceState::Offline => {
            DEFAULT_PRESENCE
        }
        PresenceState::Online => {
            PresenceStatus::NLN
        }
        PresenceState::Unavailable => {
            PresenceStatus::BSY
        }
        PresenceState::_Custom(_) => {
            DEFAULT_PRESENCE
        }
        _ => {
            DEFAULT_PRESENCE
        }
    }
}