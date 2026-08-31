use crate::matrix::verification_request_repository::VerificationRequestRepository;
use crate::tachyon::alert::{Alert, AlertReceiver};
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::identifiers::ticket::{derive_ticket, derive_token};
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use matrix_sdk::Client;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;
use tachyon_core::domain::auth::{InteractiveAuthStarted, TachyonToken};
use tachyon_core::domain::ids::LoginId;
use tachyon_core::infrastructure::app_state::AppState;

/// An interactive login the user has been alerted about but not yet completed.
///
/// Keyed by a flow id the bridge owns: for OAuth that is the `csrf_token`, which comes back
/// as the `state` query parameter, so the callback can find its way here.
pub struct PendingLogin {
    pub login_id: LoginId,
    pub email: EmailAddress,
    /// What the browser has to be pointed at to complete this login.
    pub prompt: InteractiveAuthStarted,
    /// Fired once the browser side finishes, releasing the waiting `USR` handler.
    pub alert: Alert,
}

/// Stands in for a `TachyonClient` that does not exist yet.
///
/// On a fresh login the user is already in front of their browser, so device confirmation
/// happens right there rather than costing a second `NOT` alert — but the `TachyonClient`
/// only comes into being once the waiting `USR` handler wakes. This carries what the
/// confirmation pages need in the meantime, keyed by the same ticket, so those pages never
/// have to know which of the two they are talking to.
pub struct PreSession {
    matrix_client: Client,
    alerts: DashMap<i32, Alert>,
}

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    /// Raw `local.key`, used to derive each account's ticket.
    token_secret: Vec<u8>,
    pending_alerts: DashMap<i32, AlertReceiver>,
    pending_logins: DashMap<String, PendingLogin>,
    pre_sessions: DashMap<String, PreSession>,
    pending_verification_requests: VerificationRequestRepository,
    app_state: Arc<AppState>,
}

#[derive(Clone)]
pub struct GlobalState {
    inner: Arc<GlobalStateInner>,
}

pub struct ClientDropGuard {
    global_state: GlobalState,
    key: String,
}

impl ClientDropGuard {
    pub fn new(global_state: GlobalState, key: String) -> Self {
        Self { global_state, key }
    }
}

impl Drop for ClientDropGuard {
    fn drop(&mut self) {
        let tachyon_client = self.global_state.tachyon_clients().remove(&self.key);
        if let Some(client) = tachyon_client {
            client.shutdown();

            //Todo change this so we use a neutral key
            let own_user_id = client.own_user().get_email_address().to_owned_user_id();
            self.global_state
                .pending_verification_requests()
                .remove_for(&own_user_id);
        }

        println!("Client Drop Guard dropped");
    }
}

impl GlobalState {
    pub fn new(
        config: TachyonConfig,
        token_secret: Vec<u8>,
        app_state: Arc<AppState>,
    ) -> Self {
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                token_secret,
                pending_alerts: Default::default(),
                pending_logins: DashMap::new(),
                pre_sessions: DashMap::new(),
                pending_verification_requests: Default::default(),
                app_state,
            }),
        }
    }

    pub fn get_config(&self) -> &TachyonConfig {
        &self.inner.config
    }

    //FIXME: remove this and fix everywhere it's called to get the client using the key.
    pub fn get_single_client(&self) -> Option<TachyonClient> {
        self.tachyon_clients().single()
    }

    pub fn tachyon_clients(&self) -> &TachyonClientRepository {
        &self.inner.tachyon_clients
    }

    pub fn insert_clients(&self, key: String, tachyon_client: TachyonClient) -> ClientDropGuard {
        self.inner.tachyon_clients.insert(key.clone(), tachyon_client);
        ClientDropGuard::new(self.clone(), key)
    }

    pub fn get_clients(&self, key: &str) -> Option<TachyonClient> {
        self.inner.tachyon_clients.get(key)
    }

    /// The ticket this instance hands out for an address. Stable across restarts, so the
    /// client's saved copy keeps working.
    pub fn ticket_for(&self, email: &EmailAddress) -> TicketToken {
        derive_ticket(&self.inner.token_secret, email)
    }

    /// The same value, as core's opaque account token.
    pub fn token_for(&self, email: &EmailAddress) -> TachyonToken {
        derive_token(&self.inner.token_secret, email)
    }

    pub fn store_pending_alert(&self, key: i32, receiver: AlertReceiver) {
        self.inner.pending_alerts.insert(key, receiver);
    }

    pub fn take_pending_alert(&self, key: &i32) -> Option<AlertReceiver> {
        self.inner.pending_alerts.remove(key).map(|(_, recv)| recv)
    }

    pub fn store_pending_login(&self, flow_id: String, pending_login: PendingLogin) {
        self.inner.pending_logins.insert(flow_id, pending_login);
    }

    /// Borrow a pending login without consuming it — the alert is only fired once the
    /// browser actually completes, which may be several requests later.
    pub fn peek_pending_login<T>(
        &self,
        flow_id: &str,
        read: impl FnOnce(&PendingLogin) -> T,
    ) -> Option<T> {
        self.inner.pending_logins.get(flow_id).map(|entry| read(entry.value()))
    }

    pub fn take_pending_login(&self, flow_id: &str) -> Option<PendingLogin> {
        self.inner
            .pending_logins
            .remove(flow_id)
            .map(|(_, pending)| pending)
    }

    pub fn insert_pre_session(
        &self,
        ticket: String,
        matrix_client: Client,
        notification_id: i32,
        alert: Alert,
    ) {
        let alerts = DashMap::new();
        alerts.insert(notification_id, alert);
        self.inner
            .pre_sessions
            .insert(ticket, PreSession { matrix_client, alerts });
    }

    pub fn remove_pre_session(&self, ticket: &str) {
        self.inner.pre_sessions.remove(ticket);
    }

    /// Whether a token names something we will serve pages for: a signed-in client, or a
    /// login that is still finishing.
    pub fn is_session_token(&self, token: &str) -> bool {
        self.tachyon_clients().get(token).is_some() || self.inner.pre_sessions.contains_key(token)
    }

    /// The matrix client behind a device-confirmation page, whether the MSNP client has
    /// finished signing in or is still waiting on it.
    pub fn confirmation_client(&self, token: &str) -> Option<Client> {
        if let Some(client) = self.tachyon_clients().get(token) {
            return Some(client.matrix_client());
        }

        self.inner
            .pre_sessions
            .get(token)
            .map(|pre_session| pre_session.matrix_client.clone())
    }

    pub fn has_confirmation_alert(&self, token: &str, notification_id: i32) -> bool {
        if let Some(client) = self.tachyon_clients().get(token) {
            return client.alerts().contains_key(&notification_id);
        }

        self.inner
            .pre_sessions
            .get(token)
            .is_some_and(|pre_session| pre_session.alerts.contains_key(&notification_id))
    }

    /// Takes the alert so it can be fired. It is a oneshot, so this consumes it.
    pub fn take_confirmation_alert(&self, token: &str, notification_id: i32) -> Option<Alert> {
        if let Some(client) = self.tachyon_clients().get(token) {
            return client.alerts().remove(&notification_id).map(|(_, alert)| alert);
        }

        self.inner.pre_sessions.get(token).and_then(|pre_session| {
            pre_session.alerts.remove(&notification_id).map(|(_, alert)| alert)
        })
    }

    pub fn pending_verification_requests(&self) -> &VerificationRequestRepository {
        &self.inner.pending_verification_requests
    }

    pub fn app_state(&self) -> &Arc<AppState> {
        &self.inner.app_state
    }
}
