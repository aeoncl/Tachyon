#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no login with that id")]
    LoginNotFound,
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
    /// The login is still at `Step::Authenticate`.
    #[error("login has not finished authenticating")]
    NotAuthenticated,
    /// A mutating call on a login that is already ready.
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
pub enum StoreError {
    /// The storage backend failed (I/O, database, runtime).
    #[error("{0}")]
    Technical(anyhow::Error),
    /// The row is there but cannot be read by this build (unknown format, bad data).
    #[error("stored data cannot be read by this build: {0}")]
    Corrupted(String),
}
