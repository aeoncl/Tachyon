use crate::application::auth_use_case::AuthUseCase;
use crate::application::ports::{AuthService, SessionRepository};
use crate::infrastructure::repository::{AccountRepositoryInMem, SessionRepositoryInMem};
use std::sync::Arc;

/// Core's composition root: owns the repositories and hands bridges the use cases.
pub struct AppState {
    session_repository: Arc<dyn SessionRepository>,
    auth_use_case: Arc<AuthUseCase>,
}

impl AppState {
    /// `redirect_url` is the bridge endpoint a backend sends the user's browser back to
    /// once they have authorized.
    pub fn new(auth_service: Arc<dyn AuthService>, redirect_url: String) -> AppState {
        let session_repository = Arc::new(SessionRepositoryInMem::default());
        let account_repository = Arc::new(AccountRepositoryInMem::default());

        let auth_use_case = Arc::new(AuthUseCase::new(
            account_repository.clone(),
            session_repository.clone(),
            auth_service.clone(),
            redirect_url,
        ));

        AppState {
            session_repository,
            auth_use_case,
        }
    }

    pub fn auth_use_case(&self) -> &Arc<AuthUseCase> {
        &self.auth_use_case
    }

    pub fn session_repository(&self) -> &Arc<dyn SessionRepository> {
        &self.session_repository
    }
}
