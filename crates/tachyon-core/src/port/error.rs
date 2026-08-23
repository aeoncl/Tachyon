#[derive(Debug)]
pub enum BackendError {
    CannotRestoreLogin(String),
    Technical(anyhow::Error),
    StoreError(StoreError),
}

impl From<StoreError> for BackendError {
    fn from(value: StoreError) -> Self {
        BackendError::StoreError(value)
    }
}

#[derive(Debug)]
pub enum StoreError {}
