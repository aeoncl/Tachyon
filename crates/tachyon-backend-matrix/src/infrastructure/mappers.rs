use std::convert::Infallible;
use std::iter::Map;
use std::str::FromStr;
use std::sync::Arc;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::serde::DisplayAsRefStr;
use tachyon_core::domain::ids::UserId;

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

