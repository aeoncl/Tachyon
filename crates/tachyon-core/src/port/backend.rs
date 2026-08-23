use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::{auth::RestoreOutcome, ids::LoginId},
    port::error::BackendError,
};

#[async_trait]
pub trait ChatBackend: Send + Sync {
    async fn restore_login(&self, login_id: LoginId)
    -> Result<BackendRestoreOutcome, BackendError>;
}

pub trait BackendSession {}

pub enum BackendRestoreOutcome {
    Success(Arc<dyn BackendSession>),
    SoftLoggedOut,
    LoggedOut,
}
