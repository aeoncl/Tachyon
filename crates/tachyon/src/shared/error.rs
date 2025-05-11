use matrix_sdk::{ClientBuildError, HttpError};
use matrix_sdk::event_cache::EventCacheError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum TachyonError {

    #[error(transparent)]
    MatrixConversion(#[from] MatrixConversionError),
    #[error(transparent)]
    MatrixError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    MatrixEventCacheError(#[from] EventCacheError),
    #[error(transparent)]
    HttpError(#[from] HttpError),
    #[error(transparent)]
    ClientBuildError(#[from] ClientBuildError),
    #[error(transparent)]
    Any(#[from] anyhow::Error),
    #[error("Could not send message to Notification Client")]
    NotificationChannelError
}

#[derive(Error, Debug)]
pub enum MatrixConversionError {
    #[error("Could not convert Email to Matrix ID: {}", .email)]
    EmailToMatrixId {email: String, source: anyhow::Error},
    #[error("Could not generate Device Id")]
    DeviceIdGeneration { source: anyhow::Error}

}