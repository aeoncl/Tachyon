use anyhow::anyhow;
use lazy_static::lazy_static;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use msnp::shared::models::uuid::Uuid;
use regex::Regex;
use crate::shared::error::MatrixConversionError;

lazy_static! {
    static ref MSN_ADDRESS_REGEX: Regex = Regex::new(r"(.+)@(.+)").expect("To be a valid regex");
    static ref MX_ID_REGEX: Regex = Regex::new(r"@(.+):(.+)").expect("To be a valid regex");
}

pub trait ToUuid {
    fn to_uuid(&self) -> Uuid;
}

impl ToUuid for OwnedUserId {
    fn to_uuid(&self) -> Uuid {
        Uuid::from_seed(&self.to_string())
    }
}

impl ToUuid for &UserId {
    fn to_uuid(&self) -> Uuid {
        Uuid::from_seed(&self.to_string())
    }
}