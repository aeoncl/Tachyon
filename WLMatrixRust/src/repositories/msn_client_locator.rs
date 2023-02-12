use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::models::msn_client::MSNClient;


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
        self.data.lock().unwrap().insert(data);
    }

    pub fn remove(&self) {
        self.data.lock().unwrap().take();
    }

}