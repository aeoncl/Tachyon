use crate::tachyon::tachyon_client::TachyonClient;
use crate::tachyon::secret_encryptor::SecretEncryptor;
use dashmap::DashMap;
use std::sync::Arc;

pub struct TachyonStateInner {
    clients: DashMap<String, TachyonClient>,
    token_validator: SecretEncryptor,

}

#[derive(Clone)]
pub struct TachyonState {
    pub inner: Arc<TachyonStateInner>,
}

impl TachyonState {

    pub fn new(token_validator: SecretEncryptor) -> Self {
        Self {
            inner: Arc::new(TachyonStateInner {
                clients: DashMap::new(),
                token_validator,
            })
        }
    }

    //FIXME: remove this and fix everywhere it's called to get the client using the key.
    pub fn get_single_client(&self) -> Option<TachyonClient> {
        if self.inner.clients.len() > 1 {
            return None;
        }

        self.inner.clients.iter().next().map(|x| x.value().clone())
    }

    pub fn insert_client(&self, key: String, client: TachyonClient) {
        self.inner.clients.insert(key, client);
    }

    pub fn get_client(&self, key: &str) -> Option<TachyonClient> {
        match self.inner.clients.get(key) {
            None => None,
            Some(found) => Some(found.value().clone()),
        }
    }

    pub fn remove_client(&self, key: &str) -> Option<TachyonClient> {
        self.inner.clients.remove(key).map(|(_, client)| client)
    }
    pub fn secret_encryptor(&self) -> &SecretEncryptor {
        &self.inner.token_validator
    }

}