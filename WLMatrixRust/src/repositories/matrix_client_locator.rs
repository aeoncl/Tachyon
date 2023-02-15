use std::{sync::RwLock};

use matrix_sdk::Client;
pub struct MatrixClientLocator {
    data : RwLock<Option<Client>>
}

impl MatrixClientLocator {


    pub fn new() -> Self {
        return MatrixClientLocator{ data: RwLock::new(None) };
    }

    pub fn get(&self) -> Option<Client> {
        let data = self.data.read().unwrap();
        return data.clone();
    }

    pub fn set(&self, data: Client) {
        self.data.write().unwrap().insert(data);
    }

    pub fn remove(&self) {
        self.data.write().unwrap().take();
    }

}