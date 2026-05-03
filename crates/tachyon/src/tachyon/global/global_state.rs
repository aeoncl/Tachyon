use crate::matrix::services::login::MatrixLoginService;
use crate::matrix::verification_request_repository::VerificationRequestRepository;
use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::global::secret_encryptor::SecretEncryptor;
use crate::tachyon::global::tachyon_config::TachyonConfig;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    token_validator: SecretEncryptor,
    pending_ticket: DashMap<String, TicketToken>,
    pending_alerts: DashMap<i32, AlertReceiver>,
    pending_verification_requests: VerificationRequestRepository,
    matrix_login_service: Box<dyn MatrixLoginService>
}

#[derive(Clone)]
pub struct GlobalState {
    inner: Arc<GlobalStateInner>,
}

pub struct ClientDropGuard {
    global_state: GlobalState,
    key: String
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
            self.global_state.take_pending_ticket(client.own_user().get_email_address());

            //Todo change this so we use a neutral key
            let own_user_id = client.own_user().get_email_address().to_owned_user_id();
            self.global_state.pending_verification_requests().remove_for(&own_user_id);
        }

        println!("Client Drop Guard dropped");
    }
}


impl GlobalState {

    pub fn new(config: TachyonConfig, token_validator: SecretEncryptor, matrix_login_service: Box<dyn MatrixLoginService>) -> Self {
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                token_validator,
                pending_ticket: DashMap::new(),
                pending_alerts: Default::default(),
                pending_verification_requests: Default::default(),
                matrix_login_service,
            })
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

    pub fn get_client(&self, key: &str) -> Option<TachyonClient> {
        self.inner.tachyon_clients.get(key)
    }

    pub fn store_pending_ticket(&self, key: EmailAddress,  ticket: TicketToken) {
        self.inner.pending_ticket.insert(key.to_string(), ticket);
    }

    pub fn take_pending_ticket(&self, key: &EmailAddress) -> Option<TicketToken> {
        self.inner.pending_ticket.remove(key.as_str()).map(|(_, ticket)| ticket)
    }

    pub fn store_pending_alert(&self, key: i32, receiver: AlertReceiver) {
        self.inner.pending_alerts.insert(key, receiver);
    }

    pub fn take_pending_alert(&self, key: &i32) -> Option<AlertReceiver> {
        self.inner.pending_alerts.remove(key).map(|(_, recv)| recv)
    }

    pub fn pending_verification_requests(&self) -> &VerificationRequestRepository {
        &self.inner.pending_verification_requests
    }

    pub fn matrix_login_service(&self) -> &Box<dyn MatrixLoginService> {
        &self.inner.matrix_login_service
    }

    pub fn secret_encryptor(&self) -> &SecretEncryptor {
        &self.inner.token_validator
    }

}