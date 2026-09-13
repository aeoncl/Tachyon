use crate::application::ports::BackendSession;
use crate::domain::auth::{InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::LoginId;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
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

/// One occupancy of a token's slot. `Logins` mints one for every login that goes in, and the
/// login keeps it as it advances. Whoever looked at a login names its attempt when acting on
/// it later, so a login that replaced it in the meantime is left alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt(u64);

/// A live login for one account. Sessions hold open backend connections, so this only ever
/// lives in memory; the store keeps what is needed to rebuild one after a restart.
#[derive(Clone)]
pub(crate) enum Login {
    Pending {
        attempt: Attempt,
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
        step: Step,
        /// Fired when the step advances or the login is abandoned, so a waiter re-reads.
        changed: Arc<Notify>,
    },
    Ready {
        attempt: Attempt,
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
    },
}

impl Login {
    pub(crate) fn pending(
        attempt: Attempt,
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
        step: Step,
    ) -> Self {
        Self::Pending {
            attempt,
            session,
            login_id,
            step,
            changed: Arc::new(Notify::new()),
        }
    }

    pub(crate) fn ready(
        attempt: Attempt,
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
    ) -> Self {
        Self::Ready {
            attempt,
            session,
            login_id,
        }
    }

    pub(crate) fn attempt(&self) -> Attempt {
        match self {
            Self::Pending { attempt, .. } | Self::Ready { attempt, .. } => *attempt,
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

    fn flow_id(&self) -> Option<&str> {
        match self {
            Self::Pending {
                step: Step::Authenticate { flow_id, .. },
                ..
            } => Some(flow_id),
            _ => None,
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
    attempts: AtomicU64,
}

impl Logins {
    pub(crate) fn mint(&self) -> Attempt {
        Attempt(self.attempts.fetch_add(1, Ordering::Relaxed))
    }

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

    /// Puts `login` under the token, whatever is there.
    pub(crate) fn insert(&self, token: TachyonToken, login: Login) {
        let flow = login.flow_id().map(str::to_owned);
        if let Some(previous) = self.logins.insert(token.clone(), login) {
            self.forget_flow(&previous);
        }
        if let Some(flow) = flow {
            self.flows.insert(flow, token);
        }
    }

    /// Puts `login` under the token only while the slot still holds `expected`. `false`
    /// leaves everything as it was.
    pub(crate) fn replace_if(&self, token: &TachyonToken, expected: Attempt, login: Login) -> bool {
        let Some(mut slot) = self.logins.get_mut(token) else {
            return false;
        };
        if slot.attempt() != expected {
            return false;
        }
        let flow = login.flow_id().map(str::to_owned);
        let previous = std::mem::replace(slot.value_mut(), login);
        drop(slot);
        self.forget_flow(&previous);
        if let Some(flow) = flow {
            self.flows.insert(flow, token.clone());
        }
        true
    }

    pub(crate) fn remove(&self, token: &TachyonToken) -> Option<Login> {
        let (_, removed) = self.logins.remove(token)?;
        self.forget_flow(&removed);
        Some(removed)
    }

    /// Takes the login out only while the slot still holds `expected`.
    pub(crate) fn remove_if(&self, token: &TachyonToken, expected: Attempt) -> Option<Login> {
        let (_, removed) = self
            .logins
            .remove_if(token, |_, login| login.attempt() == expected)?;
        self.forget_flow(&removed);
        Some(removed)
    }

    pub(crate) fn token_for_flow(&self, flow_id: &str) -> Option<TachyonToken> {
        self.flows.get(flow_id).map(|entry| entry.value().clone())
    }

    fn forget_flow(&self, login: &Login) {
        if let Some(flow_id) = login.flow_id() {
            self.flows.remove(flow_id);
        }
    }
}
