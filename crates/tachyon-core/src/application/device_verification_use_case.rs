use crate::application::error::VerificationError;
use crate::application::logins::{Login, Logins, Step};
use crate::application::ports::BackendSession;
use crate::domain::auth::BridgeLinkToken;
use crate::domain::ids::DeviceId;
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use std::sync::Arc;

pub struct DeviceVerificationUseCase {
    logins: Arc<Logins>,
}

impl DeviceVerificationUseCase {
    pub fn new(logins: Arc<Logins>) -> DeviceVerificationUseCase {
        DeviceVerificationUseCase { logins }
    }

    pub async fn status(&self, token: &BridgeLinkToken) -> Result<DeviceStatus, VerificationError> {
        match self.authenticated_login(token)? {
            Login::Ready { .. } => Ok(DeviceStatus::Verified),
            Login::Pending { session, .. } => Ok(session.device_status().await?),
        }
    }

    pub async fn options(
        &self,
        token: &BridgeLinkToken,
    ) -> Result<VerificationOptions, VerificationError> {
        self.authenticated_login(token)?.session().verification_options().await
    }

    pub async fn recover(
        &self,
        token: &BridgeLinkToken,
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
        token: &BridgeLinkToken,
        device: &DeviceId,
    ) -> Result<(), VerificationError> {
        self.unverified_session(token)?
            .start_device_verification(device)
            .await
    }

    pub fn verification_state(
        &self,
        token: &BridgeLinkToken,
    ) -> Result<VerificationFlowState, VerificationError> {
        self.authenticated_login(token)?
            .session()
            .verification_state()
            .ok_or(VerificationError::NoVerificationInProgress)
    }

    pub async fn verification_action(
        &self,
        token: &BridgeLinkToken,
        action: VerificationAction,
    ) -> Result<(), VerificationError> {
        self.unverified_session(token)?
            .verification_action(action)
            .await
    }

    pub async fn reset_identity(
        &self,
        token: &BridgeLinkToken,
        auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        self.unverified_session(token)?.reset_identity(auth).await
    }

    /// The account's login once it has authenticated.
    fn authenticated_login(&self, token: &BridgeLinkToken) -> Result<Login, VerificationError> {
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
        token: &BridgeLinkToken,
    ) -> Result<Arc<dyn BackendSession>, VerificationError> {
        match self.authenticated_login(token)? {
            Login::Ready { .. } => Err(VerificationError::AlreadyVerified),
            Login::Pending { session, .. } => Ok(session),
        }
    }
}
