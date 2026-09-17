pub mod auth_service;
pub mod identity_reset;
pub mod session;
pub mod verification;

use std::fmt::Display;
use tachyon_core::application::error::BackendError;

/// The SDK failed in a way the bridge has no name for.
pub(crate) fn technical(error: impl Display) -> BackendError {
    BackendError::Technical(anyhow::anyhow!("{error}"))
}
