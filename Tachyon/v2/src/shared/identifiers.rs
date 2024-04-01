use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use log::error;
use matrix_sdk::ruma::UserId;
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