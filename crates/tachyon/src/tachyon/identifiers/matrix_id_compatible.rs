use std::str::FromStr;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;

pub trait MatrixIdCompatible {
    fn from_owned_user_id(value: OwnedUserId) -> Self;

    fn from_user_id(value: &UserId) -> Self;

    fn to_owned_user_id(&self) -> OwnedUserId;
}

impl MatrixIdCompatible for MsnUser {
    fn from_owned_user_id(value: OwnedUserId) -> Self {
        MsnUser::with_email_addr(EmailAddress::from_user_id(&value))
    }

    fn from_user_id(value: &UserId) -> Self {
        MsnUser::with_email_addr(EmailAddress::from_user_id(value))
    }

    fn to_owned_user_id(&self) -> OwnedUserId {
        self.get_email_address().to_owned_user_id()
    }
}

impl MatrixIdCompatible for EmailAddress {
    fn from_owned_user_id(value: OwnedUserId) -> Self {
        let name = value.localpart();
        let domain = value.server_name().as_str();

        EmailAddress::from_str(&format!("{}@{}", name, domain)).expect("OwnedUserId to be valid")
    }

    fn from_user_id(value: &UserId) -> Self {
        let name = value.localpart();
        let domain = value.server_name().as_str();
        EmailAddress::from_str(&format!("{}@{}", name, domain)).expect("UserId to be valid")
    }

    fn to_owned_user_id(&self) -> OwnedUserId {
        let as_str = self.as_str();
        let (name, domain) = as_str.split_once("@").expect("Email to contain @");
        OwnedUserId::from_str(&format!("@{}:{}", name, domain)).expect("OwnedUserId to be valid")
    }
}