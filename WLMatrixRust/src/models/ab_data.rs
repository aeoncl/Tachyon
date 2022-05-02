use std::{sync::Mutex, collections::HashMap};
use crate::generated::msnab_datatypes::types::{ContactType, BaseMember, RoleId};

pub struct AbData {

    contact_list : Mutex<HashMap<String, ContactType>>,
    /* MessengerService */
    allow_list : Mutex<HashMap<String, BaseMember>>,
    reverse_list : Mutex<HashMap<String, BaseMember>>,
    block_list : Mutex<HashMap<String, BaseMember>>,
    pending_list : Mutex<HashMap<String, BaseMember>>,
}

impl AbData {

    pub fn new() -> Self {
        return AbData { contact_list: Mutex::new(HashMap::new()), allow_list: Mutex::new(HashMap::new()), reverse_list: Mutex::new(HashMap::new()), block_list: Mutex::new(HashMap::new()), pending_list: Mutex::new(HashMap::new()) };
    }

    pub fn has_data(&self) -> bool {
       return !self.contact_list.lock().unwrap().is_empty() || !self.allow_list.lock().unwrap().is_empty() || !self.reverse_list.lock().unwrap().is_empty() || !self.block_list.lock().unwrap().is_empty() || !self.pending_list.lock().unwrap().is_empty();
    }

    pub fn consume_contact_list(&mut self) -> Vec<ContactType> { 
        let mut out : Vec<ContactType> = Vec::new();

        let mut guarded_contact_list = self.contact_list.lock().unwrap();
        for current in guarded_contact_list.values() {
            out.push(current.clone());
        }
        guarded_contact_list.clear();
        return out;
    }

    pub fn add_to_contact_list(&self, mtx_id: String, contact: ContactType) {
        self.contact_list.lock().unwrap().insert(mtx_id, contact);
    }

    pub fn add_to_messenger_service(&self, mtx_id: String, member: BaseMember, role_id: RoleId) {
        match role_id {
            RoleId::Allow => {
                self.allow_list.lock().unwrap().insert(mtx_id, member);
            },
            RoleId::Block => {
                self.block_list.lock().unwrap().insert(mtx_id, member);
            },
            RoleId::Pending => {
                self.pending_list.lock().unwrap().insert(mtx_id, member);
            },
            RoleId::Reverse => {
                self.reverse_list.lock().unwrap().insert(mtx_id, member);
            }
        }
    }

    pub fn consume_messenger_service(&mut self) -> (Vec<BaseMember>, Vec<BaseMember>, Vec<BaseMember>, Vec<BaseMember>){
        let allow_list = self.consume_member_list(RoleId::Allow);
        let reverse_list = self.consume_member_list(RoleId::Reverse);
        let block_list = self.consume_member_list(RoleId::Block);
        let pending_list = self.consume_member_list(RoleId::Pending);
        return (allow_list, reverse_list, block_list, pending_list);
    }

    fn consume_member_list(&mut self, role_id: RoleId) -> Vec<BaseMember> {

        let mut list;

        match role_id {
            RoleId::Allow => {
                list = self.allow_list.lock().unwrap();
            },
            RoleId::Block => {
                list = self.block_list.lock().unwrap();
            },
            RoleId::Pending => {
                list = self.pending_list.lock().unwrap();
            },
            RoleId::Reverse => {
                list = self.reverse_list.lock().unwrap();
            }
        }

        let mut out : Vec<BaseMember> = Vec::new();

        for current in list.values() {
            out.push(current.clone());
        }
        list.clear();

       return out;
    }
}