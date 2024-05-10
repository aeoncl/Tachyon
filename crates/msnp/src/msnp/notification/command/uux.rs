use std::{fmt::Display, str::FromStr};

use crate::msnp::{error::{CommandError, PayloadError}, notification::models::endpoint_data::PrivateEndpointData, raw_command_parser::RawCommand};
use crate::shared::traits::{MSNPCommand, MSNPPayload};

pub struct Uux {
    tr_id : u128,
    payload: Option<UuxPayload>
}

pub type UuxClient = Uux;
pub type UuxServer = Uux;

impl Display for Uux {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(payload) = &self.payload {
            let payload = payload.to_string();
            write!(f, "UUX {tr_id} {payload_size}\r\n{payload}", tr_id = self.tr_id, payload_size = 0, payload=payload)?;
        } else {
            write!(f, "UUX {tr_id} {payload_size}\r\n", tr_id = self.tr_id, payload_size = 0)?;
        }

        Ok(())
    }
}

impl MSNPCommand for Uux {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let payload_size = raw.expected_payload_size;

        let payload = if payload_size > 0 { Some(UuxPayload::try_from_bytes(raw.payload)?) } else { None };

        Ok(Self{
            tr_id,
            payload,
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl Uux {
    pub fn get_ok_response(&self) -> Uux {
        Uux {
            tr_id: self.tr_id,
            payload: None,
        }
    }
}


pub enum UuxPayload {
    PrivateEndpointData(PrivateEndpointData),
    Unknown(String)
}

impl MSNPPayload for UuxPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        let payload = String::from_utf8(bytes)?;
        UuxPayload::from_str(&payload)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl FromStr for UuxPayload {
    type Err = PayloadError;

    fn from_str(payload: &str) -> Result<Self, Self::Err> {
        if payload.starts_with("<PrivateEndpointData>") {
            Ok(Self::PrivateEndpointData(PrivateEndpointData::from_str(payload)?))
        } else {
            Ok(Self::Unknown(payload.to_string()))
        }
    }
}

impl Display for UuxPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            UuxPayload::PrivateEndpointData(payload) => {
                write!(f, "{}", payload)
            },
            UuxPayload::Unknown(payload) => {
                write!(f, "{}", payload)
            }
            
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::{error::{CommandError, PayloadError}, notification::command::uux::{Uux, UuxPayload}, raw_command_parser::RawCommand};
    use crate::shared::traits::MSNPCommand;

    use super::UuxClient;

    #[test]
    fn request_deserialization_unknown_success() {
        let uux = UuxClient::try_from_raw(RawCommand::from_str("UUX 8 1\r\n").unwrap()).unwrap();

        assert_eq!(8, uux.tr_id);
        assert!(matches!(uux.payload, Some(UuxPayload::Unknown(_))));

    }

    #[test]
    fn request_deserialization_private_endpoint_data_payload() {
        let payload = "<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>";

        

        let uux = UuxClient::try_from_raw(RawCommand::with_payload(&format!("UUX 8 {}\r\n", payload.len()), payload.as_bytes().to_vec())).unwrap();

        assert_eq!(8, uux.tr_id);
        assert!(matches!(uux.payload, Some(UuxPayload::PrivateEndpointData(_))));
    }

    #[test]
    fn request_deserialization_no_payload() {

        let uux = UuxClient::try_from_raw(RawCommand::from_str("UUX 8 0\r\n").unwrap()).unwrap();

        assert_eq!(8, uux.tr_id);
        assert!(matches!(uux.payload, None));
    }

    #[test]
    fn request_deserialization_bad_payload() {
        let payload = "<PrivateEndpointData><malformed";

        let uux = UuxClient::try_from_raw(RawCommand::with_payload(&format!("UUX 8 {}\r\n", payload.len()), payload.as_bytes().to_vec()));

        assert!(matches!(uux, Err(CommandError::PayloadError(PayloadError::StringPayloadParsingError { .. }))));
    }

    #[test]
    fn request_serialization_no_payload() {
        let uux = Uux { tr_id: 1, payload: None };
        let ser = uux.to_string();

        assert_eq!("UUX 1 0\r\n", ser);
    }

    #[test]
    fn request_serialization_unknown() {
        let uux = Uux { tr_id: 1, payload: Some(UuxPayload::Unknown("Hello".to_string())) };
        let ser = uux.to_string();

        assert_eq!("UUX 1 5\r\nHello", ser);
    }

 

}