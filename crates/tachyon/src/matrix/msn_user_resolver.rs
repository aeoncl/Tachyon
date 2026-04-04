use anyhow::anyhow;
use base64::engine::general_purpose;
use base64::Engine;

use matrix_sdk::media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings};
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::{MxcUri, UInt};
use matrix_sdk::Client;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory, MsnObject};

pub async fn avatar_mxid_to_msn_object(client: &Client, email_address: &EmailAddress, avatar_mxc: &MxcUri) -> Result<MsnObject, anyhow::Error> {
    match get_avatar_bytes(client, &avatar_mxc).await {
        Ok(avatar_bytes) => {
            Ok(avatar_to_msn_obj(&avatar_bytes, email_address, &avatar_mxc))
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub async fn get_avatar_bytes(client: &Client, avatar_mxc: &MxcUri) -> Result<Vec<u8>, anyhow::Error> {

    let thumbnail_settings = MediaThumbnailSettings::new(UInt::new(200).unwrap(), UInt::new(200).unwrap() );

    let media_request = MediaRequestParameters{ source: MediaSource::Plain(avatar_mxc.to_owned()), format:MediaFormat::Thumbnail(thumbnail_settings) };
    client.media().get_media_content(&media_request, true).await.map_err(|e| anyhow!(e))
}

pub fn avatar_to_msn_obj(avatar_bytes: &Vec<u8>, msn_addr: &EmailAddress, avatar_mxc: &MxcUri) -> MsnObject {
    let base64_mxc =  general_purpose::STANDARD.encode(avatar_mxc.to_string());
    return MSNObjectFactory::get_display_picture(&avatar_bytes, msn_addr,format!("{}.tmp", base64_mxc), FriendlyName::default());
}