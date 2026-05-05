use dashmap::DashMap;
use msnp::shared::models::email_address::EmailAddress;
use crate::tachyon::repository::RepositoryStr;
use crate::tachyon::state::session::tachyon_client_repository::TachyonSessionData;
use crate::tachyon::tachyon_client::TachyonClient;

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

    pub fn single(&self) -> Option<TachyonClient> {

        if self.clients.len() > 1 {
            return None;
        }

        self.clients.iter().next().map(|x| x.value().clone())
    }

    pub fn find_by_email(&self, email: &EmailAddress) -> Option<TachyonSessionData> {
        self.clients.iter().find(|entry| entry.value().own_user().get_email_address() == email).map(|client| client.value().clone())
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