use crate::application::error::AuthError;
use crate::application::ports::{AccountRepository, AuthService, BackendSession, SessionRepository};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use std::sync::Arc;
use uuid::Uuid;

pub struct LoginStart {
    pub login_id: LoginId,
    pub prompt: InteractiveAuthStarted,
}

pub struct RestoredLogin {
    pub login_id: LoginId,
    pub session: Arc<dyn BackendSession>,
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    session_repository: Arc<dyn SessionRepository>,
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
            redirect_url,
        }
    }
}

impl AuthUseCase {
    pub async fn restore_session(&self, token: &TachyonToken) -> Result<RestoredLogin, AuthError> {
        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        //FIXME: In which circonstances can this happen ? if a client retriggers a restore session, we should get rid of a remaining one, not giving it back, we should also ensure the session is properly deleted on logout of the bridge.
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

    /// `callback_query_params` is the raw query params string the redirect endpoint received from the authorization server.
    pub async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query_params: &str,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        let session = self
            .auth_service
            .finish_interactive_login(login_id, callback_query_params)
            .await?;

        self.session_repository
            .insert(login_id.clone(), session.clone());

        Ok(session)
    }

    pub async fn bind_token(
        &self,
        token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), AuthError> {
        self.account_repository.save_login_for_token(token, login_id).await?;
        Ok(())
    }
}
