use chashmap::CHashMap;
use tokio::sync::broadcast::Sender;

use crate::models::{switchboard_data::SwitchboardData};

use super::repository::Repository;

pub struct SwitchboardRepository {
    data : CHashMap<String, SwitchboardData>
}

impl Repository<String, SwitchboardData> for SwitchboardRepository {


    fn new() -> Self {
        return SwitchboardRepository{ data: CHashMap::new() };
    }

    fn find(&self, id: &String) -> Option<chashmap::ReadGuard<String, SwitchboardData>> {
        return self.data.get(id);
    }

    fn find_mut(&self, id: &String) -> Option<chashmap::WriteGuard<String, SwitchboardData>> {
        return self.data.get_mut(id);
    }

    fn add(&self, id: String, data: SwitchboardData) {
        self.data.insert(id, data);
    }

    fn remove(&self, id: &String) {
        self.data.remove(id);
    }

}