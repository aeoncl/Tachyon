use crate::application::auth_use_case::AuthUseCase;
use crate::application::device_verification_use_case::DeviceVerificationUseCase;
use crate::application::logins::Logins;
use crate::application::ports::{AccountRepository, AuthService};
use crate::application::web_urls::WebUrls;
use std::sync::Arc;

/// Core's composition root: owns the live logins and hands bridges the use cases.
pub struct AppState {
    auth_use_case: Arc<AuthUseCase>,
    device_verification_use_case: Arc<DeviceVerificationUseCase>,
}

impl AppState {
    /// `web_base_url` is where the user's browser reaches the bridge's pages, for instance
    /// `http://127.0.0.1:11866/tachyon`.
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        account_repository: Arc<dyn AccountRepository>,
        web_base_url: String,
    ) -> AppState {
        let logins = Arc::new(Logins::default());

        AppState {
            auth_use_case: Arc::new(AuthUseCase::new(
                account_repository,
                logins.clone(),
                auth_service,
                WebUrls::new(web_base_url),
            )),
            device_verification_use_case: Arc::new(DeviceVerificationUseCase::new(logins)),
        }
    }

    pub fn auth_use_case(&self) -> &Arc<AuthUseCase> {
        &self.auth_use_case
    }

    pub fn device_verification_use_case(&self) -> &Arc<DeviceVerificationUseCase> {
        &self.device_verification_use_case
    }
}
