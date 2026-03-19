use std::collections::VecDeque;
use std::sync::Mutex;
use dashmap::DashMap;
use msnp::shared::models::oim::OIM;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::CircleData;
use msnp::soap::abch::msnab_datatypes::{BaseMember, ContactType};

pub enum AddressBookContact {
    Contact(ContactType),
    Circle(CircleData),
}

#[derive(Default)]
pub struct SoapHolder {
    pub oims: DashMap<String, OIM>,
    pub contacts: Mutex<Vec<AddressBookContact>>,
    pub circle_contacts: DashMap<String, Vec<ContactType>>,
    pub memberships: Mutex<Vec<BaseMember>>,
}

impl SoapHolder {

    pub fn add_oim(&self, oim: OIM) {
        self.oims.insert(oim.message_id.clone(), oim);
    }
    
}