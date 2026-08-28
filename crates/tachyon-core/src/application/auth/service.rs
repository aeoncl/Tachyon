use std::sync::Arc;

use crate::application::{
    auth::dto::{LoginHints, LoginStart},
    error::AuthError,
};
use crate::application::ports::ChatBackendTrait;
use crate::domain::auth::TachyonToken;
use crate::application::ports::SessionRepositoryTrait;
use crate::application::ports::AccountRepositoryTrait;
use crate::ids::LoginId;

pub struct AuthService {
    store: Arc<dyn AccountRepositoryTrait>,
    backend: Arc<dyn ChatBackendTrait>,
    session_repository: Arc<dyn SessionRepositoryTrait>
}

impl AuthService {
    pub async fn restore(&self, token: TachyonToken) -> Result<LoginId, AuthError> {
        let Some(login_id) = self.store.login_id_by_token(&token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        let backend_session = self.backend.restore_login(login_id.clone()).await?;

        self.session_repository.insert(login_id.clone(), backend_session);


        Ok(login_id)
    }

    pub async fn begin_login(&self, hints: LoginHints) -> Result<LoginStart, AuthError> {
        todo!()
    }

    pub async fn end_login(&self) -> Result<TachyonToken, AuthError> {
        todo!()
    }
}
