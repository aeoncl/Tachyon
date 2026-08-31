use crate::application::error::AuthError;
use crate::application::ports::{AccountRepository, AuthService, BackendSession, SessionRepository};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use std::sync::Arc;
use uuid::Uuid;

/// A login that has been started but not yet completed by the user.
pub struct LoginStart {
    pub login_id: LoginId,
    pub prompt: InteractiveAuthStarted,
}

/// A session that has been handed back to a bridge, with the login it belongs to.
pub struct RestoredLogin {
    pub login_id: LoginId,
    pub session: Arc<dyn BackendSession>,
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    session_repository: Arc<dyn SessionRepository>,
    /// Where the backend sends the user's browser once they have authorized. Owned by core
    /// so the URL is built from configuration rather than by each adapter.
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
            redirect_url,
        }
    }
}

impl AuthUseCase {
    /// Resolve a token to a live session, reusing the one already open for that login if
    /// there is one, and otherwise rebuilding it from stored credentials.
    ///
    /// Returns [`AuthError::BackendCredentialsNotInStore`] when the token has never been
    /// linked to a login — the bridge's cue to start an interactive login.
    pub async fn restore(&self, token: &TachyonToken) -> Result<RestoredLogin, AuthError> {
        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        if let Some(session) = self.session_repository.get(&login_id) {
            return Ok(RestoredLogin { login_id, session });
        }

        let session = self.auth_service.restore_login(login_id.clone()).await?;
        self.session_repository.insert(login_id.clone(), session.clone());

        Ok(RestoredLogin { login_id, session })
    }

    pub async fn start_interactive_login(
        &self,
        server_name: &str,
        user_id: UserId,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<LoginStart, AuthError> {
        let login_id = LoginId::new(Uuid::new_v4().to_string());

        let prompt = self
            .auth_service
            .start_interactive_login(
                &login_id,
                server_name,
                Some(user_id),
                &self.redirect_url,
                bridge_metadata,
            )
            .await?;

        Ok(LoginStart { login_id, prompt })
    }

    /// `callback_query` is the raw query string the redirect endpoint received.
    pub async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query: &str,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        let session = self
            .auth_service
            .finish_interactive_login(login_id, callback_query)
            .await?;

        self.session_repository
            .insert(login_id.clone(), session.clone());

        Ok(session)
    }

    /// Bind the frontend's token to a completed login, so the next `restore` finds it.
    pub async fn link_token(
        &self,
        token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), AuthError> {
        self.account_repository.link(token, login_id).await?;
        Ok(())
    }
}
