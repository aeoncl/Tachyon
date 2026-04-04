use crate::tachyon::secret_encryptor::SecretEncryptor;
use crate::tachyon::tachyon_client::TachyonClient;
use dashmap::DashMap;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::Arc;
use matrix_sdk::Client;
use tokio::sync::oneshot;
use msnp::shared::models::email_address::EmailAddress;

pub trait Repository<T> {
    fn get(&self, key: &str) -> Option<T>;
    fn insert(&self, key: String, value: T);
    fn remove(&self, key: &str) -> Option<T>;
}

#[derive(Default)]
pub struct TachyonClientRepository {
    clients: DashMap<String, TachyonClient>,
}

impl TachyonClientRepository {
    fn single(&self) -> Option<TachyonClient> {

        if self.clients.len() > 1 {
            return None;
        }

        self.clients.iter().next().map(|x| x.value().clone())
    }

    pub fn find_by_email(&self, email: &EmailAddress) -> Option<TachyonClient> {
        self.clients.iter().find(|entry| entry.value().own_user().get_email_address() == email).map(|client| client.value().clone())
    }
}

impl Repository<TachyonClient> for TachyonClientRepository {
    fn get(&self, key: &str) -> Option<TachyonClient> {
        self.clients.get(key).map(|x| x.value().clone())
    }

    fn insert(&self, key: String, value: TachyonClient) {
        self.clients.insert(key, value);
    }

    fn remove(&self, key: &str) -> Option<TachyonClient> {
        self.clients.remove(key).map(|(_, client)| client)
    }
}

#[derive(Default)]
pub struct MatrixClientRepository {
    clients: DashMap<String, Client>
}

impl Repository<Client> for MatrixClientRepository {

    fn get(&self, key: &str) -> Option<Client> {
        self.clients.get(key).map(|x| x.value().clone())
    }

    fn insert(&self, key: String, value: Client) {
        self.clients.insert(key, value);
    }

    fn remove(&self, key: &str) -> Option<Client> {
        self.clients.remove(key).map(|(_, client)| client)
    }

}


pub struct TachyonStateInner {
    tachyon_clients: TachyonClientRepository,
    matrix_clients: MatrixClientRepository,
    token_validator: SecretEncryptor,
    pending_ticket: DashMap<String, TicketToken>,
    pending_alerts: DashMap<i32, oneshot::Receiver<crate::tachyon::tachyon_client::AlertResult>>
}

#[derive(Clone)]
pub struct TachyonState {
    pub inner: Arc<TachyonStateInner>,
}

impl TachyonState {

    pub fn new(token_validator: SecretEncryptor) -> Self {
        Self {
            inner: Arc::new(TachyonStateInner {
                tachyon_clients: Default::default(),
                matrix_clients: Default::default(),
                token_validator,
                pending_ticket: DashMap::new(),
                pending_alerts: Default::default(),
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

    pub fn store_pending_alert(&self, key: i32,  oneshot: oneshot::Receiver<crate::tachyon::tachyon_client::AlertResult>) {
        self.inner.pending_alerts.insert(key, oneshot);
    }

    pub fn take_pending_alert(&self, key: &i32) -> Option<oneshot::Receiver<crate::tachyon::tachyon_client::AlertResult>> {
        self.inner.pending_alerts.remove(key).map(|(_, recv)| recv)
    }

    pub fn secret_encryptor(&self) -> &SecretEncryptor {
        &self.inner.token_validator
    }

}