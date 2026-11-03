use dashmap::DashMap;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::repository::RepositoryStr;

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

    pub(crate) fn single(&self) -> Option<TachyonClient> {

        if self.clients.len() > 1 {
            return None;
        }

        self.clients.iter().next().map(|x| x.value().clone())
    }
}


impl RepositoryStr<TachyonClient> for TachyonClientRepository {
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