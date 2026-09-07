use crate::application::error::{BackendError, StoreError, VerificationError};
use crate::domain::auth::{BridgeMetadata, CredentialBlob, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{DeviceId, LoginId, SessionId, UserId, VerificationFlowId};
use crate::domain::error::TachyonResult;
use crate::domain::events::BridgeEvent;
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// A live, authenticated connection to a chat backend.
pub trait BackendSession: Send + Sync {
    /// FIXME: TEMPORARY, we won't expose the underlying client after the refactor is done
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait AuthService: Send + Sync {

    async fn restore_session(
        &self,
        login_id: LoginId,
    ) -> Result<Arc<dyn BackendSession>, BackendError>;


    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: Option<UserId>,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<InteractiveAuthStarted, BackendError>;

    async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query_params: &str,
    ) -> Result<Arc<dyn BackendSession>, BackendError>;
}

#[async_trait]
pub trait VerificationService: Send + Sync {
    async fn device_status(&self, login_id: &LoginId) -> Result<DeviceStatus, VerificationError>;

    async fn verification_options(&self, login_id: &LoginId) -> Result<VerificationOptions, VerificationError>;

    /// Imports the cross-signing secrets guarded by the key and returns the device status afterwards.
    async fn recover(&self, login_id: &LoginId, recovery_key: &RecoveryKey) -> Result<DeviceStatus, VerificationError>;

    /// Asks one of the user's other devices to verify this one with SAS emojis.
    async fn start_device_verification(&self, login_id: &LoginId, device_id: &DeviceId) -> Result<VerificationFlowId, VerificationError>;

    async fn verification_state(&self, login_id: &LoginId, flow_id: &VerificationFlowId) -> Result<VerificationFlowState, VerificationError>;

    async fn verification_action(&self, login_id: &LoginId, flow_id: &VerificationFlowId, action: VerificationAction) -> Result<(), VerificationError>;

    /// Replaces the user's cross-signing identity. Every other device becomes unverified.
    async fn reset_identity(&self, login_id: &LoginId, password: Option<&str>) -> Result<IdentityReset, VerificationError>;
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

pub trait SessionRepository: Send + Sync {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession>) -> Option<Arc<dyn BackendSession>>;

    fn get(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;

    fn remove(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;
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
