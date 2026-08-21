use crate::matrix::application::service_locator::MATRIX_LOGIN_SERVICE;
use crate::tachyon::application::login::{TachyonCredentialsRepository, TachyonLoginService, TachyonLoginServiceImpl};

pub const TACHYON_CREDENTIALS_REPOSITORY: Box<dyn TachyonCredentialsRepository> = super::super::infrastructure::TACHYON_CREDENTIALS_REPOSITORY;
pub const TACHYON_LOGIN_SERVICE: Box<dyn TachyonLoginService> = Box::new(TachyonLoginServiceImpl::new(TACHYON_CREDENTIALS_REPOSITORY, MATRIX_LOGIN_SERVICE));