use lazy_static::lazy_static;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use regex::Regex;

lazy_static! {
    static ref MSN_ADDRESS_REGEX: Regex = Regex::new(r"(.+)@(.+)").unwrap();
    static ref MX_ID_REGEX: Regex = Regex::new(r"@(.+):(.+)").unwrap();    
}

fn msn_addr_to_matrix_id(msn_addr: &String) -> String {

    let captures = MSN_ADDRESS_REGEX.captures(&msn_addr).unwrap();

    return format!("@{}:{}", captures[1].to_string(), captures[2].to_string()).to_string();
}

pub trait FromMsnAddr {
    fn from_msn_addr(msn_addr: &String) -> Self;
}

pub trait ToMsnAddr {
    fn to_msn_addr(&self) -> String;
}

impl ToMsnAddr for &UserId {
    fn to_msn_addr(&self) -> String {
        let captures = MX_ID_REGEX.captures(self.as_str()).unwrap();
        return format!("{}@{}", captures[1].to_string(), captures[2].to_string());
    }
}

impl FromMsnAddr for OwnedUserId {
    fn from_msn_addr(msn_addr: &String) -> Self {
        let matrix_user_id_string =  msn_addr_to_matrix_id(&msn_addr);
        let matrix_user_id : OwnedUserId = UserId::parse(matrix_user_id_string.as_str()).unwrap();
        return matrix_user_id;
    }   
}

impl ToMsnAddr for OwnedUserId {
    fn to_msn_addr(&self) -> String {
        let as_string = self.to_string();
        let captures = MX_ID_REGEX.captures(as_string.as_str()).unwrap();
        return format!("{}@{}", captures[1].to_string(), captures[2].to_string());
    }
}