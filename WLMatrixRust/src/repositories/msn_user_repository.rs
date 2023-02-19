use matrix_sdk::{ruma::{UserId, RoomId}, Client, StoreError};

use crate::{generated::payloads::PresenceStatus, models::msn_user::MSNUser};



pub struct MSNUserRepository{
    matrix_client: Client
}



impl MSNUserRepository {
    pub fn new(matrix_client: Client) -> Self {
        return MSNUserRepository { matrix_client };
    }
    

    

    pub async fn get_msnuser(&self, room_id: &RoomId, user_id: &UserId, fetch_presence: bool) -> Result<MSNUser, StoreError> {
        let found = self.matrix_client.store().get_profile(&room_id, &user_id).await?;


        let mut out = MSNUser::from_matrix_id(user_id.to_owned());

        if let Some(ev) = found {
            if let Some(original) = ev.as_original() {
                let content = &original.content;
                let msn_addr = out.get_msn_addr();
                out.set_display_name(content.displayname.as_ref().unwrap_or(&msn_addr).to_owned());
                out.set_status(PresenceStatus::NLN);
                //TODO Avatar to MSNObj
            }
        }

        if fetch_presence {
            let presence_event = self.matrix_client.store().get_presence_event(&user_id).await?;
            if let Some(presence_ev) = presence_event {
                let presence_ev = presence_ev.deserialize()?;
    
                out.set_status(presence_ev.content.presence.into());
                out.set_psm(presence_ev.content.status_msg.unwrap_or(String::new()));
            }
        }
    return Ok(out);
    }
}