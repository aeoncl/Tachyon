use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;
use log::error;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::uuid::Uuid;
use crate::shared::error::MatrixConversionError;

pub struct MatrixDeviceId(String);

impl MatrixDeviceId {
    pub fn from_hostname() -> Result<Self, MatrixConversionError> {
        let device_id = hostname::get()
                            .map_err(|e| MatrixConversionError::DeviceIdGeneration { source: e.into()})?.to_str()
                            .ok_or(MatrixConversionError::DeviceIdGeneration {source: anyhow!("Could'nt parse OString to &str")})?
                            .to_string();

        Ok(MatrixDeviceId(device_id))
    }

}



impl Display for MatrixDeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub trait MatrixIdCompatible {
    fn from_owned(value: OwnedUserId) -> Self;

    fn from(value: &UserId) -> Self;

    fn into_owned(self) -> OwnedUserId;
}


impl MatrixIdCompatible for EmailAddress {
     fn from_owned(value: OwnedUserId) -> Self {
        let name = value.localpart();
        let domain = value.server_name().as_str();

        EmailAddress::from_str(&format!("{}@{}", name, domain)).expect("OwnedUserId to be valid")
    }

    fn from(value: &UserId) -> Self {
        let name = value.localpart();
        let domain = value.server_name().as_str();
        EmailAddress::from_str(&format!("{}@{}", name, domain)).expect("UserId to be valid")
    }

    fn into_owned(self) -> OwnedUserId {
        let as_str : String = self.into();
        let (name, domain) = as_str.split_once("@").expect("Email to contain @");
        OwnedUserId::from_str(&format!("@{}:{}", name, domain)).expect("OwnedUserId to be valid")
    }
}
