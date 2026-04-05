use crate::tachyon::alert::Alert;
use crate::tachyon::repository::Repository;
use dashmap::DashMap;

pub struct AlertRepository {
    alerts: DashMap<i32, Alert>,
}

impl Repository<i32, Alert> for AlertRepository {
    fn get(&self, _key: &i32) -> Option<Alert> {
        todo!("Not implemented")
    }

    fn insert(&self, key: i32, value: Alert) {
        self.alerts.insert(key, value);
    }

    fn remove(&self, key: &i32) -> Option<Alert> {
        self.alerts.remove(key).map(|(_, alert)| alert)
    }
}

impl Default for AlertRepository {
    fn default() -> Self {
        Self {
            alerts: DashMap::new(),
        }
    }
}
