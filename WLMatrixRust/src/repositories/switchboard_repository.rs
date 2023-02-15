use std::{sync::{Mutex}, collections::HashMap};

use crate::models::switchboard::switchboard::Switchboard;


pub struct SwitchboardRepository {
    data : Mutex<HashMap<String, Switchboard>>
}

impl SwitchboardRepository {


    pub fn new() -> Self {
        return SwitchboardRepository{ data: Mutex::new(HashMap::new()) };
    }

    pub fn find(&self, id: &String) -> Option<Switchboard> {
        if let Some(found) = self.data.lock().unwrap().get(id) {
            return Some(found.clone());
        }
        return None;
    }

    pub fn add(&self, id: String, data: Switchboard) {
        self.data.lock().unwrap().insert(id, data);
    }

    pub fn remove(&self, id: &String) {
        self.data.lock().unwrap().remove(id);
    }

}