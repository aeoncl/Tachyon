use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::secret_encryptor::SecretEncryptor;
use crate::tachyon::matrix_client_repository::MatrixClientRepository;
use dashmap::DashMap;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;
use matrix_sdk::encryption::verification::VerificationRequest;
use tokio::sync::oneshot;
use crate::matrix::verification_request_repository::VerificationRequestRepository;

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

    pub fn store_pending_ticket(&self, key: String,  ticket: TicketToken) {
        self.inner.pending_ticket.insert(key, ticket);
    }

    pub fn take_pending_ticket(&self, key: &str) -> Option<TicketToken> {
        self.inner.pending_ticket.remove(key).map(|(_, ticket)| ticket)
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