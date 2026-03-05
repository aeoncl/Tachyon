use anyhow::Error;
use base64::engine::general_purpose;
use base64::Engine;
use matrix_sdk::ruma::{RoomId, UserId};
use matrix_sdk::{Client, Room};
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
use msnp::shared::models::{email_address::EmailAddress, msn_user::MsnUser};
use std::str::FromStr;
use base32::Alphabet;
use matrix_sdk::media::MediaFormat;
use crate::matrix::extensions::direct::DirectRoom;

pub trait ToMsnUser {
    async fn to_msn_user(&self) -> Result<MsnUser, anyhow::Error>;
    async fn to_msn_user_lazy(&self)  -> Result<MsnUser, anyhow::Error>;
}

pub trait RoomMsnUserResolver {
    fn resolve_msn_user(user_id: &UserId) -> Result<MsnUser, anyhow::Error>;
}



impl ToMsnUser for Room {
    async fn to_msn_user(&self) -> Result<MsnUser, anyhow::Error> {
        to_msn_user_internal(self, false).await
    }

    async fn to_msn_user_lazy(&self) -> Result<MsnUser, Error> {
        to_msn_user_internal(self, true).await
    }

}

async fn to_msn_user_internal(room: &Room, lazy_resolve: bool) -> Result<MsnUser, Error> {
    let email = room.to_email_address()?;
    let mut user = MsnUser::with_email_addr(email);

    let maybe_direct_target = if room.is_valid_one_to_one_direct() {
         if lazy_resolve {
             room.get_single_direct_target_member_lazy().await
        } else {
             room.get_single_direct_target_member().await
         }
    } else {
        Ok(None)
    };
    

    if let Ok(Some(direct_target)) = &maybe_direct_target {
        user.display_name = direct_target.display_name().map(|name| name.to_string());
    } else {
        if let Ok(display_name) = room.display_name().await {
            user.display_name = Some(display_name.to_string());
        } else {
            user.display_name = Some(room.room_id().to_string());
        }
    };


    //Todo chek if direct_target for avatar.
    // Check if we call call endpoint with OPTION or HEAD request to fetch image size and avoid downloading the image at this time


    if let Some(avatar_info) = room.avatar_info() {
        //FIXME: Avatar MSNObject requires to compute the SHA1 of the bytes, let's see if we can avoid that.
        // What happens if no SHA1 is set in the MSNObject?

        let avatar_mxc = room.avatar_url().unwrap();
        let base64_mxc =  general_purpose::STANDARD.encode(avatar_mxc.to_string());


        let size = usize::try_from(avatar_info.size.unwrap()).unwrap();
        let display_picture = MSNObjectFactory::get_display_picture_no_bytes(size, user.get_email_address(), format!("{}.tmp", base64_mxc).to_string(), FriendlyName::default());
        user.display_picture = Some(display_picture)
    }

    Ok(user)
}

trait ToEmailAddress {
    fn to_email_address(&self) -> Result<EmailAddress, anyhow::Error>;
}

impl ToEmailAddress for Room {
    fn to_email_address(&self) -> Result<EmailAddress, anyhow::Error> {
        let room_info = self.clone_info();

        let room_id_format = room_info.room_version_rules_or_default().room_id_format;

        let room_id = self.room_id();
        match room_id_format {
            matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V1 => {
                let server_name = room_id
                    .server_name()
                    .expect("RoomIdV1 to contain it's server name");

                let local_part = room_id
                    .strip_sigil()
                    .strip_suffix(format!(":{}", &server_name).as_str())
                    .expect("RoomIdV1 to contain it's server name");

                let encoded_local_part = base32::encode(Alphabet::Rfc4648Lower { padding: false }, local_part.as_bytes());

                let email_str = format!("{}@{}", encoded_local_part, &server_name);
                Ok(EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid"))
            }
            matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V2 => {
                let room_create_id = room_id.strip_sigil();
                let encoded_room_create_id = base32::encode(Alphabet::Rfc4648Lower { padding: false }, room_create_id.as_bytes());

                let server_name = room_info
                    .create()
                    .expect("RoomCreateEvent to be present")
                    .creator
                    .server_name();

                let email_str = format!("{}@{}", encoded_room_create_id, &server_name);
                Ok(EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid"))
            }
            _ => {
                Err(anyhow::anyhow!("unhandled room_id_format: {}", room_id))
            }
        }
    }
}

pub trait FindRoomFromEmail {
    fn find_room_from_email(&self, email: &EmailAddress) -> Result<Option<Room>, anyhow::Error>;
}

impl FindRoomFromEmail for Client {

    /// Finds a room associated with a given email address.
    ///
    /// This function attempts to locate a room by interpreting the email address
    /// as a potential Room ID, following two possible formats:
    /// 1. A version 2 Room ID format derived from the local part of the email.
    /// 2. A version 1 Room ID format combining the local part and domain of the email.
    ///
    /// # Arguments
    ///
    /// * `email` - A reference to an `EmailAddress` instance representing the email
    ///   address to search for associated rooms.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Room))` - If a room associated with the email is found.
    /// * `Ok(None)` - If no room could be found for either Room ID format.
    /// * `Err(Error)` - If an error occurs during parsing of the Room ID or other operations.
    ///
    /// # Errors
    ///
    /// Returns an error if the parsing of a Room ID (version 1 or version 2) fails.
    ///
    fn find_room_from_email(&self, email: &EmailAddress) -> Result<Option<Room>, Error> {

        let (local_part, server_name) = email.crack();
        let decoded_local_part = String::from_utf8(base32::decode(Alphabet::Rfc4648Lower { padding: false }, local_part).ok_or(anyhow::anyhow!("Failed to decode local part"))?)?;
        
        let room_id_v2 = RoomId::parse(format!("!{}", decoded_local_part))?;
        println!("room_id_v2: {}", room_id_v2);

        if let Some(room) = self.get_room(room_id_v2.as_ref()) {
            return Ok(Some(room));
        }

        let room_id_v1 = RoomId::parse(format!("!{}:{}", decoded_local_part, server_name))?;
        println!("room_id_v1: {}", room_id_v1);

        if let Some(room) = self.get_room(room_id_v1.as_ref()) {
            return Ok(Some(room));
        }

        Ok(None)
    }
}
