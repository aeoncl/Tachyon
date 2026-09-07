use crate::tachyon::alert::{Alert, AlertReceiver};
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::identifiers::ticket::{derive_ticket, derive_token};
use crate::tachyon::repository::RepositoryStr;
use dashmap::{DashMap, DashSet};
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

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    /// Raw `local.key`, used to derive each account's ticket.
    token_secret: Vec<u8>,
    pending_alerts: DashMap<i32, AlertReceiver>,
    pending_logins: DashMap<String, PendingLogin>,
    authorized_tickets: DashSet<String>,
    pending_verifications: DashMap<String, Alert>,
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
        if let Some(client) = self.global_state.tachyon_clients().remove(&self.key) {
            client.shutdown();
        }
    }
}

impl GlobalState {
    pub fn new(config: TachyonConfig, token_secret: Vec<u8>, app_state: Arc<AppState>) -> Self {
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                token_secret,
                pending_alerts: Default::default(),
                pending_logins: DashMap::new(),
                authorized_tickets: DashSet::new(),
                pending_verifications: DashMap::new(),
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
        // The client now answers for this ticket, so the sign-in grant is spent.
        self.inner.authorized_tickets.remove(&key);
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

    /// Lets a ticket reach the device confirmation pages before its `TachyonClient` exists.
    pub fn authorize_ticket(&self, ticket: &str) {
        self.inner.authorized_tickets.insert(ticket.to_owned());
    }

    pub fn deauthorize_ticket(&self, ticket: &str) {
        self.inner.authorized_tickets.remove(ticket);
    }

    /// Whether a token names something we will serve pages for: a signed-in client, or a
    /// login that is still finishing.
    pub fn is_session_token(&self, token: &str) -> bool {
        self.tachyon_clients().get(token).is_some()
            || self.inner.authorized_tickets.contains(token)
    }

    /// The channel that releases the `USR` handler waiting on device verification. Kept
    /// apart from the ticket authorization on purpose: the pages fire this the moment the
    /// device is trusted, while the handler still needs a few more requests' worth of
    /// authorized ticket to finish restoring the session.
    pub fn store_pending_verification(&self, ticket: String, alert: Alert) {
        self.inner.pending_verifications.insert(ticket, alert);
    }

    /// Takes the alert so it can be fired. It is a oneshot, so this consumes it.
    pub fn take_pending_verification(&self, ticket: &str) -> Option<Alert> {
        self.inner
            .pending_verifications
            .remove(ticket)
            .map(|(_, alert)| alert)
    }

    pub fn app_state(&self) -> &Arc<AppState> {
        &self.inner.app_state
    }
}
