use std::fmt::Display;
use std::str::FromStr;
use yaserde_derive::{YaDeserialize, YaSerialize};
use yaserde::de::from_str;
use anyhow::anyhow;
use yaserde::ser::to_string_with_config;
use crate::msnp::error::PayloadError;
use crate::msnp::models::contact::Contact;
use crate::shared::errors::IdentifierError;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::network_id::NetworkId;
use crate::shared::traits::{IntoBytes, TryFromBytes};

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ml")]
pub struct ADLPayload {

    #[yaserde(rename = "l", attribute)]
    pub l: Option<u8>,

    #[yaserde(rename = "d")]
    pub domains: Vec<ADLDomain>

}

impl ADLPayload {

    pub fn get_contacts(&self) -> Result<Vec<Contact>, IdentifierError> {
        let mut out = Vec::new();
        for domain in &self.domains {
            out.extend(domain.get_contacts()?);
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
    pub fn get_contacts(&self) -> Result<Vec<Contact>, IdentifierError> {
        let test : Result<Vec<Contact>, IdentifierError> = self.contacts.iter().map(|c| Ok(c.get_contact(&self.domain)?)).collect();
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
    pub contact_type: NetworkId,

    #[yaserde(rename = "actual", attribute)]
    pub actual: Option<String>
}

impl ADLContact {
    pub fn get_contact(&self, domain: &str) -> Result<Contact, IdentifierError> {
        Ok(Contact::new(EmailAddress::from_str(&format!("{}@{}", &self.email_part, domain))?, self.contact_type.clone(), self.list_type))
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

impl TryFromBytes for ADLPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        let payload = String::from_utf8(bytes)?;
        Self::from_str(&payload)
    }
}

impl IntoBytes for ADLPayload {
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