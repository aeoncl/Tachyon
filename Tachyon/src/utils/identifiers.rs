use anyhow::anyhow;
use lazy_static::lazy_static;
use matrix_sdk::ruma::{OwnedRoomId, RoomId};
use rand::Rng;
use regex::Regex;
use crate::models::tachyon_error::PayloadError;

use crate::models::uuid::UUID;

lazy_static! {
    static ref MXC_REGEX: Regex = Regex::new(r"mxc://(.+)/(.+)").unwrap();
}

pub fn matrix_room_id_to_annoying_matrix_room_id(matrix_room_id: &String) -> OwnedRoomId {
    let mtx_room_id : OwnedRoomId = <&RoomId>::try_from(matrix_room_id.as_str()).unwrap().to_owned();
    return mtx_room_id;
}

pub fn get_device_uuid() -> String {
    return UUID::from_string(&mac_address::get_mac_address().unwrap().unwrap().to_string()).to_string();
}


pub fn trim_endpoint_guid(endpoint_guid: &str) -> Result<&str, PayloadError> {
         endpoint_guid.trim().strip_prefix("{")
        .ok_or(PayloadError::StringPayloadParsingError { payload: endpoint_guid.to_string(), sauce: anyhow!("Error stripping {{ prefix from GUID: {}", &endpoint_guid)})?
        .strip_suffix("}")
        .ok_or(PayloadError::StringPayloadParsingError { payload: endpoint_guid.to_string(), sauce: anyhow!("Error stripping }} suffix from GUID: {}", &endpoint_guid)})
}



pub fn get_matrix_device_id() -> String {
    return get_hostname();
}

pub fn get_hostname() -> String {
    return String::from(hostname::get().unwrap().to_str().unwrap());
}

pub fn parse_mxc(mxc : &String) -> (String, String) {

    let captures = MXC_REGEX.captures(mxc).unwrap();
    return (captures[1].to_string(), captures[2].to_string());
}

pub fn get_sb_session_id() -> String{
    let mut rng = rand::thread_rng();
    let n2: u16 = rng.gen();
    return n2.to_string();
}


#[cfg(test)]
mod tests {
    use super::get_hostname;

    #[test]
    fn test_get_hostname() {

        let result = get_hostname();
        print!("{}", result);
        assert_eq!(result.is_empty(), false);
    }

}
