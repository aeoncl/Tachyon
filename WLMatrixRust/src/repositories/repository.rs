use chashmap::{WriteGuard, ReadGuard};


pub trait Repository<K, V> {

    fn find(&self, id: &K) -> Option<ReadGuard<K,V>>;
    fn find_mut(&self, id: &K) -> Option<WriteGuard<K,V>>;
    fn add(&self, id: K, data: V);
    fn remove(&self, id: &K);
    fn new() -> Self;
}