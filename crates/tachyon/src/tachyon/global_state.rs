use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use msnp::shared::models::client_version::ClientVersion;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;
use tachyon_core::application::auth_use_case::AuthUseCase;
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::ids::{BridgeId, ClientVersion as CoreClientVersion, UserId};
use tachyon_core::infrastructure::app_state::AppState;

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    /// This bridge's name in every token it mints, so its logins never meet another bridge's.
    bridge: BridgeId,
    pending_alerts: DashMap<i32, AlertReceiver>,
    app_state: Arc<AppState>,
}

#[derive(Clone)]
pub struct GlobalState {
    inner: Arc<GlobalStateInner>,
}

/// Takes down what one Messenger connection registered, and only that: a reconnection may
/// already have put its own client and login under the same key by the time this drops.
pub struct ClientDropGuard {
    global_state: GlobalState,
    token: BridgeLinkToken,
    /// Taken by `release`, so `Drop` knows the teardown already ran.
    client: Option<TachyonClient>,
}

impl ClientDropGuard {
    /// Takes the client and its login down before the connection handler returns, so a
    /// reconnection finds the token free. `Drop` covers a handler that never got here.
    pub async fn release(mut self) {
        if let Some(client) = self.client.take() {
            self.remove_client(&client);
            abandon_login(self.global_state.app_state().auth_use_case(), &self.token).await;
        }
    }

    fn remove_client(&self, client: &TachyonClient) {
        if let Some(client) = self
            .global_state
            .tachyon_clients()
            .remove_if_same(self.token.as_str(), client)
        {
            client.shutdown();
        }
    }
}

impl Drop for ClientDropGuard {
    fn drop(&mut self) {
        let Some(client) = self.client.take() else {
            return;
        };
        self.remove_client(&client);
        // The backend session does not outlive the Messenger connection it served.
        let auth_use_case = self.global_state.app_state().auth_use_case().clone();
        let token = self.token.clone();
        tokio::spawn(async move { abandon_login(&auth_use_case, &token).await });
    }
}

pub(crate) async fn abandon_login(auth_use_case: &AuthUseCase, token: &BridgeLinkToken) {
    if let Err(e) = auth_use_case.abandon_login(token).await {
        log::error!("Could not close the backend session: {:?}", e);
    }
}

impl GlobalState {
    pub fn new(config: TachyonConfig, bridge: BridgeId, app_state: Arc<AppState>) -> Self {
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                bridge,
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

    pub fn insert_clients(
        &self,
        token: BridgeLinkToken,
        tachyon_client: TachyonClient,
    ) -> ClientDropGuard {
        self.inner
            .tachyon_clients
            .insert(token.as_str().to_owned(), tachyon_client.clone());
        ClientDropGuard {
            global_state: self.clone(),
            token,
            client: Some(tachyon_client),
        }
    }

    pub fn get_clients(&self, key: &str) -> Option<TachyonClient> {
        self.inner.tachyon_clients.get(key)
    }

    /// The ticket this bridge hands out for one client of an address. Deterministic, so the
    /// client's saved copy keeps working across restarts.
    pub fn ticket_for(&self, email: &EmailAddress, client: &ClientVersion) -> TicketToken {
        TicketToken(self.token_for(email, client).as_str().to_owned())
    }

    /// The same value, as the token core keys the login by.
    pub fn token_for(&self, email: &EmailAddress, client: &ClientVersion) -> BridgeLinkToken {
        let user = UserId::new(email.to_owned_user_id().as_str());
        BridgeLinkToken::mint(
            &self.inner.bridge,
            &user,
            &CoreClientVersion::new(client.to_string()),
        )
    }

    pub fn store_pending_alert(&self, key: i32, receiver: AlertReceiver) {
        self.inner.pending_alerts.insert(key, receiver);
    }

    pub fn take_pending_alert(&self, key: &i32) -> Option<AlertReceiver> {
        self.inner.pending_alerts.remove(key).map(|(_, recv)| recv)
    }

    pub fn is_token_linked(&self, token: &str) -> bool {
        self.app_state().auth_use_case().has_login(&BridgeLinkToken::new(token))
    }

    pub fn app_state(&self) -> &Arc<AppState> {
        &self.inner.app_state
    }
}
