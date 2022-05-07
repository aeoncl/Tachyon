use lazy_static::lazy_static;
use matrix_sdk::ruma::{OwnedUserId, UserId, OwnedRoomId, RoomId};
use regex::Regex;

use crate::models::uuid::UUID;

lazy_static! {
    static ref MSN_ADDRESS_REGEX: Regex = Regex::new(r"(.+)@(.+)").unwrap();
    static ref MX_ID_REGEX: Regex = Regex::new(r"@(.+):(.+)").unwrap();

    static ref MXC_REGEX: Regex = Regex::new(r"mxc://(.+)/(.+)").unwrap();

}



pub fn msn_addr_to_matrix_id(msn_addr: &String) -> String {

    let captures = MSN_ADDRESS_REGEX.captures(&msn_addr).unwrap();

    return format!("@{}:{}", captures[1].to_string(), captures[2].to_string()).to_string();
}

pub fn msn_addr_to_matrix_user_id(msn_addr: &String) -> OwnedUserId {
    let matrix_user_id_string =  msn_addr_to_matrix_id(msn_addr);
    let matrix_user_id : OwnedUserId = <&UserId>::try_from(matrix_user_id_string.as_str()).unwrap().to_owned();
    return matrix_user_id;
}

pub fn matrix_room_id_to_annoying_matrix_room_id(matrix_room_id: &String) -> OwnedRoomId {
    let mtx_room_id : OwnedRoomId = <&RoomId>::try_from(matrix_room_id.as_str()).unwrap().to_owned();
    return mtx_room_id;
}

pub fn matrix_id_to_msn_addr(matrix_id: &String) -> String {
    let captures = MX_ID_REGEX.captures(&matrix_id).unwrap();
    return format!("{}@{}", captures[1].to_string(), captures[2].to_string()).to_string();
}

pub fn get_device_uuid() -> String {
    return UUID::from_string(&mac_address::get_mac_address().unwrap().unwrap().to_string()).to_string();
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
