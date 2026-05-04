use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use anyhow::Error;
use matrix_sdk::async_trait;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::{OwnedRoomId, UserId};
use msnp::shared::models::{email_address::EmailAddress, msn_user::MsnUser};

#[async_trait]
pub trait ToMsnUser {
    async fn to_msn_user(&self) -> Result<MsnUser, anyhow::Error>;
    async fn to_msn_user_lazy(&self)  -> Result<MsnUser, anyhow::Error>;
}

pub trait RoomMsnUserResolver {
    fn resolve_msn_user(user_id: &UserId) -> Result<MsnUser, anyhow::Error>;
}

pub trait ToRoomId {
    fn to_room_id(&self) -> Result<OwnedRoomId, anyhow::Error>;
}

pub trait ToEmailAddress {
    fn to_email_address(&self) -> Result<EmailAddress, anyhow::Error>;
}

#[async_trait]
impl ToMsnUser for RoomMember {
    async fn to_msn_user(&self) -> Result<MsnUser, Error> {
        let mut msn_user = MsnUser::from_user_id(self.user_id());
        msn_user.display_name = self.display_name().map(|name| name.to_string());

        //TODO Display Picture

        Ok(msn_user)
    }

    async fn to_msn_user_lazy(&self) -> Result<MsnUser, Error> {
        let mut msn_user = MsnUser::from_user_id(self.user_id());
        msn_user.display_name = self.display_name().map(|name| name.to_string());


        Ok(msn_user)
    }
}

impl ToEmailAddress for RoomMember {
    fn to_email_address(&self) -> Result<EmailAddress, Error> {
        Ok(EmailAddress::from_user_id(self.user_id()))
    }
}