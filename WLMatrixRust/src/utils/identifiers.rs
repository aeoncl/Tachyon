use lazy_static::lazy_static;
use regex::Regex;

use crate::models::uuid::UUID;

lazy_static! {
    static ref MSN_ADDRESS_REGEX: Regex = Regex::new(r"(.+)@(.+)").unwrap();
    static ref MXC_REGEX: Regex = Regex::new(r"mxc://(.+)/(.+)").unwrap();

}



pub fn msn_addr_to_matrix_id(msn_addr: &String) -> String {

    let captures = MSN_ADDRESS_REGEX.captures(&msn_addr).unwrap();

    return format!("@{}:{}", captures[1].to_string(), captures[2].to_string()).to_string();
}

pub fn get_device_uuid() -> String {
    return UUID::from_string(&mac_address::get_mac_address().unwrap().unwrap().to_string()).to_string();
}

pub fn get_matrix_device_id() -> String {
    return format!("WLMatrix[{}]", get_device_uuid());
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
        assert_eq!(result.is_empty(), false);
    }

}
