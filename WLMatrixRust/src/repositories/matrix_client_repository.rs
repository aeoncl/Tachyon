use chashmap::{CHashMap, ReadGuard, WriteGuard};
use matrix_sdk::Client;

use super::repository::Repository;

pub struct MatrixClientRepository {
    data : CHashMap<String, Client>
}

impl Repository<String, Client> for MatrixClientRepository {

    fn new() -> MatrixClientRepository {
        return MatrixClientRepository{ data: CHashMap::new() };
    }

    fn find(&self, id: &String) -> Option<ReadGuard<String, Client>> {
       return self.data.get(id);
    }

    fn find_mut(&self, id: &String) -> Option<WriteGuard<String, Client>> {
        return self.data.get_mut(id);
     }

    fn add(&self, id: String, data: Client) {
        self.data.insert(id, data);
    }

    fn remove(&self, id: &String){
        self.data.remove(id);
    }
}