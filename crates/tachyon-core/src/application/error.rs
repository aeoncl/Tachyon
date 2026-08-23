use crate::port::error::{BackendError, StoreError};

#[derive(Debug)]
pub enum AuthError {
    BackendCredentialsNotInStore,
    BackendError(BackendError),
    StoreError(StoreError),
}

impl From<BackendError> for AuthError {
    fn from(value: BackendError) -> Self {
        Self::BackendError(value)
    }
}

impl From<StoreError> for AuthError {
    fn from(value: StoreError) -> Self {
        Self::StoreError(value)
    }
}
