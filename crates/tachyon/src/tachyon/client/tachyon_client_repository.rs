use dashmap::DashMap;
use crate::tachyon::client::tachyon_client::TachyonClient;

#[derive(Default)]
pub struct TachyonClientRepository {
    clients: DashMap<String, TachyonClient>,
}

impl TachyonClientRepository {

    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    pub fn find_by_oauth_state(&self, state: &str) -> Option<(String, TachyonClient)> {
        self.clients.iter()
            .find(|entry| entry.value().pending_oauth_state().as_deref() == Some(state))
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }
}


impl TachyonClientRepository {
    pub fn get(&self, key: &str) -> Option<TachyonClient> {
        self.clients.get(key).map(|x| x.value().clone())
    }

    pub fn insert(&self, key: String, value: TachyonClient) {
        self.clients.insert(key, value);
    }

    pub fn remove(&self, key: &str) -> Option<TachyonClient> {
        self.clients.remove(key).map(|(_, client)| client)
    }
}