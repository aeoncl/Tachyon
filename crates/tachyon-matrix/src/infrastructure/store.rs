use async_trait::async_trait;
use tachyon_core::ids::LoginId;
use tachyon_core::application::error::StoreError;
use crate::domain::auth::SessionRestoreData;

pub struct CredentialStoreSqlite;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn session_restore_data_by_login_id(
        &self,
        login_id: &LoginId,
    ) -> Result<Option<SessionRestoreData>, StoreError>;

    async fn update_tokens(&self, login_id: &LoginId, access_token: String, refresh_token: Option<String>) -> Result<(), StoreError>;

}
#[async_trait]
impl CredentialStore for CredentialStoreSqlite {
    async fn session_restore_data_by_login_id(
        &self,
        login_id: &LoginId,
    ) -> Result<Option<SessionRestoreData>, StoreError> {
        todo!("Fetch from DB for {:?}", login_id)
    }

    async fn update_tokens(&self, login_id: &LoginId, access_token: String, refresh_token: Option<String>) -> Result<(), StoreError> {
        todo!("update tokens")
    }

}
