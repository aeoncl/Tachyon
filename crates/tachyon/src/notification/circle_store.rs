use std::collections::HashSet;
use std::hash::Hash;

use dashmap::DashMap;
use matrix_sdk::ruma::OwnedUserId;

use msnp::shared::models::uuid::Uuid;

pub struct CircleStore {
    circles: DashMap<Uuid, Circle>,
}

impl CircleStore {

    pub fn new() -> Self {
        Self {
            circles: Default::default(),
        }
    }

    pub async fn add_to_roster(&self, circle_id: &Uuid, to_add: Vec<OwnedUserId>) {
        match self.circles.get_mut(circle_id) {
            None => {
                let circle = Circle {
                    roster:  HashSet::from_iter(to_add.into_iter())
                };

                self.circles.insert(circle_id.clone(),  circle);

            }
            Some(mut circle) => {
                circle.roster.extend(to_add.into_iter())
            }
        }
    }

    pub async fn remove_from_roster(&self, circle_id: &Uuid, to_remove: Vec<OwnedUserId>) {
        match self.circles.get_mut(circle_id) {
            None => {}
            Some(mut circle) => {

                for current in to_remove {
                    circle.roster.remove(&current);
                }

                if circle.roster.is_empty() {
                    self.circles.remove(circle_id);
                }
            }
        }
    }

    pub fn get_roster(&self, circle_id: &Uuid) -> Option<HashSet<OwnedUserId>> {
        self.circles.get(circle_id).map(|c|c.value().roster.clone())
    }

}


pub struct Circle {
    roster: HashSet<OwnedUserId>
}