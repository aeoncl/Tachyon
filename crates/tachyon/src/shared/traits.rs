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

pub trait TryFromMsnAddr {
    fn try_from_msn_addr(msn_addr: &str) -> Result<Self, MatrixConversionError> where Self: Sized;
}

impl TryFromMsnAddr for OwnedUserId {
    fn try_from_msn_addr(msn_addr: &str) -> Result<Self, MatrixConversionError> {
        let (name, server) =  msn_addr.split_once("@").ok_or(MatrixConversionError::EmailToMatrixId { email: msn_addr.to_string(), source: anyhow!("Email address didn't split correctly on @")})?;
        let matrix_user_id_string = format!("@{}:{}", name, server);
        let matrix_user_id : OwnedUserId = UserId::parse(&matrix_user_id_string).map_err(|e| MatrixConversionError::EmailToMatrixId { email: msn_addr.to_string(), source: e.into() } )?;
        return Ok(matrix_user_id);
    }
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