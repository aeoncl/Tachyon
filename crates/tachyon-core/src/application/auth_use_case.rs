use crate::application::error::AuthError;
use crate::application::ports::{AccountRepository, SessionRepository};
use crate::domain::auth::TachyonToken;
use crate::domain::backend_ports::AuthService;
use crate::domain::ids::{LoginId, UserId};
use std::sync::Arc;
use uuid::Uuid;


pub struct LoginStart {
    login_id: LoginId,
    auth_url: String,
    csrf_token: String,
}


pub struct BridgeMetadata {
    name: String,
    image_url: Option<String>,
    tos: Option<String>
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    session_repository: Arc<dyn SessionRepository>
}

impl AuthUseCase {
    pub fn new(account_repository: Arc<dyn AccountRepository>, session_repository: Arc<dyn SessionRepository>, auth_service: Arc<dyn AuthService>) -> AuthUseCase {
        AuthUseCase {
            account_repository,
            auth_service,
            session_repository,
        }
    }
}

impl AuthUseCase {

    pub async fn restore(&self, token: TachyonToken) -> Result<LoginId, AuthError> {

        let Some(login_id) = self.account_repository.login_id_by_token(&token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        let backend_session = self.auth_service.restore_login(login_id.clone()).await?;

        self.session_repository.insert(login_id.clone(), backend_session);

        Ok(login_id)
    }

    pub async fn start_interactive_login(&self, server_name: &str, user_id: UserId, bridge_metadata: BridgeMetadata) -> Result<LoginStart, AuthError> {
        let login_id = LoginId::new(Uuid::new_v4().to_string());
        //TODO build that with config & url builder service
        let redirect_url = "http://127.0.0.1:{{webport}}/tachyon/login";
        let login_metadata = self.auth_service.start_interactive_login(&login_id, server_name, Some(user_id), redirect_url).await?;

        Ok(LoginStart {
            login_id,
            auth_url: login_metadata.auth_url,
            csrf_token: login_metadata.csrf_token,
        })
    }

    pub async fn finish_interactive_login(&self, login_id: &LoginId, code: &str) -> Result<LoginId, AuthError> {
        let backend_session = self.auth_service.finish_interactive_login(login_id, code).await?;

        self.session_repository.insert(login_id.clone(), backend_session);

        Ok(login_id.clone())
    }

}
