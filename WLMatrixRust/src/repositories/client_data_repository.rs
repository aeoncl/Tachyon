use chashmap::{CHashMap, ReadGuard, WriteGuard};
use crate::models::client_data::ClientData;

pub struct ClientDataRepository {
    data : CHashMap<String, ClientData>
}

pub trait Repository<K, V> {

    fn find(&self, id: &K) -> Option<ReadGuard<K,V>>;
    fn find_mut(&self, id: &K) -> Option<WriteGuard<K,V>>;
    fn add(&self, id: K, data: V);
    fn remove(&self, id: &K);
    fn new() -> Self;
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