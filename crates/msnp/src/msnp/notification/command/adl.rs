use std::{fmt::Display, str::FromStr};

use crate::{msnp::{error::{CommandError, PayloadError}, raw_command_parser::RawCommand}, shared::command::ok::OkCommand};
use crate::msnp::notification::models::adl_payload::ADLPayload;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand};
#[derive(Debug)]
pub struct AdlClient {
    pub tr_id: u128,
    pub payload: ADLPayload
}

pub type RmlClient = AdlClient;

impl AdlClient {
    pub fn get_ok_response(&self, operand: &str) -> OkCommand {
        OkCommand { tr_id: self.tr_id, operand: operand.to_string() }
    }
}


impl TryFromRawCommand for AdlClient {

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
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::notification::models::adl_payload::ADLPayload;

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