use std::{collections::HashSet, path::Path};

use matrix_sdk::{Client, ruma::{UserId, DeviceId, device_id, user_id, events::{AnySyncMessageLikeEvent, SyncMessageLikeEvent, OriginalSyncStateEvent, room::member::{RoomMemberEventContent, MembershipState}}, OwnedUserId}, Session, Error};
use reqwest::Url;

use super::identifiers::get_matrix_device_id;



pub async fn login(matrix_id: String, matrix_token: String) -> Result<Client, Error> {


    let matrix_id_str = matrix_id.as_str();

    let matrix_user : OwnedUserId = <&UserId>::try_from(matrix_id_str).unwrap().to_owned();
    let device_id = get_matrix_device_id();
    let device_id_str = device_id.as_str();
    let device_id = device_id!(device_id_str).to_owned();
    
    let path = Path::new("c:\\temp");
    match Client::builder().disable_ssl_verification().server_name(matrix_user.server_name()).sled_store(path, None).build().await {
        Ok(client) => {
            client.restore_session(Session{ access_token: matrix_token.to_owned(), refresh_token: None, user_id: matrix_user, device_id: device_id}).await?;
            let _check_connection_status = client.whoami().await?;
            return Ok(client);
        },
        Err(err) => {
            return Err(Error::UnknownError(Box::new(err)));
        }
    }
   
}

pub fn save_mtx_timestamp(msn_addr: &String, mtx_timestamp: String) {


}

pub fn load_mtx_timestamp(msn_addr: &String) -> String {
    return String::new();
}

