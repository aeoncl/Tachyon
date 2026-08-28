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

#[derive(Debug)]
pub enum BackendError {
    CannotRestoreLogin(String),
    LoggedOut,
    SoftLoggedOut,
    Technical(anyhow::Error),
    StoreError(StoreError),
}

pub enum SessionError {


}

impl From<StoreError> for BackendError {
    fn from(value: StoreError) -> Self {
        BackendError::StoreError(value)
    }
}

#[derive(Debug)]
pub enum StoreError {}