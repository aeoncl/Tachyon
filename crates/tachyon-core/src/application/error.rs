use crate::domain::auth::Readiness;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no backend credentials stored for this token")]
    BackendCredentialsNotInStore,
    #[error("no login with that id")]
    LoginNotFound,
    #[error("the device is not verified")]
    DeviceNotVerified,
    #[error(transparent)]
    BackendError(#[from] BackendError),
    #[error(transparent)]
    StoreError(#[from] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("cannot restore login: {0}")]
    CannotRestoreLogin(String),
    #[error("the backend has logged this login out")]
    LoggedOut,
    #[error("the backend needs this login to authenticate again")]
    SoftLoggedOut,
    #[error("{0}")]
    Technical(anyhow::Error),
    #[error(transparent)]
    StoreError(#[from] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("no login with that id")]
    LoginNotFound,
    /// The login is still `Readiness::AuthNeeded`.
    #[error("login has not finished authenticating")]
    NotAuthenticated,
    /// A mutating call on a login that is already `Readiness::Ready`.
    #[error("device is already verified")]
    AlreadyVerified,
    #[error("no verification in progress")]
    NoVerificationInProgress,
    #[error("recovery key rejected")]
    RecoveryKeyRejected,
    #[error("device is still unverified")]
    StillUnverified,
    #[error(transparent)]
    Backend(#[from] BackendError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum ReadinessError {
    #[error("no session with that login id")]
    NotFound,
    #[error("readiness cannot go backwards, from {from:?} to {to:?}")]
    Backwards { from: Readiness, to: Readiness },
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The storage backend failed (I/O, database, runtime).
    #[error("{0}")]
    Technical(anyhow::Error),
    /// The row is there but cannot be read by this build (unknown format, bad data).
    #[error("stored data cannot be read by this build: {0}")]
    Corrupted(String),
}
