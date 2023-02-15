use std::{sync::{Mutex}};

use crate::models::notification::msn_client::MSNClient;
pub struct MSNClientLocator {
    data : Mutex<Option<MSNClient>>
}

impl MSNClientLocator {


    pub fn new() -> Self {
        return MSNClientLocator{ data: Mutex::new(None) };
    }

    pub fn get(&self) -> Option<MSNClient> {
        let data = self.data.lock().unwrap();
        return data.clone();
    }

    pub fn set(&self, data: MSNClient) {
        let _result = self.data.lock().unwrap().insert(data);
    }

    pub fn remove(&self) {
        self.data.lock().unwrap().take();
    }

}