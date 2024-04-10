use std::num::ParseIntError;
use anyhow::anyhow;
use matrix_sdk::{ClientBuilder, ClientBuildError, HttpError, IdParseError};
use thiserror::Error;
use uuid::Uuid;
use crate::models::notification::error::MSNPServerError;

#[derive(Error, Debug)]
#[error("A message error has occured: {}", .msg)]
pub struct MessageError {
    msg: String
}

impl MessageError {
    pub fn new(msg: String) -> Self {
        Self {msg}
    }
}

#[derive(Error, Debug)]
pub enum TachyonError {

    #[error(transparent)]
    PayloadError(#[from] PayloadError),

    #[error(transparent)]
    P2PError(#[from] P2PError),

    #[error("An error has occured while authenticating the client")]
    AuthenticationError{
        sauce: anyhow::Error
    },
    #[error("An error has occured extracting data from malformed command: {}", .command)]
    CommandSplitOutOfBounds{
        command: String
    },
    #[error(transparent)]
    MatrixError(#[from] MatrixError),

    #[error(transparent)]
    UUIDConversionError(#[from] uuid::Error)
}


#[derive(Error, Debug)]
pub enum MatrixError {
    #[error(transparent)]
    WebError(#[from] HttpError),
    #[error(transparent)]
    IdParseError(#[from] IdParseError),
    #[error(transparent)]
    SdkError(#[from] matrix_sdk::Error)
}

#[derive(Error, Debug)]
pub enum P2PError {

    #[error(transparent)]
    PayloadError(#[from] PayloadError),

    #[error("Session was closed: {}", .payload)]
    SessionClosed {
        payload: String,
        sauce: anyhow::Error
    },

    #[error(transparent)]
    AnyError(#[from] anyhow::Error)

}

#[derive(Error, Debug)]
pub enum PayloadError {
    #[error("Couldn't parse the payload: {}", .payload)]
    StringPayloadParsingError {
        payload: String,
        sauce: anyhow::Error

    },
    #[error("Couldn't parse the binary payload: {:?}", .payload)]
    BinaryPayloadParsingError {
        payload: Vec<u8>,
        sauce: anyhow::Error
    },
    #[error("Couldn't parse enum: {:?}", .payload)]
    EnumParsingError {
        payload: String,
        sauce: anyhow::Error
    },
    #[error("The payload was chunked & not complete")]
    PayloadBytesMissing,
    #[error("The payload did not contain SLP packet")]
    PayloadDoesNotContainsSLP,
    #[error("The payload type is unknown to us & not handled {}", .payload)]
    PayloadNotHandled {payload: String},
    #[error("The payload did not contain a mandatory part {} - payload: {:?}", .name, .payload)]
    MandatoryPartNotFound{ name: String, payload: String},
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    AnyError(#[from] anyhow::Error)

}

impl From<ClientBuildError> for TachyonError {
    fn from(value: ClientBuildError) -> Self {
        Self::AuthenticationError {sauce: anyhow!(value).context("Couldn't build the Matrix Client")}
    }
}