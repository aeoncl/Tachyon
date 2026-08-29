use std::sync::Arc;
use crate::application::auth_use_case::AuthUseCase;
use crate::application::ports::{AccountRepository, SessionRepository};
use crate::domain::backend_ports::{AuthService};
use crate::infrastructure::repository::{AccountRepositoryyInMem, SessionRepositoryInMem};

pub struct AppState {

    session_repository: Arc<dyn SessionRepository>,
    account_repository: Arc<dyn AccountRepository>,

    auth_service: Arc<dyn AuthService>,

    auth_use_case: Arc<AuthUseCase>

}

pub fn new(auth_service: Arc<dyn AuthService>) -> AppState {

    let session_repository = Arc::new(SessionRepositoryInMem::default());
    let account_repository = Arc::new(AccountRepositoryyInMem::default());

    let auth_use_case = Arc::new(AuthUseCase::new(account_repository.clone(), session_repository.clone(), auth_service.clone()));
    
    AppState {
        session_repository,
        account_repository,
        auth_service,
        auth_use_case,
    }
}