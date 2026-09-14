use crate::application::error::{BackendError, StoreError, VerificationError};
use crate::domain::auth::{
    BridgeMetadata, Credential, CredentialBlob, InteractiveAuthStarted, BridgeLinkToken,
};
use crate::domain::ids::{DeviceId, LoginId, UserId};
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// One login's connection to a chat backend, from the first authorization redirect to the
/// last message sent. It outlives every step of the login flow; the `Step` of its `Login`
/// says which of its methods make sense right now.
#[async_trait]
pub trait BackendSession: Send + Sync {
    /// Completes the login this session was started for. Valid while the login is at
    /// `Step::Authenticate`; a rejected credential leaves it there.
    async fn authenticate(&self, credential: &Credential) -> Result<(), BackendError>;

    async fn device_status(&self) -> Result<DeviceStatus, BackendError>;


    async fn wait_until_verified(&self) -> Result<(), BackendError>;

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

    /// `close`, and remove whatever the backend keeps on disk for this login. For a login
    /// that will never be restored. The device stays on the backend; only `log_out` ends it.
    async fn discard(&self);

    async fn hard_log_out(&self) -> Result<(), BackendError>;

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

    /// Removes what the backend keeps on disk for a login that is not live. Nothing to do
    /// when there is nothing.
    async fn remove_login(&self, login_id: &LoginId) -> Result<(), BackendError>;

    /// Removes the on-disk state of every login not in `keep`.
    async fn clear_logins_except(&self, keep: &[LoginId]) -> Result<(), BackendError>;
}

/// A login row as the store holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredLogin {
    pub login_id: LoginId,
    /// Whether any token still points at it. One nobody points at is a leftover.
    pub linked: bool,
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn login_id_by_token(
        &self,
        tachyon_token: &BridgeLinkToken,
    ) -> Result<Option<LoginId>, StoreError>;

    async fn save_login_for_token(
        &self,
        tachyon_token: BridgeLinkToken,
        login_id: LoginId,
    ) -> Result<(), StoreError>;

    /// Removes the login and every token bound to it. Nothing to do when it is not there.
    async fn delete_login(&self, login_id: &LoginId) -> Result<(), StoreError>;

    async fn logins(&self) -> Result<Vec<StoredLogin>, StoreError>;
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn credentials(&self, login_id: &LoginId) -> Result<Option<CredentialBlob>, StoreError>;

    async fn store(&self, login_id: &LoginId, blob: CredentialBlob) -> Result<(), StoreError>;
}
