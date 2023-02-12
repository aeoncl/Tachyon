use std::str::FromStr;

use matrix_sdk::{ruma::{OwnedUserId, UserId, RoomId}, Client, StoreError};

use crate::{models::msn_user::MSNUser, utils::identifiers::matrix_id_to_msn_addr, generated::payloads::PresenceStatus};



pub struct MSNUserRepository{
    matrix_client: Client
}



impl MSNUserRepository {
    pub fn new(matrix_client: Client) -> Self {
        return MSNUserRepository { matrix_client };
    }

    pub async fn get_msnuser(&self, room_id: &RoomId, user_id: &UserId) -> Result<MSNUser, StoreError> {
        let found = self.matrix_client.store().get_profile(&room_id, &user_id).await?;

        let found2 = self.matrix_client.store().get_presence_event(&user_id).await?;

        let mut out = MSNUser::from_matrix_id(user_id.to_string());

        if let Some(ev) = found {
            if let Some(original) = ev.as_original() {
                let content = &original.content;
                let msn_addr = matrix_id_to_msn_addr(&user_id.to_string());
                out.set_display_name(content.displayname.as_ref().unwrap_or(&msn_addr).to_owned());
                out.set_status(PresenceStatus::NLN);
                //TODO Avatar to MSNObj
            }
        }

        if let Some(presence_ev) = found2 {
            let presence_ev = presence_ev.deserialize()?;

            out.set_status(presence_ev.content.presence.into());
            out.set_psm(presence_ev.content.status_msg.unwrap_or(String::new()));
        }

    return Ok(out);
    }
}