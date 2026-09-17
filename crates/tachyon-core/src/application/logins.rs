use crate::application::ports::BackendSession;
use crate::domain::auth::{InteractiveAuthStarted, BridgeLinkToken};
use crate::domain::ids::LoginId;
use crate::domain::verification::DeviceStatus;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Notify;

/// What the user still has to do before a pending login can be Ready
#[derive(Clone, Debug)]
pub enum Step {
    /// The backend has not accepted the login yet. `flow_id` is how the authorization
    Authenticate {
        flow_id: String,
        prompt: InteractiveAuthStarted,
    },
    /// Authenticated, but this device is not trusted by the user's identity.
    VerifyDevice,
}

/// A live login for one account
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

    pub(crate) fn ready(session: Arc<dyn BackendSession>, login_id: LoginId) -> Self {
        Self::Ready { session, login_id }
    }

    pub(crate) fn after_auth(
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
        status: DeviceStatus,
    ) -> Self {
        match status {
            DeviceStatus::Verified => Self::ready(session, login_id),
            DeviceStatus::Unverified => Self::pending(session, login_id, Step::VerifyDevice),
        }
    }

    pub(crate) fn authenticated(&self) -> bool {
        !matches!(
            self,
            Self::Pending {
                step: Step::Authenticate { .. },
                ..
            }
        )
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

    fn session_equals(&self, session: &Arc<dyn BackendSession>) -> bool {
        Arc::ptr_eq(self.session(), session)
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

#[derive(Default)]
pub struct Logins {
    logins: DashMap<BridgeLinkToken, Login>,
    flows: DashMap<String, BridgeLinkToken>,
}

impl Logins {
    pub(crate) fn get(&self, token: &BridgeLinkToken) -> Option<Login> {
        self.logins.get(token).map(|entry| entry.value().clone())
    }

    pub(crate) fn contains(&self, token: &BridgeLinkToken) -> bool {
        self.logins.contains_key(token)
    }

    pub(crate) fn is_token_linked_to(&self, token: &BridgeLinkToken, session: &Arc<dyn BackendSession>) -> bool {
        self.logins
            .get(token)
            .is_some_and(|entry| entry.value().session_equals(session))
    }

    pub(crate) fn ready_session(&self, token: &BridgeLinkToken) -> Option<Arc<dyn BackendSession>> {
        match self.logins.get(token)?.value() {
            Login::Ready { session, .. } => Some(session.clone()),
            Login::Pending { .. } => None,
        }
    }

    /// Puts `login` under the token, whatever is there.
    pub(crate) fn insert(&self, token: BridgeLinkToken, login: Login) {
        let flow = login.flow_id().map(str::to_owned);
        if let Some(previous) = self.logins.insert(token.clone(), login) {
            self.forget_flow(&previous);
        }
        if let Some(flow) = flow {
            self.flows.insert(flow, token);
        }
    }

    /// Puts `login` under the token only while the slot still holds the login built on
    /// `expected`. `false` leaves everything as it was.
    pub(crate) fn replace_if(
        &self,
        token: &BridgeLinkToken,
        expected: &Arc<dyn BackendSession>,
        login: Login,
    ) -> bool {
        let Some(mut slot) = self.logins.get_mut(token) else {
            return false;
        };
        if !slot.session_equals(expected) {
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

    pub(crate) fn remove(&self, token: &BridgeLinkToken) -> Option<Login> {
        let (_, removed) = self.logins.remove(token)?;
        self.forget_flow(&removed);
        Some(removed)
    }

    pub(crate) fn token_for_flow(&self, flow_id: &str) -> Option<BridgeLinkToken> {
        self.flows.get(flow_id).map(|entry| entry.value().clone())
    }

    fn forget_flow(&self, login: &Login) {
        if let Some(flow_id) = login.flow_id() {
            self.flows.remove(flow_id);
        }
    }
}
