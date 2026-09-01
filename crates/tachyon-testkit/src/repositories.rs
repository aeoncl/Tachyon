use async_trait::async_trait;
use dashmap::DashMap;
use tachyon_core::application::error::StoreError;
use tachyon_core::application::ports::{AccountRepository, CredentialRepository};
use tachyon_core::domain::auth::{CredentialBlob, TachyonToken};
use tachyon_core::domain::ids::LoginId;

#[derive(Default)]
pub struct AccountRepositoryInMem {
    logins: DashMap<TachyonToken, LoginId>,
}

#[async_trait]
impl AccountRepository for AccountRepositoryInMem {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError> {
        Ok(self.logins.get(tachyon_token).map(|entry| entry.value().clone()))
    }

    async fn save_login_for_token(&self, tachyon_token: TachyonToken, login_id: LoginId) -> Result<(), StoreError> {
        self.logins.insert(tachyon_token, login_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct CredentialRepositoryInMem {
    credentials: DashMap<LoginId, CredentialBlob>,
}

#[async_trait]
impl CredentialRepository for CredentialRepositoryInMem {
    async fn credentials(&self, login_id: &LoginId) -> Result<Option<CredentialBlob>, StoreError> {
        Ok(self.credentials.get(login_id).map(|entry| entry.value().clone()))
    }

    async fn store(&self, login_id: &LoginId, blob: CredentialBlob) -> Result<(), StoreError> {
        self.credentials.insert(login_id.clone(), blob);
        Ok(())
    }
}
