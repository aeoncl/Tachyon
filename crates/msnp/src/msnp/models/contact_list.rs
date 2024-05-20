use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::msnp::models::contact::Contact;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::role_list::RoleList;

pub struct ContactList {
    pub contact_list: HashMap<EmailAddress, Contact>,
}

impl Default for ContactList {
    fn default() -> Self {
       Self{
           contact_list: HashMap::default(),
       }
    }
}

impl ContactList {

    pub fn add_contacts(&mut self, mut contacts: Vec<Contact>, is_initial: bool) {
        for current in contacts.drain(..) {
            match self.contact_list.get_mut(&current.email_address) {
                None => {
                    self.contact_list.insert(current.email_address.clone(), current);
                },
                Some(contact) => {
                    if is_initial {
                        contact.memberships = current.memberships;
                    } else {
                        contact.memberships += current.memberships;
                    }
                }
            };
        }
    }

    pub fn remove_contacts(&mut self, mut contacts: Vec<Contact>) {
        for current in contacts.drain(..) {
            match self.contact_list.get_mut(&current.email_address) {
                None => {
                },
                Some(contact) => {
                        contact.memberships -= current.memberships;
                        if contact.memberships == 0 {
                            self.contact_list.remove(&current.email_address);
                        }
                }
            };
        }
    }

    pub fn get_memberships(&self) -> HashMap<RoleList, Vec<&Contact>> {
        let mut out = HashMap::new();

        for (_msn_addr, contact) in &self.contact_list {
            for role_id in RoleList::iter() {
                if contact.has_role(role_id.clone()) {
                    let list = out.entry(role_id).or_insert(Vec::new());
                    list.push(contact);
                }
            }
        }
        out
    }

    pub fn get_forward_list(&self) -> Vec<Contact> {
        self.contact_list.iter().filter(|(k, v )| v.memberships & (RoleList::Forward as u8) != 0 ).map(|(k, v)| v.clone()).collect()
    }

}

