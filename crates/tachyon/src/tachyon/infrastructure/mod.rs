use crate::tachyon::application::login::TachyonCredentialsRepository;
use crate::tachyon::infrastructure::credentials_repo::TachyonCredentialsRepositoryImpl;

mod credentials_repo;

pub(super) const TACHYON_CREDENTIALS_REPOSITORY: Box<dyn TachyonCredentialsRepository> = Box::new(TachyonCredentialsRepositoryImpl::default());