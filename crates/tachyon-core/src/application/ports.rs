use crate::application::error::{BackendError, ReadinessError, StoreError, VerificationError};
use crate::domain::auth::{
    BridgeMetadata, CredentialBlob, InteractiveAuthStarted, Readiness, TachyonToken,
};
use crate::domain::error::TachyonResult;
use crate::domain::events::BridgeEvent;
use crate::domain::ids::{DeviceId, LoginId, SessionId, UserId};
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// One login's connection to a chat backend, from the first authorization redirect to the
/// last message sent. It outlives every step of the login flow; `Readiness` in
/// `SessionRepository` says which of its methods make sense right now.
#[async_trait]
pub trait BackendSession: Send + Sync {
    /// `callback_query` is the raw query string the redirect endpoint received from the
    /// authorization server. Valid while the login is `Readiness::AuthNeeded`.
    async fn finish_interactive_login(&self, callback_query: &str) -> Result<(), BackendError>;

    async fn device_status(&self) -> Result<DeviceStatus, BackendError>;

    async fn verification_options(&self) -> Result<VerificationOptions, VerificationError>;

    /// Imports the cross-signing secrets guarded by the key and reports the device status
    /// afterwards.
    async fn recover(&self, key: &RecoveryKey) -> Result<DeviceStatus, VerificationError>;

    /// Asks another of the user's devices to verify this one with SAS emojis. Replaces and
    /// cancels the current flow, if any.
    async fn start_device_verification(&self, device: &DeviceId) -> Result<(), VerificationError>;

    /// `None` when no flow has been started on this session.
    fn verification_state(&self) -> Option<VerificationFlowState>;

    async fn verification_action(
        &self,
        action: VerificationAction,
    ) -> Result<(), VerificationError>;

    /// Replaces the user's cross-signing identity. Every other device becomes unverified.
    async fn reset_identity(
        &self,
        auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError>;

    /// Idempotent. Every later call on this session fails with
    /// `BackendError::Technical("session closed")`.
    async fn close(&self);

    /// FIXME: TEMPORARY, we won't expose the underlying client after the refactor is done
    fn as_any(&self) -> &dyn Any;
}

/// Builds a `BackendSession`, either from stored credentials or from a fresh sign-in.
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn restore(&self, login_id: &LoginId) -> Result<Arc<dyn BackendSession>, BackendError>;

    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: Option<UserId>,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError>;

    async fn save_login_for_token(
        &self,
        tachyon_token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), StoreError>;
}

#[derive(Clone)]
pub struct SessionEntry {
    pub session: Arc<dyn BackendSession>,
    pub readiness: Readiness,
}

pub trait SessionRepository: Send + Sync {
    fn insert(
        &self,
        login_id: LoginId,
        session: Arc<dyn BackendSession>,
        readiness: Readiness,
    ) -> Option<SessionEntry>;

    fn get(&self, login_id: &LoginId) -> Option<SessionEntry>;

    /// The only accessor bridges may use.
    fn get_ready(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;

    /// Forward transitions only (`Readiness::can_advance_to`); returns the previous readiness.
    fn set_readiness(
        &self,
        login_id: &LoginId,
        readiness: Readiness,
    ) -> Result<Readiness, ReadinessError>;

    fn remove(&self, login_id: &LoginId) -> Option<SessionEntry>;
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn credentials(&self, login_id: &LoginId) -> Result<Option<CredentialBlob>, StoreError>;

    async fn store(&self, login_id: &LoginId, blob: CredentialBlob) -> Result<(), StoreError>;
}

#[async_trait]
pub trait BridgeHandle: Send + Sync {
    async fn send(&self, event: BridgeEvent) -> TachyonResult<()>;
}

#[async_trait]
pub trait BridgeRepository: Send + Sync {
    async fn register_bridge(&self, session_id: SessionId, bridge: Arc<dyn BridgeHandle>);

    async fn bridge_by_id(&self, session_id: &SessionId) -> Option<Arc<dyn BridgeHandle>>;
}
