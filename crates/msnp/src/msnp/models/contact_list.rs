use std::collections::HashMap;
use strum::IntoEnumIterator;
use crate::msnp::notification::command::adl::ADLPayload;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::msn_user::MsnUser;
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

pub struct Contact {
    user: MsnUser,
    memberships: u8
}

impl Contact {
    pub fn has_role(&self, role: RoleList) -> bool {
        self.memberships & role as u8 != 0
    }
}

impl ContactList {

    pub fn add_contacts(&mut self, mut contacts: HashMap<EmailAddress, u8>, is_initial: bool) {
        for (msn_addr, memberships ) in contacts.drain() {
            match self.contact_list.get_mut(&msn_addr) {
                None => {
                    self.contact_list.insert(msn_addr.clone(), Contact{ user: MsnUser::with_email_addr(msn_addr), memberships });
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
                    self.contact_list.insert(msn_addr.clone(), Contact{ user: MsnUser::with_email_addr(msn_addr), memberships });
                },
                Some(contact) => {
                        contact.memberships -= memberships;
                }
            };
        }
    }

    pub fn get_memberships(&self) -> HashMap<RoleList, Vec<MsnUser>> {
        let mut out = HashMap::new();

        for (_msn_addr, contact) in &self.contact_list {
            for role_id in RoleList::iter() {
                if contact.has_role(role_id.clone()) {
                    let list = out.entry(role_id).or_insert(Vec::new());
                    list.push(contact.user.clone());
                }
            }
        }
        out
    }

    pub fn get_forward_list(&self) -> Vec<MsnUser> {
        self.contact_list.iter().filter(|(k, v )| v.memberships & (RoleList::Forward as u8) != 0 ).map(|(k, v)| v.user.clone()).collect()
    }

}

