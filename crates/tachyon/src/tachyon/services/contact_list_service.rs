use std::sync::{Arc, Mutex};

use msnp::msnp::models::contact::Contact;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::uuid::Uuid;

pub trait ContactListService: Send + Sync {
    fn add(&self, contacts: Vec<Contact>, is_initial: bool);

    fn remove(&self, contacts: Vec<Contact>);

    fn get_forward_list(&self) -> Vec<Contact>;

    fn find_by_uuid(&self, uuid: &Uuid) -> Option<Contact>;

    fn contains(&self, email: &EmailAddress) -> bool;
}

pub struct ContactListServiceImpl {
    contact_list: Arc<Mutex<msnp::msnp::models::contact_list::ContactList>>,
}

impl ContactListServiceImpl {
    pub fn new(contact_list: Arc<Mutex<msnp::msnp::models::contact_list::ContactList>>) -> Self {
        Self { contact_list }
    }
}

impl ContactListService for ContactListServiceImpl {
    fn add(&self, contacts: Vec<Contact>, is_initial: bool) {
        let mut list = self.contact_list.lock().expect("contact_list lock");
        list.add_contacts(contacts, is_initial);
    }

    fn remove(&self, contacts: Vec<Contact>) {
        let mut list = self.contact_list.lock().expect("contact_list lock");
        list.remove_contacts(contacts);
    }

    fn get_forward_list(&self) -> Vec<Contact> {
        let list = self.contact_list.lock().expect("contact_list lock");
        list.get_forward_list()
    }

    fn find_by_uuid(&self, uuid: &Uuid) -> Option<Contact> {
        let list = self.contact_list.lock().expect("contact_list lock");
        list.find_contact_by_uuid(uuid).cloned()
    }

    fn contains(&self, email: &EmailAddress) -> bool {
        let list = self.contact_list.lock().expect("contact_list lock");
        list.get_contact(email).is_some()
    }
}
