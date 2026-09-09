use crate::application::error::VerificationError;
use crate::application::ports::{AccountRepository, BackendSession, SessionRepository};
use crate::domain::auth::{Readiness, TachyonToken};
use crate::domain::ids::DeviceId;
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use std::sync::Arc;

/// Everything a client does between signing in and being trusted: read the device status,
/// import cross-signing secrets from a recovery key, verify against another device, or
/// reset the identity outright. It never writes `Readiness` — `AuthUseCase::restore` does
/// that, once, when the bridge comes back to collect the session.
pub struct DeviceVerificationUseCase {
    account_repository: Arc<dyn AccountRepository>,
    session_repository: Arc<dyn SessionRepository>,
}

impl DeviceVerificationUseCase {
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        session_repository: Arc<dyn SessionRepository>,
    ) -> DeviceVerificationUseCase {
        DeviceVerificationUseCase {
            account_repository,
            session_repository,
        }
    }

    pub async fn status(&self, token: &TachyonToken) -> Result<DeviceStatus, VerificationError> {
        let (session, readiness) = self.session(token).await?;
        if readiness == Readiness::Ready {
            return Ok(DeviceStatus::Verified);
        }
        Ok(session.device_status().await?)
    }

    pub async fn options(
        &self,
        token: &TachyonToken,
    ) -> Result<VerificationOptions, VerificationError> {
        let (session, _) = self.session(token).await?;
        session.verification_options().await
    }

    pub async fn recover(
        &self,
        token: &TachyonToken,
        key: &RecoveryKey,
    ) -> Result<(), VerificationError> {
        let session = self.unverified_session(token).await?;
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
        let session = self.unverified_session(token).await?;
        session.start_device_verification(device).await
    }

    pub async fn verification_state(
        &self,
        token: &TachyonToken,
    ) -> Result<VerificationFlowState, VerificationError> {
        let (session, _) = self.session(token).await?;
        session
            .verification_state()
            .ok_or(VerificationError::NoVerificationInProgress)
    }

    pub async fn verification_action(
        &self,
        token: &TachyonToken,
        action: VerificationAction,
    ) -> Result<(), VerificationError> {
        let session = self.unverified_session(token).await?;
        session.verification_action(action).await
    }

    pub async fn reset_identity(
        &self,
        token: &TachyonToken,
        auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        let session = self.unverified_session(token).await?;
        session.reset_identity(auth).await
    }

    async fn session(
        &self,
        token: &TachyonToken,
    ) -> Result<(Arc<dyn BackendSession>, Readiness), VerificationError> {
        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return Err(VerificationError::LoginNotFound);
        };
        let Some(entry) = self.session_repository.get(&login_id) else {
            return Err(VerificationError::LoginNotFound);
        };
        if entry.readiness == Readiness::AuthNeeded {
            return Err(VerificationError::NotAuthenticated);
        }
        Ok((entry.session, entry.readiness))
    }

    async fn unverified_session(
        &self,
        token: &TachyonToken,
    ) -> Result<Arc<dyn BackendSession>, VerificationError> {
        let (session, readiness) = self.session(token).await?;
        if readiness == Readiness::Ready {
            return Err(VerificationError::AlreadyVerified);
        }
        Ok(session)
    }
}
