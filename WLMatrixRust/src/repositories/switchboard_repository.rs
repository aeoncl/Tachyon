use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::models::{switchboard_handle::SwitchboardHandle};

pub struct SwitchboardRepository {
    data : Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<SwitchboardHandle>>>>> 
}

impl SwitchboardRepository {


    pub fn new() -> Self {
        return SwitchboardRepository{ data: Arc::new(Mutex::new(HashMap::new())) };
    }

    pub fn find(&self, id: &String) -> Option<Arc<tokio::sync::Mutex<SwitchboardHandle>>> {
        if let Some(found) = self.data.lock().unwrap().get(id) {
            return Some(found.clone());
        }
        return None;
    }

    pub fn add(&self, id: String, data: Arc<tokio::sync::Mutex<SwitchboardHandle>>) {
        self.data.lock().unwrap().insert(id, data);
    }

    pub fn remove(&self, id: &String) {
        self.data.lock().unwrap().remove(id);
    }

}