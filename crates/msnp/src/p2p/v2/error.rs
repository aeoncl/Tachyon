use anyhow::anyhow;
use thiserror::Error;

use crate::msnp::error::PayloadError;

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