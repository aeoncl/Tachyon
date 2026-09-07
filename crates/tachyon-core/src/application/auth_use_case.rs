use crate::application::error::AuthError;
use crate::application::ports::{AccountRepository, AuthService, BackendSession, SessionRepository};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, Readiness, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use crate::domain::verification::DeviceStatus;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct LoginStart {
    pub login_id: LoginId,
    pub prompt: InteractiveAuthStarted,
}

pub enum LoginOutcome {
    SessionOpened {
        login_id: LoginId,
        session: Arc<dyn BackendSession>,
    },
    DeviceVerificationRequired {
        login_id: LoginId,
    },
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    session_repository: Arc<dyn SessionRepository>,
    /// Serializes everything that creates, settles or drops a login, so two callers
    /// cannot each build a backend session for the same login id.
    lifecycle: Mutex<()>,
    /// TODO: Move this configuration towards the bridges
    redirect_url: String,
}

impl AuthUseCase {
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        session_repository: Arc<dyn SessionRepository>,
        auth_service: Arc<dyn AuthService>,
        redirect_url: String,
    ) -> AuthUseCase {
        AuthUseCase {
            account_repository,
            auth_service,
            session_repository,
            lifecycle: Mutex::new(()),
            redirect_url,
        }
    }
}

impl AuthUseCase {
    pub async fn start_interactive_login(
        &self,
        server_name: &str,
        user_id: UserId,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<LoginStart, AuthError> {
        let login_id = LoginId::new(Uuid::new_v4().to_string());

        let (session, prompt) = self
            .auth_service
            .start_interactive_login(
                &login_id,
                server_name,
                Some(user_id),
                &self.redirect_url,
                bridge_metadata,
            )
            .await?;

        self.session_repository
            .insert(login_id.clone(), session, Readiness::AuthNeeded);

        Ok(LoginStart { login_id, prompt })
    }

    /// `callback_query_params` is the raw query params string the redirect endpoint received
    /// from the authorization server.
    pub async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query_params: &str,
    ) -> Result<LoginOutcome, AuthError> {
        let Some(entry) = self.session_repository.get(login_id) else {
            return Err(AuthError::LoginNotFound);
        };
        if entry.readiness != Readiness::AuthNeeded {
            return Err(AuthError::LoginNotFound);
        }

        entry
            .session
            .finish_interactive_login(callback_query_params)
            .await?;

        let _guard = self.lifecycle.lock().await;
        self.settle(login_id, entry.session).await
    }

    pub async fn restore(&self, token: &TachyonToken) -> Result<LoginOutcome, AuthError> {
        let _guard = self.lifecycle.lock().await;
        self.restore_under_lifecycle(token).await
    }

    /// `Ok(())` when there was nothing to abandon.
    pub async fn abandon_login(&self, login_id: &LoginId) -> Result<(), AuthError> {
        let _guard = self.lifecycle.lock().await;

        if let Some(entry) = self.session_repository.remove(login_id) {
            entry.session.close().await;
        }
        Ok(())
    }

    pub async fn bind_token(
        &self,
        token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), AuthError> {
        self.account_repository
            .save_login_for_token(token, login_id)
            .await?;
        Ok(())
    }

    async fn restore_under_lifecycle(
        &self,
        token: &TachyonToken,
    ) -> Result<LoginOutcome, AuthError> {
        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        let Some(entry) = self.session_repository.get(&login_id) else {
            let session = self.auth_service.restore(&login_id).await?;
            self.session_repository
                .insert(login_id.clone(), session.clone(), Readiness::AuthNeeded);
            return self.settle(&login_id, session).await;
        };

        match entry.readiness {
            Readiness::Ready => Ok(LoginOutcome::SessionOpened {
                login_id,
                session: entry.session,
            }),
            Readiness::VerificationNeeded => self.settle(&login_id, entry.session).await,
            // Unreachable: the token to login row is only written by `bind_token`, after the
            // authorization server has already called back.
            Readiness::AuthNeeded => Err(AuthError::LoginNotFound),
        }
    }

    /// The only writer of `Readiness`. The caller holds the lifecycle guard.
    async fn settle(
        &self,
        login_id: &LoginId,
        session: Arc<dyn BackendSession>,
    ) -> Result<LoginOutcome, AuthError> {
        let readiness = match session.device_status().await? {
            DeviceStatus::Verified => Readiness::Ready,
            DeviceStatus::Unverified => Readiness::VerificationNeeded,
        };
        self.session_repository.set_readiness(login_id, readiness)?;

        let login_id = login_id.clone();
        Ok(match readiness {
            Readiness::Ready => LoginOutcome::SessionOpened { login_id, session },
            _ => LoginOutcome::DeviceVerificationRequired { login_id },
        })
    }
}
