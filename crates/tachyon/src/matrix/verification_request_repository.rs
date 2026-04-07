use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use matrix_sdk::encryption::verification::VerificationRequest;
use crate::tachyon::repository::RepositoryStr;

pub struct VerificationRequestRepository {

    requests: DashMap<String, VerificationRequest>,

}

impl VerificationRequestRepository {

    pub fn get(&self, key: &str) -> Option<Ref<String, VerificationRequest>> {
        self.requests.get(key)
    }

    pub fn insert(&self, key: String, value: VerificationRequest) {
        self.requests.insert(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.requests.remove(key);
    }
}



impl Default for VerificationRequestRepository {
    fn default() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }
}