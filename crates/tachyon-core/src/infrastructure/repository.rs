use crate::application::error::ReadinessError;
use crate::application::ports::{
    BackendSession, BridgeHandle, BridgeRepository, SessionEntry, SessionRepository,
};
use crate::domain::auth::Readiness;
use crate::domain::ids::{LoginId, SessionId};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Sessions live only as long as the process — they hold open backend connections, so
/// there is nothing to persist. In-memory is this repository's production shape, not a
/// test double.
#[derive(Default)]
pub(crate) struct SessionRepositoryInMem {
    sessions: DashMap<LoginId, SessionEntry>,
}

impl SessionRepository for SessionRepositoryInMem {
    fn insert(
        &self,
        login_id: LoginId,
        session: Arc<dyn BackendSession>,
        readiness: Readiness,
    ) -> Option<SessionEntry> {
        self.sessions
            .insert(login_id, SessionEntry { session, readiness })
    }

    fn get(&self, login_id: &LoginId) -> Option<SessionEntry> {
        self.sessions
            .get(login_id)
            .map(|entry| entry.value().clone())
    }

    fn get_ready(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>> {
        self.sessions.get(login_id).and_then(|entry| {
            matches!(entry.value().readiness, Readiness::Ready)
                .then(|| entry.value().session.clone())
        })
    }

    fn set_readiness(
        &self,
        login_id: &LoginId,
        readiness: Readiness,
    ) -> Result<Readiness, ReadinessError> {
        let mut entry = self
            .sessions
            .get_mut(login_id)
            .ok_or(ReadinessError::NotFound)?;

        let previous = entry.readiness;
        if !previous.can_advance_to(readiness) {
            return Err(ReadinessError::Backwards {
                from: previous,
                to: readiness,
            });
        }
        entry.readiness = readiness;
        Ok(previous)
    }

    fn remove(&self, login_id: &LoginId) -> Option<SessionEntry> {
        self.sessions.remove(login_id).map(|(_, entry)| entry)
    }
}

pub(crate) struct BridgeRepositoryInMem {
    bridges: DashMap<SessionId, Arc<dyn BridgeHandle>>,
}

#[async_trait]
impl BridgeRepository for BridgeRepositoryInMem {
    async fn register_bridge(&self, session_id: SessionId, bridge: Arc<dyn BridgeHandle>) {
        self.bridges.insert(session_id, bridge);
    }

    async fn bridge_by_id(&self, session_id: &SessionId) -> Option<Arc<dyn BridgeHandle>> {
        self.bridges.get(session_id).map(|e| e.to_owned())
    }
}
