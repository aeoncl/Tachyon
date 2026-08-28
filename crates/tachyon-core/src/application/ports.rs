use std::sync::Arc;
use async_trait::async_trait;
use crate::application::error::{BackendError, StoreError};
use crate::domain::auth::TachyonToken;
use crate::ids::LoginId;
#[async_trait]
pub trait ChatBackendTrait: Send + Sync {
    async fn restore_login(&self, login_id: LoginId)
    -> Result<Arc<dyn BackendSession>, BackendError>;
}

pub trait BackendSession: Send + Sync {}

#[async_trait]
pub trait AccountRepositoryTrait: Send + Sync {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError>;
}

pub trait SessionRepositoryTrait: Send + Sync {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession> );
}