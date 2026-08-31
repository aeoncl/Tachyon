use crate::application::error::{BackendError, StoreError};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// A live, authenticated connection to a chat backend.
///
/// The interface is deliberately thin for now: the legacy `tachyon` crate still drives the
/// backend directly, and this port only exists so core can own the session's lifetime.
pub trait BackendSession: Send + Sync {
    /// TEMPORARY (refactor scaffold).
    ///
    /// Lets the legacy `tachyon` crate downcast to the concrete adapter session
    /// (`BackendSessionMatrix`) and borrow the underlying `matrix_sdk::Client`, which the
    /// not-yet-migrated notification/switchboard/SOAP handlers still need.
    ///
    /// Delete this the moment `BackendSession` is deep enough to serve those handlers
    /// (messaging, presence, typing, media, conversation ops) — see
    /// `docs/architecture/tachyon-ports-adapters.md`, correction 4. Nothing outside the
    /// legacy crate may call it.
    fn as_any(&self) -> &dyn Any;
}

/// Authentication against a chat backend. Backends implement the steps; core owns the
/// choreography.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Rebuild a session from credentials the backend previously persisted for `login_id`.
    async fn restore_login(
        &self,
        login_id: LoginId,
    ) -> Result<Arc<dyn BackendSession>, BackendError>;

    /// Begin an interactive login. Returns what the user's browser must be pointed at, or
    /// [`InteractiveAuthStarted::PasswordRequired`] if this backend has no interactive flow.
    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: Option<UserId>,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<InteractiveAuthStarted, BackendError>;

    /// Complete an interactive login from what the browser brought back to the redirect
    /// endpoint. `callback_query` is the raw query string (`code=..&state=..`), not just
    /// the authorization code.
    async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query: &str,
    ) -> Result<Arc<dyn BackendSession>, BackendError>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError>;

    /// Bind a token to a login, so a later `restore` can find it.
    async fn link(
        &self,
        tachyon_token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), StoreError>;
}

pub trait SessionRepository: Send + Sync {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession>);

    fn get(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;

    fn remove(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;
}
