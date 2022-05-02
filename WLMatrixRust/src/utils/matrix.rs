use matrix_sdk::{Client, ruma::{UserId, DeviceId, device_id, user_id, OwnedUserId, events::{AnySyncMessageLikeEvent, AnySyncRoomEvent, SyncMessageLikeEvent}}, Session, Error, deserialized_responses::JoinedRoom};
use reqwest::Url;

use super::identifiers::get_matrix_device_id;



pub async fn login(matrix_id: String, matrix_token: String) -> Result<Client, Error> {


    let matrix_id_str = matrix_id.as_str();

    let matrix_user : OwnedUserId = <&UserId>::try_from(matrix_id_str).unwrap().to_owned();
    let device_id = get_matrix_device_id();
    let device_id_str = device_id.as_str();
    let device_id = device_id!(device_id_str).to_owned();

    //let homeserver_url = Url::parse(format!("https://{}", matrix_user.server_name()).as_str())?;
    let homeserver_url = Url::parse(format!("http://{}:8008", matrix_user.server_name()).as_str())?;

    let client = Client::new(homeserver_url).await?; //Todo fix this
    client.restore_login(Session{ access_token: matrix_token.to_owned(), user_id: matrix_user, device_id: device_id}).await?;
    let _check_connection_status = client.whoami().await?;
    return Ok(client);

}