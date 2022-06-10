use std::{collections::HashSet, path::Path};

use matrix_sdk::{Client, ruma::{UserId, DeviceId, device_id, user_id, events::{AnySyncMessageLikeEvent, AnySyncRoomEvent, SyncMessageLikeEvent}, OwnedUserId}, Session, Error, deserialized_responses::JoinedRoom, store::make_store_config};
use reqwest::Url;

use super::identifiers::get_matrix_device_id;



pub async fn login(matrix_id: String, matrix_token: String) -> Result<Client, Error> {


    let matrix_id_str = matrix_id.as_str();

    let matrix_user : OwnedUserId = <&UserId>::try_from(matrix_id_str).unwrap().to_owned();
    let device_id = get_matrix_device_id();
    let device_id_str = device_id.as_str();
    let device_id = device_id!(device_id_str).to_owned();
    
    let path = Path::new("c:\\temp");
    let config =  make_store_config(path, None).unwrap();

    let client = Client::builder().store_config(config).user_id(&matrix_user).build().await.unwrap();
    

    client.restore_login(Session{ access_token: matrix_token.to_owned(), user_id: matrix_user, device_id: device_id}).await?;
    let _check_connection_status = client.whoami().await?;
    return Ok(client);
}

pub fn save_mtx_timestamp(msn_addr: &String, mtx_timestamp: String) {


}

pub fn load_mtx_timestamp(msn_addr: &String) -> String {
    return String::new();
}


pub fn get_direct_target_that_isnt_me(direct_targets: &HashSet<OwnedUserId>, me: &UserId) -> Option<OwnedUserId> {

    for direct_target in direct_targets {
        if(direct_target != me) {
            return Some(direct_target.clone());
        }
    }
    return None;

}