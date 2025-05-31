use strum::IntoEnumIterator;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::network_id::NetworkId;
use crate::shared::models::network_id_email::NetworkIdEmail;
use crate::shared::models::role_list::RoleList;

#[derive(Debug, Clone)]
pub struct Contact {

    pub email_address: EmailAddress,
    pub network_id: NetworkId,
    pub memberships: u8,
}

impl Contact {

    pub fn new(email_address: EmailAddress, network_id: NetworkId, memberships: u8) -> Self {
        Contact {
            email_address,
            network_id,
            memberships
        }

    }
    pub fn has_role(&self, role: RoleList) -> bool {
        self.memberships & role as u8 != 0
    }

    pub fn get_roles(&self) -> Vec<RoleList> {
        RoleList::iter().filter(|role| self.has_role(role.clone())).collect()
    }

    pub fn is_from_network(&self, network_id: NetworkId) -> bool {
        self.network_id == network_id
    }

    pub fn get_network_id_email(&self) -> NetworkIdEmail {
        NetworkIdEmail::new(self.network_id.clone(), self.email_address.clone())
    }

    pub fn get_email_addr(&self) -> EmailAddress {
        self.email_address.clone()
    }
}