use crate::matrix::verification_request_repository::VerificationRequestRepository;
use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::secret_encryptor::SecretEncryptor;
use crate::tachyon::matrix_client_repository::MatrixClientRepository;
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use matrix_sdk::Client;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;

pub struct GlobalStateInner {
    tachyon_clients: TachyonClientRepository,
    matrix_clients: MatrixClientRepository,
    token_validator: SecretEncryptor,
    pending_ticket: DashMap<String, TicketToken>,
    pending_alerts: DashMap<i32, AlertReceiver>,
    pending_verification_requests: VerificationRequestRepository
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
        let matrix_client = self.global_state.matrix_clients().remove(&self.key);
        if let Some(client) = matrix_client {
            if let Some(user_id) = client.user_id() {
                self.global_state.pending_verification_requests().remove_for(user_id);
            }
        }
        let tachyon_client = self.global_state.tachyon_clients().remove(&self.key);
        if let Some(client) = tachyon_client {
            self.global_state.take_pending_ticket(client.own_user().get_email_address());
        }
    }
}


impl GlobalState {

    pub fn new(token_validator: SecretEncryptor) -> Self {
        Self {
            inner: Arc::new(GlobalStateInner {
                tachyon_clients: Default::default(),
                matrix_clients: Default::default(),
                token_validator,
                pending_ticket: DashMap::new(),
                pending_alerts: Default::default(),
                pending_verification_requests: Default::default(),
            })
        }
    }

    //FIXME: remove this and fix everywhere it's called to get the client using the key.
    pub fn get_single_client(&self) -> Option<TachyonClient> {
        self.tachyon_clients().single()
    }

    pub fn tachyon_clients(&self) -> &TachyonClientRepository {
        &self.inner.tachyon_clients
    }

    pub fn matrix_clients(&self) -> &MatrixClientRepository {
        &self.inner.matrix_clients
    }

    pub fn insert_clients(&self, key: String, tachyon_client: TachyonClient, matrix_client: Client) -> ClientDropGuard {
        self.inner.tachyon_clients.insert(key.clone(), tachyon_client);
        self.inner.matrix_clients.insert(key.clone(), matrix_client);

        ClientDropGuard::new(self.clone(), key)
    }

    pub fn get_clients(&self, key: &str) -> Option<(TachyonClient, Client)> {
        if let Some(tachyon_client) = self.inner.tachyon_clients.get(key) {
            if let Some(matrix_client) = self.inner.matrix_clients.get(key) {
                return Some((tachyon_client, matrix_client));
            }
        }
        None
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

    pub fn secret_encryptor(&self) -> &SecretEncryptor {
        &self.inner.token_validator
    }

}