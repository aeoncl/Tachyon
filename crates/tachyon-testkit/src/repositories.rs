use async_trait::async_trait;
use dashmap::{DashMap, DashSet};
use std::sync::Arc;
use tachyon_core::application::error::StoreError;
use tachyon_core::application::ports::{AccountRepository, CredentialRepository, StoredLogin};
use tachyon_core::domain::auth::{CredentialBlob, BridgeLinkToken};
use tachyon_core::domain::ids::LoginId;

/// Logins and the tokens bound to them, the way the sqlite store keeps them: a login can
/// exist with no token pointing at it.
#[derive(Default)]
pub struct AccountRepositoryInMem {
    logins: DashSet<LoginId>,
    tokens: DashMap<BridgeLinkToken, LoginId>,
}

impl AccountRepositoryInMem {
    /// A store that already knows one account.
    pub fn with_login(token: &BridgeLinkToken, login_id: &LoginId) -> Arc<Self> {
        let repository = Self::default();
        repository.logins.insert(login_id.clone());
        repository.tokens.insert(token.clone(), login_id.clone());
        Arc::new(repository)
    }

    /// A login no token points at, the leftover of an interrupted sign-in.
    pub fn with_unbound_login(self: Arc<Self>, login_id: &LoginId) -> Arc<Self> {
        self.logins.insert(login_id.clone());
        self
    }
}

#[async_trait]
impl AccountRepository for AccountRepositoryInMem {
    async fn login_id_by_token(
        &self,
        tachyon_token: &BridgeLinkToken,
    ) -> Result<Option<LoginId>, StoreError> {
        Ok(self.tokens.get(tachyon_token).map(|entry| entry.value().clone()))
    }

    async fn save_login_for_token(
        &self,
        tachyon_token: BridgeLinkToken,
        login_id: LoginId,
    ) -> Result<(), StoreError> {
        self.logins.insert(login_id.clone());
        self.tokens.insert(tachyon_token, login_id);
        Ok(())
    }

    async fn delete_login(&self, login_id: &LoginId) -> Result<(), StoreError> {
        self.tokens.retain(|_, bound| bound != login_id);
        self.logins.remove(login_id);
        Ok(())
    }

    async fn logins(&self) -> Result<Vec<StoredLogin>, StoreError> {
        Ok(self
            .logins
            .iter()
            .map(|login_id| StoredLogin {
                linked: self.tokens.iter().any(|entry| entry.value() == &*login_id),
                login_id: login_id.clone(),
            })
            .collect())
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
