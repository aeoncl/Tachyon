use std::str::FromStr;
use anyhow::Error;
use base64::Engine;
use base64::engine::general_purpose;
use crate::tachyon::client::tachyon_client::TachyonClient;
use dashmap::DashMap;
use matrix_sdk::media::{MediaFormat, MediaThumbnailSettings};
use matrix_sdk::room::RoomMember;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use ruma::{OwnedRoomId, RoomId};
use sha1::{Digest, Sha1};
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
use msnp::shared::models::msn_user::MsnUser;
use crate::matrix::extensions::direct::DirectRoom;

pub struct RoomMappingIdCache {
    pub email_to_room: DashMap<EmailAddress, OwnedRoomId>,
    pub room_to_email: DashMap<OwnedRoomId, EmailAddress>,
}

impl Default for RoomMappingIdCache {
    fn default() -> Self {
        Self {
            email_to_room: Default::default(),
            room_to_email: Default::default(),
        }
    }
}


fn hash_room_id(room_id: &RoomId) -> String {
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, room_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

impl TachyonClient {

    pub async fn resolve_room_to_msn_user(&self, room_id: &RoomId) -> Result<MsnUser, anyhow::Error> {
        let endpoint_id = self.resolve_room_to_endpoint_id(room_id)?;
        let room = self.matrix_client().get_room(room_id).ok_or(anyhow::anyhow!("room not found"))?;

        let direct_target = if room.is_valid_one_to_one_direct() {
            room.get_single_direct_target_member().await.unwrap_or(None)
        } else {
            None
        };


        let display_name = match &direct_target {
            None => {
                room.display_name().await.map(|d| d.to_string()).unwrap_or(room.room_id().to_string())
            }
            Some(direct_target) => {
                direct_target.display_name().map(|d| d.to_string()).unwrap_or(direct_target.user_id().to_string())
            }
        };

        let avatar = self.get_avatar_as_msn_object(room_id).await?;

        let mut user = MsnUser::new(endpoint_id);
        user.display_name = Some(display_name);

        user.display_picture = avatar;

        Ok(user)

    }

    pub fn resolve_room_to_endpoint_id(&self, room_id: &RoomId) -> Result<EndpointId, anyhow::Error> {
        self.resolve_room_to_email_address(room_id).map(|e| EndpointId::from_email_addr(e))
    }

    pub fn resolve_room_to_email_address(&self, room_id: &RoomId) -> Result<EmailAddress, anyhow::Error> {
        if let Some(cached) = self.inner.room_id_mapping_cache.room_to_email.get(room_id).map(|addr| addr.to_owned()) {
            return Ok(cached);
        }

        let Some(room) = self.matrix_client().get_room(room_id) else {
            return anyhow::bail!("room not found: {}", room_id);
        };

        let room_info = room.clone_info();
        let room_id_format = room_info.room_version_rules_or_default().room_id_format;

        let room_id_hashed = hash_room_id(room_id);

        let email = match room_id_format {
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
                EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid")
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
                EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid")
            }
            _ => {
                return Err(anyhow::anyhow!("unhandled room_id_format: {}", room_id));
            }
        };

        self.inner.room_id_mapping_cache.email_to_room.insert(email.clone(), room_id.to_owned());
        self.inner.room_id_mapping_cache.room_to_email.insert(room_id.to_owned(), email.clone());

        Ok(email)
    }

    pub fn resolve_room_id_from_email(&self, email: &EmailAddress) -> Option<OwnedRoomId> {
        if let Some(found) = self.inner.room_id_mapping_cache.email_to_room.get(email).map(|e| e.value().clone()) {
            return Some(found)
        };

        self.matrix_client().rooms().iter().find_map(|room| {
            match self.resolve_room_to_email_address(room.room_id()) {
                Ok(candidate) => {
                    if candidate == *email {
                        Some(room.room_id().to_owned())
                    } else {
                        None
                    }
                }
                Err(err) => {
                    log::error!("An error has occured resolving room_id from email ({}): {}", email, err);
                    None
                }
            }
        })
    }



}