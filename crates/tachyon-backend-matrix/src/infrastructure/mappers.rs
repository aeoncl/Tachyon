use std::convert::Infallible;
use std::str::FromStr;
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use tachyon_core::domain::ids::{DeviceId, UserId};

pub trait FromMapper<From>  {

    type Error;

    fn map_from(from: From) -> Result<Self, Self::Error> where Self: Sized;
}

pub trait IntoMapper<Into> {
    type Error;
    fn map_into(self) -> Result<Into, Self::Error>;
}

#[derive(Debug)]
pub enum MapperError {
    RumaError(String)
}


impl FromMapper<OwnedUserId> for UserId {
    type Error = Infallible;

    fn map_from(from: OwnedUserId) -> Result<Self, Self::Error> where Self: Sized {
        Ok(UserId::new(from.as_str()))
    }
}

impl IntoMapper<OwnedUserId> for UserId {
    type Error = MapperError;

    fn map_into(self) -> Result<OwnedUserId, Self::Error> {
        let user_id = OwnedUserId::from_str(self.as_str()).map_err(|e| MapperError::RumaError(format!("{}", e).to_string()))?;
        Ok(user_id)
    }
}

impl FromMapper<OwnedDeviceId> for DeviceId {
    type Error = Infallible;

    fn map_from(from: OwnedDeviceId) -> Result<Self, Self::Error> {
        Ok(DeviceId::new(from.as_str()))
    }
}

impl IntoMapper<OwnedDeviceId> for DeviceId {
    type Error = Infallible;

    fn map_into(self) -> Result<OwnedDeviceId, Self::Error> {
        Ok(OwnedDeviceId::from(self.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_id_round_trips_through_ruma() {
        let device_id = DeviceId::new("DEVICEID");

        let ruma_device_id: OwnedDeviceId = device_id.clone().map_into().unwrap();
        let mapped_back = DeviceId::map_from(ruma_device_id).unwrap();

        assert_eq!(mapped_back, device_id);
    }
}
