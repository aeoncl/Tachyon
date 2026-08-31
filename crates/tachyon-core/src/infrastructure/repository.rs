use crate::application::error::StoreError;
use crate::application::ports::{AccountRepository, BackendSession, SessionRepository};
use crate::domain::auth::TachyonToken;
use crate::domain::ids::LoginId;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Sessions live only as long as the process — they hold open backend connections, so
/// there is nothing to persist.
#[derive(Default)]
pub(crate) struct SessionRepositoryInMem {
    sessions: DashMap<LoginId, Arc<dyn BackendSession>>,
}

impl SessionRepository for SessionRepositoryInMem {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession>) {
        self.sessions.insert(login_id, session);
    }

    fn get(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>> {
        self.sessions.get(login_id).map(|entry| entry.value().clone())
    }

    fn remove(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>> {
        self.sessions.remove(login_id).map(|(_, session)| session)
    }
}

/// TEMPORARY: token bindings are lost on restart, so every restart re-runs the interactive
/// login. Replaced by a `tachyon-store` table — see the architecture doc, correction 7.
#[derive(Default)]
pub(crate) struct AccountRepositoryInMem {
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

    async fn link(
        &self,
        tachyon_token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), StoreError> {
        self.logins.insert(tachyon_token, login_id);
        Ok(())
    }
}
