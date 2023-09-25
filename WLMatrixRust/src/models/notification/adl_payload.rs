use std::{fmt::Display, str::FromStr};

use substring::Substring;
use yaserde::{de::from_str, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{generated::msnab_datatypes::types::RoleId, models::errors::Errors};

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
    pub fn get_list_types(&self) -> Vec<RoleId> {
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
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      if let Ok(deserialized) = from_str::<ADLPayload>(s) {
        return Ok(deserialized);
      } else {
          return Err(Errors::PayloadDeserializeError);
      }
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

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
    fn test_get_list_types() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"3\" t=\"1\"/><c n=\"facebookbot1\" l=\"1\" t=\"1\"/></d></ml>").unwrap();
        let contact = payload.domains.first().unwrap().contacts.first().unwrap();

        let lists = contact.get_list_types();
        println!("{:?}", &lists);
    }

    #[test]
    fn test_serialize() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"1\" t=\"1\"/></d></ml>").unwrap();
        let serialized = payload.to_string();

        assert_eq!(serialized.as_str(), "<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\" /><c n=\"facebookbot1\" l=\"1\" t=\"1\" /></d></ml>")

    }

}
