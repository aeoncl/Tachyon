use crate::matrix::extensions::direct::DirectRoom;
use crate::shared::identifiers::MatrixIdCompatible;
use anyhow::Error;
use base64::engine::general_purpose;
use base64::Engine;
use dashmap::DashMap;
use lazy_static::lazy_static;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::{OwnedRoomId, RoomId, UserId};
use matrix_sdk::{Client, Room};
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
use msnp::shared::models::{email_address::EmailAddress, msn_user::MsnUser};
use sha1::digest::DynDigest;
use sha1::{Digest, Sha1};
use std::str::FromStr;
use dashmap::mapref::one::Ref;

lazy_static! {
    static ref ROOM_HASH_TABLE: DashMap<String, OwnedRoomId> = DashMap::new();
    static ref ROOM_HASH_CACHE: DashMap<OwnedRoomId, String> = DashMap::new();

}

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
    


        if let Ok(display_name) = room.display_name().await {
            user.display_name = Some(display_name.to_string());
        } else {
            user.display_name = Some(room.room_id().to_string());
        }


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

pub trait ToRoomId {
    fn to_room_id(&self) -> Result<OwnedRoomId, anyhow::Error>;
}

pub trait ToEmailAddress {
    fn to_email_address(&self) -> Result<EmailAddress, anyhow::Error>;
}


impl ToEmailAddress for Room {
    fn to_email_address(&self) -> Result<EmailAddress, anyhow::Error> {
        let room_info = self.clone_info();

        let room_id_format = room_info.room_version_rules_or_default().room_id_format;

        let room_id = self.room_id();

        let room_id_hashed = hash_room_id_cached(room_id);

        match room_id_format {
            matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V1 => {
                let server_name = room_id
                    .server_name()
                    .expect("RoomIdV1 to contain it's server name");

                let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                    "t.local"
                } else {
                    server_name.as_str()
                };

                let email_str = format!("{}@{}", room_id_hashed, &domain);
                Ok(EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid"))
            }
            matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V2 => {

                let server_name = room_info
                    .create()
                    .expect("RoomCreateEvent to be present")
                    .creator
                    .server_name();

                let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                    "t.local"
                } else {
                    server_name.as_str()
                };

                let email_str = format!("{}@{}", room_id_hashed, &domain);
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
    
    fn find_room_from_email(&self, email: &EmailAddress) -> Result<Option<Room>, Error> {

        let (room_id_hashed, server_name) = email.crack();

        let out = if let Some(entry) = ROOM_HASH_TABLE.get(room_id_hashed) {
            let room_id = entry.value();
            self.get_room(room_id.as_ref())
        } else {
            
            let mut found = None;
            
            for curent_room in self.rooms() {
                let current_room_hash = hash_room_id_cached(curent_room.room_id());
                if current_room_hash == room_id_hashed {
                    found = Some(curent_room);
                    break;
                }
            }
            found
        };

        Ok(out)
    }
}


fn hash_room_id_cached(room_id: &RoomId) -> String {
    match ROOM_HASH_CACHE.get(room_id) {
        None => {
            let hash = hash_room_id(room_id);
            ROOM_HASH_TABLE.insert(hash.clone(), room_id.to_owned());
            ROOM_HASH_CACHE.insert(room_id.to_owned(), hash.clone());
            hash
        }
        Some(hash) => { hash.value().clone() }
    }
}
fn hash_room_id(room_id: &RoomId) -> String {
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, room_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

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