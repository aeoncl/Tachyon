use std::{sync::{Mutex, RwLock}, collections::HashMap};

use crate::models::switchboard::switchboard::Switchboard;

#[derive(Debug)]
pub struct SwitchboardRepository {
    data : RwLock<HashMap<String, Switchboard>>
}

impl SwitchboardRepository {


    pub fn new() -> Self {
        return SwitchboardRepository{ data: RwLock::new(HashMap::new()) };
    }

    pub fn find(&self, id: &String) -> Option<Switchboard> {
        if let Some(found) = self.data.read().unwrap().get(id) {
            return Some(found.clone());
        }
        return None;
    }

    pub fn add(&self, id: String, data: Switchboard) {
        self.data.write().unwrap().insert(id, data);
    }
    pub fn remove(&self, id: &String) {
        self.data.write().unwrap().remove(id);
    }

}