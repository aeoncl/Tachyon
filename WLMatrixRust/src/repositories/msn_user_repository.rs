use base64::{engine::general_purpose, Engine};
use js_int::UInt;
use matrix_sdk::{ruma::{UserId, RoomId, events::{room::MediaSource, GlobalAccountDataEvent, direct::DirectEventContent}, api::client::media::get_content_thumbnail::v3::Method, MxcUri, OwnedRoomId, OwnedMxcUri}, Client, StoreError, media::{MediaRequest, MediaFormat, MediaThumbnailSize}, sync::JoinedRoom, room::Joined, Error};

use crate::{generated::payloads::PresenceStatus, models::{msn_user::MSNUser, msn_object::{MSNObjectFactory, MSNObject}}};



pub struct MSNUserRepository{
    matrix_client: Client
}



impl MSNUserRepository {
    pub fn new(matrix_client: Client) -> Self {
        return MSNUserRepository { matrix_client };
    }
    

    //Todo move this to a more general purpose class
    async fn get_joined_direct_room_for_user(&self, user_id: &UserId) -> Result<Option<(OwnedRoomId, Joined)>, StoreError> {

        let directs = self.matrix_client.store().get_account_data_event(matrix_sdk::ruma::events::GlobalAccountDataEventType::Direct).await?;

        if let Some(direct_ev) = directs {
            
            let directs_parsed : GlobalAccountDataEvent<DirectEventContent> = direct_ev.deserialize_as().expect("Direct Event to be formatted correctly");
         
                let content = directs_parsed.content.0;
                if let Some(rooms) = content.get(user_id) {
                    for room in rooms {
                        if let Some(found_room ) = self.matrix_client.get_joined_room(room) {
                            if let Ok(Some(_found_member)) = found_room.get_member(user_id).await {
                               return Ok(Some((room.to_owned(), found_room)));
                            }
                        } else {
                            break;
                        }
                    }
                }
        }

        return Ok(None);
    }

    async fn get_joined_room_for_user(&self, user_id: &UserId) -> Result<Option<(OwnedRoomId, Joined)>, StoreError> {

        for joined_room in self.matrix_client.joined_rooms() {
            if let Ok(Some(member)) = joined_room.get_member(user_id).await {
                return Ok(Some((joined_room.room_id().to_owned(), joined_room)));
            }
        }

        return Ok(None);
    }

    pub async fn get_msnuser_from_userid(&self, user_id: &UserId, fetch_presence: bool) -> Result<MSNUser, StoreError> {


        let maybe_found_direct = self.get_joined_direct_room_for_user(user_id).await?;
        if let Some((room_id, _room)) = maybe_found_direct {
            return self.get_msnuser(&room_id, user_id, fetch_presence).await;
        }

        let maybe_found_non_direct = self.get_joined_room_for_user(user_id).await?;

        if let Some((room_id, _room)) = maybe_found_non_direct {
            return self.get_msnuser(&room_id, user_id, fetch_presence).await;
        }

        return Ok(MSNUser::from_matrix_id(user_id.to_owned()));
    }

    
    pub async fn get_avatar_from_string(&self, avatar_mxc: String)  -> Result<Vec<u8>, Error>  {
        let owned_mxc_uri = <&MxcUri>::try_from(avatar_mxc.as_str()).unwrap().to_owned();
        return self.get_avatar(owned_mxc_uri).await;
    }


    pub async fn get_avatar(&self, avatar_mxc: OwnedMxcUri) -> Result<Vec<u8>, Error> {
        let media_request = MediaRequest{ source: MediaSource::Plain(avatar_mxc), format: MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })};
        return self.matrix_client.media().get_media_content(&media_request, true).await;
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
                

                if let Some(avatar_mxc) = content.avatar_url.as_ref() {

                    match self.get_avatar(avatar_mxc.clone()).await {
                        Ok(avatar) => {
                           out.set_display_picture(Some(self.avatar_to_msn_obj(&avatar, msn_addr.clone(), &avatar_mxc)));
                        },
                        Err(err) => {
                            log::error!("Couldn't download avatar: {} - {}", &avatar_mxc, err);
                        }
                    }
                }
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

    pub fn avatar_to_msn_obj(&self, avatar_bytes: &Vec<u8>, msn_addr: String, avatar_mxc: &OwnedMxcUri) -> MSNObject {
        let base64_mxc = general_purpose::STANDARD.encode(avatar_mxc.to_string());
        return MSNObjectFactory::get_display_picture(&avatar_bytes, msn_addr.clone(),format!("{}.tmp", base64_mxc), None);
    } 

}