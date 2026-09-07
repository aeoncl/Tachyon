use async_trait::async_trait;
use tachyon_core::application::error::VerificationError;
use tachyon_core::application::ports::VerificationService;
use tachyon_core::domain::ids::{DeviceId, LoginId, VerificationFlowId};
use tachyon_core::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, VerificationAction, VerificationFlowState,
    VerificationOptions,
};

pub struct VerificationServiceMatrixSdk {}

#[async_trait]
impl VerificationService for VerificationServiceMatrixSdk {
    async fn device_status(&self, _login_id: &LoginId) -> Result<DeviceStatus, VerificationError> {
        todo!()
    }

    async fn verification_options(&self, _login_id: &LoginId) -> Result<VerificationOptions, VerificationError> {
        todo!()
    }

    async fn recover(&self, _login_id: &LoginId, _recovery_key: &RecoveryKey) -> Result<DeviceStatus, VerificationError> {
        todo!()
    }

    async fn start_device_verification(&self, _login_id: &LoginId, _device_id: &DeviceId) -> Result<VerificationFlowId, VerificationError> {
        todo!()
    }

    async fn verification_state(&self, _login_id: &LoginId, _flow_id: &VerificationFlowId) -> Result<VerificationFlowState, VerificationError> {
        todo!()
    }

    async fn verification_action(&self, _login_id: &LoginId, _flow_id: &VerificationFlowId, _action: VerificationAction) -> Result<(), VerificationError> {
        todo!()
    }

    async fn reset_identity(&self, _login_id: &LoginId, _password: Option<&str>) -> Result<IdentityReset, VerificationError> {
        todo!()
    }
}
