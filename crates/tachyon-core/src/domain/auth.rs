use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// The opaque, bridge-issued value a frontend client holds and echoes back to identify an
/// account. Maps to a [`crate::domain::ids::LoginId`] server-side; it is never a backend
/// credential and is never derived from one.
///
/// How it is minted is the bridge's business — core only ever compares and stores it.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct TachyonToken(Arc<str>);

impl TachyonToken {
    pub fn new(token: impl AsRef<str>) -> Self {
        Self(Arc::from(token.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Redacted: possession of a token is enough to restore a session, and request bodies are
/// logged at debug level.
impl Debug for TachyonToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TachyonToken(<redacted>)")
    }
}

pub enum RestoreOutcome {
    Success,
    SoftLogout,
    Logout,
}

/// What the backend needs the user's browser to do to complete a login.
pub enum InteractiveAuthStarted {
    /// Point the browser at `auth_url`. `csrf_token` is the OAuth `state` the redirect
    /// endpoint will receive back, so the bridge can correlate the callback with this login.
    OAuth {
        auth_url: String,
        csrf_token: String,
    },
    /// This backend has no interactive flow; the bridge must collect a password itself.
    PasswordRequired,
}

/// How the bridge introduces itself to a backend's authorization screen.
pub struct BridgeMetadata {
    pub name: String,
    pub client_uri: String,
    pub image_url: Option<String>,
    pub tos: Option<String>,
}
