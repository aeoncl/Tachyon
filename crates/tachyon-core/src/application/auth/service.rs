use std::sync::Arc;

use crate::{
    application::{
        auth::dto::{LoginHints, LoginStart},
        error::AuthError,
    },
    domain::auth::{RestoreOutcome, TachyonToken},
    port::{
        backend::{BackendRestoreOutcome, ChatBackend},
        store::AccountStore,
    },
};

pub struct AuthService {
    store: Arc<dyn AccountStore>,
    backend: Arc<dyn ChatBackend>,
}

impl AuthService {
    pub async fn restore(&self, token: TachyonToken) -> Result<RestoreOutcome, AuthError> {
        let Some(login_id) = self.store.login_id_by_token(&token).await? else {
            return Err(AuthError::BackendCredentialsNotInStore);
        };

        let outcome = self.backend.restore_login(login_id).await?;

        Ok(outcome.to_auth_restore_outcome())
    }

    pub async fn begin_login(&self, hints: LoginHints) -> Result<LoginStart, AuthError> {
        todo!()
    }

    pub async fn end_login(&self) -> Result<TachyonToken, AuthError> {
        todo!()
    }
}

impl BackendRestoreOutcome {
    pub fn to_auth_restore_outcome(&self) -> RestoreOutcome {
        match self {
            BackendRestoreOutcome::Success(_) => RestoreOutcome::Success,
            BackendRestoreOutcome::SoftLoggedOut => RestoreOutcome::SoftLogout,
            BackendRestoreOutcome::LoggedOut => RestoreOutcome::Logout,
        }
    }
}
