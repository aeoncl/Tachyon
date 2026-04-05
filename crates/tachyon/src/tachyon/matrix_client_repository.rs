use dashmap::DashMap;
use matrix_sdk::Client;
use crate::tachyon::repository::RepositoryStr;

#[derive(Default)]
pub struct MatrixClientRepository {
    clients: DashMap<String, Client>
}

impl RepositoryStr<Client> for MatrixClientRepository {

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