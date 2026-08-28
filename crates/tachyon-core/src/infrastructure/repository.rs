use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use crate::application::ports::{BackendSession, SessionRepositoryTrait};
use crate::ids::LoginId;


pub(crate) struct SessionRepository {
    sessions: DashMap<LoginId, Arc<dyn BackendSession>>
}

impl SessionRepositoryTrait for SessionRepository {
    fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession> ) {
        self.sessions.insert(login_id, session);
    }
}

