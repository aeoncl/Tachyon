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

impl IntoBytes for AdlClient {
    fn into_bytes(self) -> Vec<u8> {
        self.into_bytes_with_operand("ADL")
    }
}

impl AdlClient {
    pub(crate) fn into_bytes_with_operand(self, operand: &str) -> Vec<u8> {
        let payload = self.payload.to_string();
        format!("{operand} {tr_id} {len}\r\n{payload}", tr_id = self.tr_id, len = payload.len(), payload = payload).into_bytes()
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



    use crate::msnp::raw_command_parser::RawCommandParser;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::AdlClient;

    #[test]
    fn serializes_the_payload_length_and_round_trips() {
        let adl = AdlClient {
            tr_id: 7,
            payload: ADLPayload::from_str("<ml l=\"1\"></ml>").unwrap(),
        };

        let bytes = adl.into_bytes();
        assert_eq!(String::from_utf8(bytes.clone()).unwrap(), "ADL 7 12\r\n<ml l=\"1\" />");

        let mut parser = RawCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = AdlClient::try_from_raw(raw).unwrap();

        assert_eq!(parsed.tr_id, 7);
        assert_eq!(parsed.payload.to_string(), "<ml l=\"1\" />");
    }
}
