use anyhow::anyhow;
use matrix_sdk::{Client, Room};
use matrix_sdk::crypto::vodozemac::base64_encode;
use matrix_sdk::media::{MediaFormat, MediaRequest, MediaThumbnailSize};
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::{MxcUri, UInt, UserId};
use matrix_sdk::ruma::api::client::media::get_content_thumbnail::v3::Method;
use matrix_sdk::ruma::events::room::MediaSource;

use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_object::{FriendlyName, MsnObject, MSNObjectFactory};
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::presence_status::PresenceStatus;

use crate::notification::client_store::ClientData;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::PresenceStateCompatible;


pub fn resolve_msn_user_lean(user_id: &UserId, client_data: &ClientData) -> MsnUser {
    let client = client_data.get_matrix_client();

    if user_id == client.user_id().expect("to be here") {
        return client_data.get_user_clone().expect("to be here");
    }

    MsnUser::with_email_addr(EmailAddress::from_user_id(user_id))
}

pub async fn resolve_msn_user_from_rm(room_member: &RoomMember, client_data: &ClientData, profile: bool, presence: bool) -> Result<MsnUser, anyhow::Error> {
    let mut out = resolve_msn_user_lean(room_member.user_id(), client_data);
    let client = client_data.get_matrix_client();
    out = resolve_msn_user_from_rm_internal(out, room_member, &client, profile, presence).await?;
    Ok(out)
}

async fn resolve_msn_user_from_rm_internal(mut out: MsnUser, room_member: &RoomMember, client: &Client, profile: bool, presence: bool) -> Result<MsnUser, anyhow::Error> {

    if profile {
        out.display_name = room_member.display_name().clone().map(|s| s.to_string());

        let avatar_url = room_member.avatar_url();
        let avatar_bytes = room_member.avatar(MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap()})).await?;

        match avatar_bytes {
            None => {},
            Some(avatar_bytes) => {
                let avatar_url = avatar_url.expect("to be here");
                let avatar = avatar_to_msn_obj(&avatar_bytes, out.get_email_address(), avatar_url);
                out.display_picture = Some(avatar);
            }
        }
    }

    if presence {
        let presence_event = client.store().get_presence_event(room_member.user_id()).await?;
        if let Some(presence_ev) = presence_event {
            let presence_ev = presence_ev.deserialize()?;
            out.status = PresenceStatus::from_presence_state(presence_ev.content.presence);
            out.psm = presence_ev.content.status_msg.unwrap_or(String::new());
        }
    }

    Ok(out)

}



pub async fn resolve_msn_user(user_id: &UserId, room: Option<Room>, client_data: &ClientData, profile: bool, presence: bool) -> Result<MsnUser, anyhow::Error> {
    let mut out = resolve_msn_user_lean(user_id, client_data);

    let client = client_data.get_matrix_client();

    if profile {
        let room = resolve_room(user_id, room, &client).await?;
        match room {
            Some(room) => {
                let rm = room.get_member_no_sync(user_id).await?.expect("to be here");
                out = resolve_msn_user_from_rm_internal(out, &rm, &client, true, false).await?;

            }
            None => {
                let profile = client.account().fetch_user_profile_of(user_id).await?;
                out.display_name = profile.displayname.clone().map(|s| s.to_string());

                match profile.avatar_url {
                    None => {}
                    Some(avatar_mxid) => {
                        let avatar_bytes = get_avatar_bytes(&client, &avatar_mxid).await?;
                        let avatar = avatar_to_msn_obj(&avatar_bytes, out.get_email_address(), &avatar_mxid);
                        out.display_picture = Some(avatar);
                    }
                }
            }
        };

    }

    if presence {
            let presence_event = client.store().get_presence_event(&user_id).await?;
            if let Some(presence_ev) = presence_event {
                let presence_ev = presence_ev.deserialize()?;
                out.status = PresenceStatus::from_presence_state(presence_ev.content.presence);
                out.psm = presence_ev.content.status_msg.unwrap_or(String::new());
            }
    }

    Ok(out)
}

async fn resolve_room(user_id: &UserId, room: Option<Room>, client: &Client) -> Result<Option<Room>, anyhow::Error> {
    if room.is_some() {
        return Ok(room);
    }

    let room = client.get_dm_room(user_id);

    if room.is_some() {
        return Ok(room);
    }

    for room in client.joined_rooms() {
        match room.get_member_no_sync(user_id).await? {
            None => {}
            Some(found) => {
                return Ok(Some(room));
            }
        }
    }

    return Ok(None);
}

pub async fn get_avatar_bytes(client: &Client, avatar_mxc: &MxcUri) -> Result<Vec<u8>, anyhow::Error> {
    let media_request = MediaRequest{ source: MediaSource::Plain(avatar_mxc.to_owned()), format:MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap()}) };
    client.media().get_media_content(&media_request, true).await.map_err(|e| anyhow!(e))
}

pub fn avatar_to_msn_obj(avatar_bytes: &Vec<u8>, msn_addr: &EmailAddress, avatar_mxc: &MxcUri) -> MsnObject {
    let base64_mxc = base64_encode(avatar_mxc.to_string());
    return MSNObjectFactory::get_display_picture(&avatar_bytes, msn_addr,format!("{}.tmp", base64_mxc), FriendlyName::default());
}