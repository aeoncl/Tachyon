use std::collections::HashSet;
use matrix_sdk::ruma::EventId;

pub struct EventDeduplicator {
    events: HashSet<String>
}

impl Default for EventDeduplicator {
    fn default() -> Self {
        EventDeduplicator { events: HashSet::new() }
    }
}

impl EventDeduplicator {

    pub fn insert(&mut self, event_id: &EventId) {
        self.events.insert(event_id.to_string());
    }

    pub fn insert_once(&mut self,  event_id: &EventId) -> bool {
        if self.contains(event_id) {
            false
        } else {
            self.insert(event_id);
            true
        }
    }

    pub fn contains(&self, event_id: &EventId) -> bool {
        self.events.contains(event_id.as_str())
    }
}