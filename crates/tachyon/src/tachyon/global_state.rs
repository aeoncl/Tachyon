use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::identifiers::ticket::{derive_ticket, derive_token};
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;
use tachyon_core::domain::auth::TachyonToken;
use tachyon_core::infrastructure::app_state::AppState;

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    /// Raw `local.key`, used to derive each account's ticket.
    token_secret: Vec<u8>,
    pending_alerts: DashMap<i32, AlertReceiver>,
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
        // The backend session does not outlive the Messenger connection it served.
        let auth_use_case = self.global_state.app_state().auth_use_case().clone();
        let token = TachyonToken::new(&self.key);
        tokio::spawn(async move {
            if let Err(e) = auth_use_case.abandon(&token).await {
                log::error!("Could not close the backend session: {:?}", e);
            }
        });
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

    /// Whether a token names something we will serve pages for: a signed-in client, or a
    /// login that is still finishing.
    pub fn is_session_token(&self, token: &str) -> bool {
        self.tachyon_clients().get(token).is_some()
            || self.app_state().auth_use_case().has_login(&TachyonToken::new(token))
    }

    pub fn app_state(&self) -> &Arc<AppState> {
        &self.inner.app_state
    }
}
