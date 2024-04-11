use std::collections::HashMap;
use strum::IntoEnumIterator;
use crate::msnp::notification::command::adl::ADLPayload;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::msn_user::MSNUser;
use crate::shared::models::role_id::RoleId;

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

pub struct Contact {
    user: MSNUser,
    memberships: u8
}

impl Contact {
    pub fn has_role(&self, role: RoleId) -> bool {
        self.memberships & role as u8 != 0
    }
}

impl ContactList {

    pub fn add_contacts(&mut self, mut contacts: HashMap<EmailAddress, u8>, is_initial: bool) {
        for (msn_addr, memberships ) in contacts.drain() {
            match self.contact_list.get_mut(&msn_addr) {
                None => {
                    self.contact_list.insert(msn_addr.clone(), Contact{ user: MSNUser::with_email_addr(msn_addr), memberships });
                },
                Some(contact) => {
                    if is_initial {
                        contact.memberships = memberships;
                    } else {
                        contact.memberships += memberships;
                    }
                }
            };
        }
    }

    pub fn remove_contacts(&mut self, mut contacts: HashMap<EmailAddress, u8>) {
        for (msn_addr, memberships ) in contacts.drain() {
            match self.contact_list.get_mut(&msn_addr) {
                None => {
                    self.contact_list.insert(msn_addr.clone(), Contact{ user: MSNUser::with_email_addr(msn_addr), memberships });
                },
                Some(contact) => {
                        contact.memberships -= memberships;
                }
            };
        }
    }

    pub fn get_memberships(&self) -> HashMap<RoleId, Vec<MSNUser>> {
        let mut out = HashMap::new();

        for (_msn_addr, contact) in &self.contact_list {
            for role_id in RoleId::iter() {
                if contact.has_role(role_id.clone()) {
                    let list = out.entry(role_id).or_insert(Vec::new());
                    list.push(contact.user.clone());
                }
            }
        }
        out
    }

    pub fn get_forward_list(&self) -> Vec<MSNUser> {
        self.contact_list.iter().filter(|(k, v )| v.memberships & (RoleId::Forward as u8) != 0 ).map(|(k, v)| v.user.clone()).collect()
    }

}

