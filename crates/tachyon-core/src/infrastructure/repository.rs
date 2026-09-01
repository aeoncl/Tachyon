use crate::application::ports::{BackendSession, SessionRepository};
use crate::domain::ids::LoginId;
use dashmap::DashMap;
use std::sync::Arc;

/// Sessions live only as long as the process — they hold open backend connections, so
/// there is nothing to persist. In-memory is this repository's production shape, not a
/// test double.
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
