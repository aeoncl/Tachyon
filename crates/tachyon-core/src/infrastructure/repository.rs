use crate::application::ports::{BackendSession, BridgeRepository, SessionRepository};
use crate::domain::ids::{LoginId, SessionId};
use dashmap::DashMap;
use std::sync::Arc;
use crate::domain::bridge::BridgeHandle;

/// Sessions live only as long as the process — they hold open backend connections, so
/// there is nothing to persist. In-memory is this repository's production shape, not a
/// test double.
#[derive(Default)]
pub(crate) struct SessionRepositoryInMem {
    sessions: DashMap<LoginId, Arc<dyn BackendSession>>,
}

impl SessionRepository for SessionRepositoryInMem {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession>) -> Option<Arc<dyn BackendSession>> {
        self.sessions.insert(login_id, session)
    }

    fn get(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>> {
        self.sessions.get(login_id).map(|entry| entry.value().clone())
    }

    fn remove(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>> {
        self.sessions.remove(login_id).map(|(_, session)| session)
    }
}


pub(crate) struct BridgeRepositoryInMem {
    bridges: DashMap<SessionId, Arc<dyn BridgeHandle>>

}
impl BridgeRepository for BridgeRepositoryInMem {
    async fn register_bridge(&self, session_id: SessionId, bridge: Arc<dyn BridgeHandle>) {
        self.bridges.insert(session_id, bridge);
    }

    async fn bridge_by_id(&self, session_id: &SessionId) -> Option<Arc<dyn BridgeHandle>> {
        self.bridges.get(session_id).map(|e| e.to_owned())
    }
}