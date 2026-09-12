use crate::application::error::VerificationError;
use crate::application::logins::{Login, Logins, Step};
use crate::application::ports::BackendSession;
use crate::domain::auth::TachyonToken;
use crate::domain::ids::DeviceId;
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use std::sync::Arc;

/// Everything a client does between signing in and being trusted: read the device status,
/// import cross-signing secrets from a recovery key, verify against another device, or
/// reset the identity outright. It never moves a login forward; the session reports the
/// device as verified and `AuthUseCase::wait_for_session` picks that up.
pub struct DeviceVerificationUseCase {
    logins: Arc<Logins>,
}

impl DeviceVerificationUseCase {
    pub fn new(logins: Arc<Logins>) -> DeviceVerificationUseCase {
        DeviceVerificationUseCase { logins }
    }

    pub async fn status(&self, token: &TachyonToken) -> Result<DeviceStatus, VerificationError> {
        match self.login(token)? {
            Login::Ready { .. } => Ok(DeviceStatus::Verified),
            Login::Pending { session, .. } => Ok(session.device_status().await?),
        }
    }

    pub async fn options(
        &self,
        token: &TachyonToken,
    ) -> Result<VerificationOptions, VerificationError> {
        self.login(token)?.session().verification_options().await
    }

    pub async fn recover(
        &self,
        token: &TachyonToken,
        key: &RecoveryKey,
    ) -> Result<(), VerificationError> {
        let session = self.unverified_session(token)?;
        match session.recover(key).await? {
            DeviceStatus::Verified => Ok(()),
            DeviceStatus::Unverified => Err(VerificationError::StillUnverified),
        }
    }

    pub async fn start_device_verification(
        &self,
        token: &TachyonToken,
        device: &DeviceId,
    ) -> Result<(), VerificationError> {
        self.unverified_session(token)?
            .start_device_verification(device)
            .await
    }

    pub async fn verification_state(
        &self,
        token: &TachyonToken,
    ) -> Result<VerificationFlowState, VerificationError> {
        self.login(token)?
            .session()
            .verification_state()
            .ok_or(VerificationError::NoVerificationInProgress)
    }

    pub async fn verification_action(
        &self,
        token: &TachyonToken,
        action: VerificationAction,
    ) -> Result<(), VerificationError> {
        self.unverified_session(token)?
            .verification_action(action)
            .await
    }

    pub async fn reset_identity(
        &self,
        token: &TachyonToken,
        auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        self.unverified_session(token)?.reset_identity(auth).await
    }

    /// The account's login once it has authenticated.
    fn login(&self, token: &TachyonToken) -> Result<Login, VerificationError> {
        match self.logins.get(token) {
            None => Err(VerificationError::LoginNotFound),
            Some(Login::Pending {
                step: Step::Authenticate { .. },
                ..
            }) => Err(VerificationError::NotAuthenticated),
            Some(login) => Ok(login),
        }
    }

    fn unverified_session(
        &self,
        token: &TachyonToken,
    ) -> Result<Arc<dyn BackendSession>, VerificationError> {
        match self.login(token)? {
            Login::Ready { .. } => Err(VerificationError::AlreadyVerified),
            Login::Pending { session, .. } => Ok(session),
        }
    }
}
