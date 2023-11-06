use thiserror::Error;
use crate::models::conversion::error::ConversionError;

#[derive(Debug, Error)]
pub enum SwitchboardError {
    #[error(transparent)]
    MatrixSdkError(#[from] matrix_sdk::Error),
    #[error("Couldn't find matrix room")]
    MatrixRoomNotFound,
    #[error(transparent)]
    MatrixHttpError(#[from] matrix_sdk::HttpError),
    #[error(transparent)]
    MimeError(#[from] mime::FromStrError),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error("Unkown switchboard error")]
    UnknownError
}