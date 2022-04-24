use chashmap::{CHashMap, ReadGuard, WriteGuard};
use crate::models::client_data::ClientData;

use super::repository::Repository;

pub struct ClientDataRepository {
    data : CHashMap<String, ClientData>
}

impl Repository<String, ClientData> for ClientDataRepository {

    fn new() -> ClientDataRepository {
        return ClientDataRepository{ data: CHashMap::new() };
    }

    fn find(&self, id: &String) -> Option<ReadGuard<String, ClientData>> {
       return self.data.get(id);
    }

    fn find_mut(&self, id: &String) -> Option<WriteGuard<String, ClientData>> {
        return self.data.get_mut(id);
     }

    fn add(&self, id: String, data: ClientData) {
        self.data.insert(id, data);
    }

    fn remove(&self, id: &String){
        self.data.remove(id);
    }
}