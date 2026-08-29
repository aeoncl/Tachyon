use std::sync::Arc;
use crate::application::ports::BackendSession;

#[derive(Hash, Eq, PartialEq)]
pub struct TachyonToken(String);

pub enum RestoreOutcome {
    Success,
    SoftLogout,
    Logout,
}

pub struct InteractiveAuthStarted {
    pub backend_session: Arc<dyn BackendSession>,
    pub auth_url: String,
    pub csrf_token: String
}