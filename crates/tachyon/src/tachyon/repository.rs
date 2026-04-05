pub trait RepositoryStr<T> {
    fn get(&self, key: &str) -> Option<T>;
    fn insert(&self, key: String, value: T);
    fn remove(&self, key: &str) -> Option<T>;
}

pub trait Repository<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn remove(&self, key: &K) -> Option<V>;
}