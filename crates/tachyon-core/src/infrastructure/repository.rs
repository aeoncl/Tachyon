use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use crate::application::error::StoreError;
use crate::application::ports::{AccountRepository, BackendSession, SessionRepository};
use crate::domain::auth::TachyonToken;
use crate::domain::ids::LoginId;


#[derive(Default)]
pub(crate) struct SessionRepositoryInMem {
    sessions: DashMap<LoginId, Arc<dyn BackendSession>>
}


impl SessionRepository for SessionRepositoryInMem {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession> ) {
        self.sessions.insert(login_id, session);
    }
}

pub(crate) struct AccountRepositoryyInMem {
    logins: DashMap<TachyonToken, LoginId>
}

impl Default for AccountRepositoryyInMem {
    fn default() -> Self {
        Self {
            logins: DashMap::new()
        }
    }
}

#[async_trait]
impl AccountRepository for AccountRepositoryyInMem {
    async fn login_id_by_token(&self, tachyon_token: &TachyonToken) -> Result<Option<LoginId>, StoreError> {
todo!()

    }
}

