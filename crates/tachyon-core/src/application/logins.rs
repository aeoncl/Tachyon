use crate::application::ports::BackendSession;
use crate::domain::auth::{InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::LoginId;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Notify;

/// What the user still has to do in a browser before a pending login can be used.
#[derive(Clone, Debug)]
pub enum Step {
    /// The backend has not accepted the login yet. `flow_id` is how the authorization
    /// callback finds its way back to this login; for OAuth it is the CSRF `state`.
    Authenticate {
        flow_id: String,
        prompt: InteractiveAuthStarted,
    },
    /// Authenticated, but this device is not trusted by the user's identity.
    VerifyDevice,
}

/// A live login for one account. Sessions hold open backend connections, so this only ever
/// lives in memory; the store keeps what is needed to rebuild one after a restart.
#[derive(Clone)]
pub(crate) enum Login {
    Pending {
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
        step: Step,
        /// Fired when the step advances or the login is abandoned, so a waiter re-reads.
        changed: Arc<Notify>,
    },
    Ready {
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
    },
}

impl Login {
    pub(crate) fn pending(session: Arc<dyn BackendSession>, login_id: LoginId, step: Step) -> Self {
        Self::Pending {
            session,
            login_id,
            step,
            changed: Arc::new(Notify::new()),
        }
    }

    pub(crate) fn session(&self) -> &Arc<dyn BackendSession> {
        match self {
            Self::Pending { session, .. } | Self::Ready { session, .. } => session,
        }
    }

    pub(crate) fn login_id(&self) -> &LoginId {
        match self {
            Self::Pending { login_id, .. } | Self::Ready { login_id, .. } => login_id,
        }
    }
}

/// One live login per token, plus the reverse index the authorization callback needs. A
/// token is a bridge's name for an account, so the same account signed in through two
/// bridges is two logins with two backend sessions.
#[derive(Default)]
pub struct Logins {
    logins: DashMap<TachyonToken, Login>,
    flows: DashMap<String, TachyonToken>,
}

impl Logins {
    pub(crate) fn get(&self, token: &TachyonToken) -> Option<Login> {
        self.logins.get(token).map(|entry| entry.value().clone())
    }

    pub(crate) fn contains(&self, token: &TachyonToken) -> bool {
        self.logins.contains_key(token)
    }

    pub(crate) fn ready_session(&self, token: &TachyonToken) -> Option<Arc<dyn BackendSession>> {
        match self.logins.get(token)?.value() {
            Login::Ready { session, .. } => Some(session.clone()),
            Login::Pending { .. } => None,
        }
    }

    pub(crate) fn insert(&self, token: TachyonToken, login: Login) -> Option<Login> {
        if let Login::Pending {
            step: Step::Authenticate { flow_id, .. },
            ..
        } = &login
        {
            self.flows.insert(flow_id.clone(), token.clone());
        }
        let previous = self.logins.insert(token, login);
        if let Some(previous) = &previous {
            self.forget_flow(previous);
        }
        previous
    }

    pub(crate) fn remove(&self, token: &TachyonToken) -> Option<Login> {
        let (_, removed) = self.logins.remove(token)?;
        self.forget_flow(&removed);
        Some(removed)
    }

    pub(crate) fn token_for_flow(&self, flow_id: &str) -> Option<TachyonToken> {
        self.flows.get(flow_id).map(|entry| entry.value().clone())
    }

    fn forget_flow(&self, login: &Login) {
        if let Login::Pending {
            step: Step::Authenticate { flow_id, .. },
            ..
        } = login
        {
            self.flows.remove(flow_id);
        }
    }
}
