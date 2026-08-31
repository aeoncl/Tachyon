use crate::domain::auth::SessionRestoreData;
use async_trait::async_trait;
use dashmap::DashMap;
use tachyon_core::application::error::StoreError;
use tachyon_core::domain::ids::LoginId;

#[async_trait]
pub trait CredentialsRepository: Send + Sync {
    async fn session_restore_data_by_login_id(
        &self,
        login_id: &LoginId,
    ) -> Result<Option<SessionRestoreData>, StoreError>;

    async fn insert(
        &self,
        login_id: &LoginId,
        session_restore_data: SessionRestoreData,
    ) -> Result<(), StoreError>;

    async fn update_tokens(
        &self,
        login_id: &LoginId,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Result<(), StoreError>;
}

/// TEMPORARY: credentials are lost on restart, so every restart re-runs the interactive
/// login. Replaced by `CredentialRepositorySqlite` — see the architecture doc, correction 7.
#[derive(Default)]
pub struct CredentialsRepositoryInMem {
    credentials: DashMap<LoginId, SessionRestoreData>,
}

#[async_trait]
impl CredentialsRepository for CredentialsRepositoryInMem {
    async fn session_restore_data_by_login_id(
        &self,
        login_id: &LoginId,
    ) -> Result<Option<SessionRestoreData>, StoreError> {
        Ok(self.credentials.get(login_id).map(|entry| entry.value().clone()))
    }

    async fn insert(
        &self,
        login_id: &LoginId,
        session_restore_data: SessionRestoreData,
    ) -> Result<(), StoreError> {
        self.credentials.insert(login_id.clone(), session_restore_data);
        Ok(())
    }

    async fn update_tokens(
        &self,
        login_id: &LoginId,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Result<(), StoreError> {
        if let Some(mut entry) = self.credentials.get_mut(login_id) {
            entry.access_token = access_token;
            entry.refresh_token = refresh_token;
        }
        Ok(())
    }
}

pub struct CredentialRepositorySqlite;

#[async_trait]
impl CredentialsRepository for CredentialRepositorySqlite {
    async fn session_restore_data_by_login_id(
        &self,
        login_id: &LoginId,
    ) -> Result<Option<SessionRestoreData>, StoreError> {
        todo!("Fetch from DB for {:?}", login_id)
    }

    async fn insert(
        &self,
        _login_id: &LoginId,
        _session_restore_data: SessionRestoreData,
    ) -> Result<(), StoreError> {
        todo!("insert credentials")
    }

    async fn update_tokens(
        &self,
        _login_id: &LoginId,
        _access_token: String,
        _refresh_token: Option<String>,
    ) -> Result<(), StoreError> {
        todo!("update tokens")
    }
}
