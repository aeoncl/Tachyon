use chashmap::CHashMap;

use crate::models::ab_data::AbData;

use super::repository::Repository;

pub struct AbDataRepository {
    data : CHashMap<String, AbData>
}

impl Repository<String, AbData> for AbDataRepository{


    fn new() -> Self {
        return AbDataRepository{ data: CHashMap::new() };
    }

    fn find(&self, id: &String) -> Option<chashmap::ReadGuard<String,AbData>> {
        return self.data.get(id);
    }

    fn find_mut(&self, id: &String) -> Option<chashmap::WriteGuard<String,AbData>> {
        return self.data.get_mut(id);
    }

    fn add(&self, id: String, data: AbData) {
        self.data.insert(id, data);
    }

    fn remove(&self, id: &String) {
        self.data.remove(id);
    }

}