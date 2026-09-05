use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::VerificationService;
use tachyon_core::domain::ids::LoginId;

pub struct VerificationServiceMatrixSdk {

}

impl VerificationService for VerificationServiceMatrixSdk {
    async fn start_device_verification(&self, login_id: &LoginId) -> Result<(), BackendError> {
        let _ = login_id;
        Ok(())

    }

    async fn complete_device_verification(&self, login_id: &LoginId, code: &str) -> Result<(), BackendError> {
        todo!()
    }
}
