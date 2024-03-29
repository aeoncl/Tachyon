use std::{fmt::Display, str::FromStr};
use anyhow::anyhow;

use substring::Substring;
use yaserde::{de::from_str, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{generated::msnab_datatypes::types::RoleId};
use crate::models::msn_user::PartialMSNUser;
use crate::models::tachyon_error::PayloadError;

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ml")]
pub struct ADLPayload {

    #[yaserde(rename = "l", attribute)]
    pub l: Option<u8>,

    #[yaserde(rename = "d")]
    pub domains: Vec<ADLDomain>

}

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
pub struct ADLDomain {

    #[yaserde(rename = "n", attribute)]
    pub domain: String,

    #[yaserde(rename = "c")]
    pub contacts: Vec<ADLContact>
}

impl ADLDomain {
    pub fn get_contacts(&self) -> Vec<PartialMSNUser> {
        return self.contacts.iter().map(|c| c.to_partial_msn_user(&self.domain)).collect();
    }

    pub fn get_contacts_for_role(&self, role: RoleId) -> Vec<PartialMSNUser> {
       return self.contacts.iter().filter(|c| c.has_role(role.clone())).map(|c| c.to_partial_msn_user(&self.domain)).collect();
    }
}

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
pub struct ADLContact {

    #[yaserde(rename = "n", attribute)]
    pub email_part: String,

    #[yaserde(rename = "l", attribute)]
    pub list_type: u8,

    #[yaserde(rename = "t", attribute)]
    pub contact_type: String
}

impl ADLContact {
    pub fn has_role(&self, role: RoleId) -> bool {
        let test = self.list_type & role as u8;
        test != 0
    }

    pub fn get_roles(&self) -> Vec<RoleId> {
        let mut out = Vec::new();

        if self.list_type & RoleId::Forward as u8 != 0 {
            out.push(RoleId::Forward);
        }

        if self.list_type & RoleId::Allow as u8 != 0 {
            out.push(RoleId::Allow);
        }

        if self.list_type & RoleId::Block as u8 != 0 {
            out.push(RoleId::Block);
        }

        if self.list_type & RoleId::Reverse as u8 != 0 {
            out.push(RoleId::Reverse);
        }

        if self.list_type & RoleId::Pending as u8 != 0 {
            out.push(RoleId::Pending);
        }

        return out;
    }

    pub fn to_partial_msn_user(&self, domain: &str) -> PartialMSNUser {
        PartialMSNUser::new(format!("{}@{}", &self.email_part, domain))
    }
}


impl Display for ADLPayload {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {


        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        if let Ok(serialized) = to_string_with_config(self, &yaserde_cfg) {
            return write!(f, "{}", serialized.as_str());
        } else {
            return Err(std::fmt::Error);
        }
    }
}

impl FromStr for ADLPayload {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
         from_str::<ADLPayload>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), sauce: anyhow!("Couldn't deserialize ADL Payload: {} - error: {}",s, e) })
    }
}

impl ADLPayload {

    pub fn is_initial(&self) -> bool {
        if self.l.is_none() {
            return false;
        } 

        let l = self.l.unwrap();
        return l == 1;
    }

    pub fn get_contacts_for_role(&self, role_id: RoleId) -> Vec<PartialMSNUser> {
        self.domains.iter().flat_map(|e| e.get_contacts_for_role(role_id.clone())).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::generated::msnab_datatypes::types::RoleId;

    use super::ADLPayload;

    #[test]
    fn test_deserialize() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"1\" t=\"1\"/></d></ml>").unwrap();
        
        println!("debug: {:?}", &payload);

        assert_eq!(payload.l, None);
        assert!(payload.domains.len() == 1);

        let first_domain = payload.domains.first().unwrap();
        
        assert_eq!(first_domain.domain.as_str(), "shlasouf.local");
        assert!(first_domain.contacts.len() > 0);

        let first_contact = first_domain.contacts.get(0).unwrap();
        assert_eq!(first_contact.email_part.as_str(), "facebookbot");

        let second_contact = first_domain.contacts.get(1).unwrap();
        assert_eq!(second_contact.email_part.as_str(), "facebookbot1");
    }

    #[test]
    fn test_domain_get_contacts() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"3\" t=\"1\"/></d></ml>").unwrap();

        let first_domain=payload.domains.get(0).unwrap();
        let contacts_of_domain = first_domain.get_contacts();
        assert_eq!(contacts_of_domain.len(), 2usize);

        first_domain.get_contacts_for_role(RoleId::Forward);

        let result = payload.get_contacts_for_role(RoleId::Forward);
        assert_eq!(result.len(), 2usize);
        let test = 0;
    }

    #[test]
    fn test_serialize() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"1\" t=\"1\"/></d></ml>").unwrap();
        let serialized = payload.to_string();

        assert_eq!(serialized.as_str(), "<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\" /><c n=\"facebookbot1\" l=\"1\" t=\"1\" /></d></ml>")

    }

}
