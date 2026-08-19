use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::msn_user_resolver::ToEmailAddress;
use crate::tachyon::client::tachyon_client::TachyonClient;
use base64::engine::general_purpose;
use base64::Engine;
use matrix_sdk::media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings};
use matrix_sdk::Room;
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory, MsnObject};
use ruma::events::room::MediaSource;
use ruma::media::Method;
use ruma::{OwnedMxcUri, RoomId, UInt, UserId};

impl TachyonClient {
    pub async fn get_avatar_as_msn_object(
        &self,
        room_id: &RoomId,
    ) -> Result<Option<MsnObject>, anyhow::Error> {
        let matrix_client = self.matrix_client();
        let out = match matrix_client.get_room(room_id) {
            None => None,
            Some(room) => match room.get_single_direct_target() {
                None => self.get_room_avatar_as_msn_object(room_id).await?,
                Some(direct) => self.get_user_avatar_as_msn_object(&direct, room_id).await?,
            },
        };

        Ok(out)
    }

    pub async fn get_user_avatar_as_msn_object(
        &self,
        user_id: &UserId,
        room_id: &RoomId,
    ) -> Result<Option<MsnObject>, anyhow::Error> {
        let matrix_client = self.matrix_client();

        let out = match matrix_client.get_room(room_id) {
            None => None,
            Some(room) => {
                let avatar_bytes = self.get_user_avatar_thumbnail(user_id, &room).await?;
                let room_email_address = room.to_email_address()?;
                 match avatar_bytes {
                    None => None,
                    Some((uri, avatar_bytes)) => {
                        let base64_mxc = general_purpose::STANDARD.encode(uri.to_string());
                        Some(MSNObjectFactory::get_display_picture(
                            avatar_bytes.as_slice(),
                            &room_email_address,
                            format!("{}.tmp", base64_mxc),
                            FriendlyName::default(),
                        ))
                    }
                }
            },
        };



        Ok(out)
    }

    pub async fn get_room_avatar_as_msn_object(
        &self,
        room_id: &RoomId,
    ) -> Result<Option<MsnObject>, anyhow::Error> {
        let matrix_client = self.matrix_client();
        let out = match matrix_client.get_room(room_id) {
            None => None,
            Some(room) => {
                let room_email_address = room.to_email_address()?;
                match self.get_room_avatar_thumbnail(&room).await? {
                    None => None,
                    Some((uri, avatar_bytes)) => {
                        let base64_mxc = general_purpose::STANDARD.encode(uri.to_string());
                        Some(MSNObjectFactory::get_display_picture(
                            avatar_bytes.as_slice(),
                            &room_email_address,
                            format!("{}.tmp", base64_mxc),
                            FriendlyName::default(),
                        ))
                    }
                }
            }
        };

        Ok(out)
    }

    pub async fn get_avatar_thumbnail(&self, room: &Room) -> Result<Option<(OwnedMxcUri, Vec<u8>)>, anyhow::Error>  {
        let out = match room.get_single_direct_target() {
            None => self.get_room_avatar_thumbnail(room).await?,
            Some(direct) => self.get_user_avatar_thumbnail(&direct, room).await?,
        };
        Ok(out)
    }

    pub async fn get_room_avatar_thumbnail(
        &self,
        room: &Room,
    ) -> Result<Option<(OwnedMxcUri, Vec<u8>)>, anyhow::Error> {
        let matrix_client = self.matrix_client();

        let avatar_url = room.avatar_url();
        let out = match avatar_url {
            None => None,
            Some(avatar_url) => {
                let format = MediaFormat::Thumbnail(MediaThumbnailSettings {
                    method: Method::Crop,
                    width: UInt::new_saturating(120),
                    height: UInt::new_saturating(120),
                    animated: true,
                });
                let request = MediaRequestParameters {
                    source: MediaSource::Plain(avatar_url.to_owned()),
                    format,
                };
                let avatar_bytes = matrix_client
                    .media()
                    .get_media_content(&request, true)
                    .await?;

                Some((avatar_url.to_owned(), avatar_bytes))
            }
        };

        Ok(out)
    }

    pub async fn get_user_avatar_thumbnail(
        &self,
        user_id: &UserId,
        room: &Room,
    ) -> Result<Option<(OwnedMxcUri, Vec<u8>)>, anyhow::Error> {
        let matrix_client = self.matrix_client();

        let member = room.get_member(user_id).await?;
        let out = match member {
            None => None,
            Some(member) => {
                let avatar_url = member.avatar_url();
                match avatar_url {
                    None => None,
                    Some(avatar_url) => {
                        let format = MediaFormat::Thumbnail(MediaThumbnailSettings {
                            method: Method::Crop,
                            width: UInt::new_saturating(120),
                            height: UInt::new_saturating(120),
                            animated: true,
                        });
                        let request = MediaRequestParameters {
                            source: MediaSource::Plain(avatar_url.to_owned()),
                            format,
                        };
                        let avatar_bytes = matrix_client
                            .media()
                            .get_media_content(&request, true)
                            .await?;

                        Some((avatar_url.to_owned(), avatar_bytes))
                    }
                }
            }
        };

        Ok(out)
    }
}
