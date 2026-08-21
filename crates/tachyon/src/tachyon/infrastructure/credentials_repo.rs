use anyhow::Error;
use dashmap::DashMap;
use crate::tachyon::application::login::{MatrixCredentials, TachyonCredentialsRepository, TachyonToken};


//TODO store this in a persisted db
pub struct TachyonCredentialsRepositoryImpl {
    repository: DashMap<TachyonToken, MatrixCredentials>
}

impl Default for TachyonCredentialsRepositoryImpl {
    fn default() -> Self {
        Self {
            repository: Default::default(),
        }
    }
}

impl TachyonCredentialsRepository for TachyonCredentialsRepositoryImpl {
    fn get_credentials(&self, tachyon_token: &TachyonToken) -> Result<Option<MatrixCredentials>, Error> {
        Ok(self.repository.get(tachyon_token).map(|e| e.clone()))
    }

    fn update_credentials(&self, tachyon_token: TachyonToken, matrix_credentials: MatrixCredentials) -> Result<(), Error> {

        let _ = self.repository.insert(tachyon_token, matrix_credentials);
        Ok(())
    }

    fn create_credentials(&self, tachyon_token: TachyonToken, matrix_credentials: MatrixCredentials) -> Result<(), Error> {
        let _ = self.repository.insert(tachyon_token, matrix_credentials);
        Ok(())
    }

    fn remove_credentials(&self, tachyon_token: &TachyonToken) -> Result<(), Error> {
        self.repository.remove(tachyon_token);
        Ok(())
    }
}