use crate::application::error::BackendError;
use crate::application::ports::BackendSession;
use crate::domain::auth::InteractiveAuthStarted;
use crate::domain::ids::{LoginId, UserId};
use async_trait::async_trait;
use std::sync::Arc;


#[async_trait]
    pub trait AuthService {
        async fn restore_login(&self, login_id: LoginId)
                               -> Result<Arc<dyn BackendSession>, BackendError>;

        async fn start_interactive_login(&self, login_id: &LoginId, server_name: &str, user_id: Option<UserId>, redirect_url: &str) -> Result<InteractiveAuthStarted, BackendError>;
        async fn finish_interactive_login(&self, login_id: &LoginId, code: &str) -> Result<Arc<dyn BackendSession>, BackendError>;

    }
