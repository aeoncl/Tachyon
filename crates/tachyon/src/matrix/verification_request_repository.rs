use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use matrix_sdk::encryption::verification::VerificationRequest;
use matrix_sdk::ruma::UserId;
use crate::tachyon::repository::RepositoryStr;

pub struct VerificationRequestRepository {

    requests: DashMap<String, VerificationRequest>,

}

impl VerificationRequestRepository {

    pub fn get(&self, key: &str) -> Option<Ref<'_, String, VerificationRequest>> {
        self.requests.get(key)
    }

    pub fn insert(&self, key: String, value: VerificationRequest) {
        self.requests.insert(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.requests.remove(key);
    }

    pub fn remove_for(&self, user_id: &UserId) {
        self.requests.iter()
            .filter(|entry| entry.value().own_user_id() == user_id)
            .for_each(|entry| {
                self.requests.remove(entry.key().as_str());
            });
    }
}



impl Default for VerificationRequestRepository {
    fn default() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }
}