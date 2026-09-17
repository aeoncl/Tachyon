#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no login with that id")]
    LoginNotFound,
    /// A login for the token is live already, pending or ready. One client per token.
    #[error("a login for that token is live already")]
    AlreadySignedIn,
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

impl BackendError {
    /// Whether the backend has said the login is over, as opposed to being out of reach
    /// right now. One that is over is not worth keeping; one that is merely unreachable
    /// must not cost the user their sign-in.
    pub fn ends_the_login(&self) -> bool {
        matches!(
            self,
            Self::LoggedOut | Self::SoftLoggedOut | Self::CannotRestoreLogin(_)
        )
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unreachable_backend_does_not_cost_the_user_their_login() {
        assert!(BackendError::LoggedOut.ends_the_login());
        assert!(BackendError::SoftLoggedOut.ends_the_login());
        assert!(BackendError::CannotRestoreLogin("corrupt".into()).ends_the_login());
        assert!(!BackendError::Technical(anyhow::anyhow!("timeout")).ends_the_login());
    }
}
