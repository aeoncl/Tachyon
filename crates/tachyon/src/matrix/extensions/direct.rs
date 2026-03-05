use std::future::Future;
use anyhow::Error;
use matrix_sdk::Room;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::api::error::MatrixError;
use matrix_sdk::ruma::events::direct::OwnedDirectUserIdentifier;
use matrix_sdk::ruma::OwnedUserId;

pub trait DirectRoom {

    fn is_one_to_one_direct(&self) -> bool;

    fn get_single_direct_target(&self) -> Option<OwnedUserId>;
    async fn get_single_direct_target_member(&self) -> Result<Option<RoomMember>, MatrixError>;

    async fn get_single_direct_target_member_lazy(&self) -> Result<Option<RoomMember>, MatrixError>;


}

impl DirectRoom for Room {
    fn is_one_to_one_direct(&self) -> bool {
        let direct_targets = self.direct_targets();
        direct_targets.len() == 1
    }

    fn get_single_direct_target(&self) -> Option<OwnedUserId> {
        if self.is_one_to_one_direct() {
            if let Some(user_id) = self.direct_targets().iter().last() {
                user_id.clone().into_user_id()
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn get_single_direct_target_member(&self) -> Result<Option<RoomMember>, MatrixError> {
        if let Some(user_id) = self.get_single_direct_target() {
            if let Ok(Some(maybe_member)) = self.get_member(&user_id).await {
               return Ok(Some(maybe_member));
            }
        }

        Ok(None)
    }

    async fn get_single_direct_target_member_lazy(&self) -> Result<Option<RoomMember>, MatrixError> {
        if let Some(user_id) = self.get_single_direct_target() {
            if let Ok(Some(maybe_member)) = self.get_member_no_sync(&user_id).await {
                return Ok(Some(maybe_member));
            }
        }

        Ok(None)    
    }
}