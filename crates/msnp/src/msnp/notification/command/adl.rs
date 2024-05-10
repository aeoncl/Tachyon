use std::{fmt::Display, str::{from_utf8, FromStr}};
use std::collections::HashMap;

use anyhow::anyhow;
use yaserde::{de::from_str, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{msnp::{error::{CommandError, PayloadError}, raw_command_parser::RawCommand}, shared::{command::ok::OkCommand, models::role_list::RoleList}};
use crate::shared::models::email_address::EmailAddress;
use crate::shared::traits::{MSNPCommand, MSNPPayload};
use crate::shared::errors::IdentifierError;

pub struct AdlClient {
    tr_id: u128,
    payload: ADLPayload
}

pub type RmlClient = AdlClient;

impl AdlClient {
    pub fn get_ok_response(&self, operand: &str) -> OkCommand {
        OkCommand { tr_id: self.tr_id, operand: operand.to_string() }
    }
}


impl MSNPCommand for AdlClient {

    type Err = CommandError;

    fn try_from_raw(command: RawCommand) -> Result<Self, Self::Err> {
        let mut split = command.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(command.command.clone(), "tr_id".into(), 1))?;

        let tr_id = u128::from_str(&raw_tr_id)?;

        let payload_size = command.expected_payload_size;

        if payload_size == 0 {
            Err(PayloadError::MissingPayload { command: command.command })?;
        }

        let payload = ADLPayload::try_from_bytes(command.payload)?;

        Ok(Self{
            tr_id,
            payload,
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ml")]
pub struct ADLPayload {

    #[yaserde(rename = "l", attribute)]
    pub l: Option<u8>,

    #[yaserde(rename = "d")]
    pub domains: Vec<ADLDomain>

}

impl ADLPayload {

    pub fn get_contacts(&self) -> Result<HashMap<EmailAddress, u8>, IdentifierError> {
        let mut out = HashMap::new();
        for domain in &self.domains {
            out.extend(domain.get_contacts_and_roles()?);
        }
        Ok(out)
    }

}


#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
pub struct ADLDomain {

    #[yaserde(rename = "n", attribute)]
    pub domain: String,

    #[yaserde(rename = "c")]
    pub contacts: Vec<ADLContact>
}

impl ADLDomain {
    pub fn get_contacts_and_roles(&self) -> Result<HashMap<EmailAddress, u8>, IdentifierError> {
        let test : Result<HashMap<EmailAddress, u8>, IdentifierError> = self.contacts.iter().map(|c| Ok((c.get_msn_addr(&self.domain)?, c.list_type))).collect();
        test
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
    pub fn get_msn_addr(&self, domain: &str) -> Result<EmailAddress, IdentifierError> {
        EmailAddress::from_str(&format!("{}@{}", &self.email_part, domain))
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

impl MSNPPayload for ADLPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        let payload = String::from_utf8(bytes)?;
        Self::from_str(&payload)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl FromStr for ADLPayload {
    type Err = PayloadError;

    fn from_str(payload: &str) -> Result<Self, Self::Err> {
        from_str::<ADLPayload>(&payload).map_err(|e| PayloadError::StringPayloadParsingError { payload: payload.to_string(), source: anyhow!("Couldn't deserialize ADL Payload: - error: {}", e) })
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

    use crate::shared::models::role_list::RoleList;

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

    // #[test]
    // fn test_domain_get_contacts() {
    //     let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"3\" t=\"1\"/></d></ml>").unwrap();
    //
    //     let first_domain=payload.domains.get(0).unwrap();
    //     let contacts_of_domain = first_domain.get_contacts();
    //     assert_eq!(contacts_of_domain.len(), 2usize);
    //
    //     first_domain.get_contacts_for_role(RoleId::Forward);
    //
    //     let result = payload.get_contacts_for_role(RoleId::Forward);
    //     assert_eq!(result.len(), 2usize);
    //     let test = 0;
    // }

    #[test]
    fn test_serialize() {
        let payload = ADLPayload::from_str("<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\"/><c n=\"facebookbot1\" l=\"1\" t=\"1\"/></d></ml>").unwrap();
        let serialized = payload.to_string();

        assert_eq!(serialized.as_str(), "<ml><d n=\"shlasouf.local\"><c n=\"facebookbot\" l=\"1\" t=\"1\" /><c n=\"facebookbot1\" l=\"1\" t=\"1\" /></d></ml>")

    }

}