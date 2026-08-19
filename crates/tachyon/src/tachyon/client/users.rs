use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use anyhow::anyhow;
use msnp::shared::models::msn_user::MsnUser;
use ruma::{RoomId, UserId};

impl TachyonClient {

    pub async fn get_msn_user(&self, room_id: &RoomId, user_id: &UserId) -> Result<MsnUser, anyhow::Error> {

        let client = self.matrix_client();

        let room = client.get_room(room_id).ok_or(anyhow!("Could not find room : {}", &room_id))?;
        let room_member = room.get_member(user_id).await?.ok_or(anyhow!("Could find user {} in room {}", &user_id, &room_id))?;

        let mut user = MsnUser::from_user_id(user_id);
        user.display_name = room_member.display_name().map(|d| d.to_string());
        user.display_picture = self.get_user_avatar_as_msn_object(user_id, room_id).await?;
        user.status = self.get_presence(user_id).await;

        Ok(user)
    }

}