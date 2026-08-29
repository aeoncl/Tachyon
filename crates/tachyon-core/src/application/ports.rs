use std::sync::Arc;
use async_trait::async_trait;
use crate::application::error::{BackendError, StoreError};
use crate::domain::auth::TachyonToken;
use crate::domain::ids::LoginId;

pub trait BackendSession: Send + Sync {}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError>;
}

pub trait SessionRepository: Send + Sync {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession> );
}