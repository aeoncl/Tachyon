
use std::{fmt::Display, str::{from_utf8, FromStr}};

use crate::{msnp::{error::{CommandError, PayloadError}, notification::models::endpoint_data::PrivateEndpointData, raw_command_parser::RawCommand}, shared::{command::command::{get_split_part, parse_tr_id, split_raw_command_no_arg, SerializeMsnp}, payload}};

pub struct Uux {
    tr_id : u128,
    payload_size: usize,
    payload: Option<UuxPayload>
}

pub type UuxClient = Uux;
pub type UuxServer = Uux;

impl Display for Uux {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUX {tr_id} {payload_size}\r\n", tr_id = self.tr_id, payload_size = self.payload_size)?;

        if let Some(payload) = &self.payload {
            write!(f, "{}", payload)?;
        }

        Ok(())
    }
}

impl TryFrom<RawCommand> for Uux {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {

        let split = split_raw_command_no_arg(command.get_command());
        let tr_id = parse_tr_id(&split)?;
        let payload_size = command.get_expected_payload_size();
        let payload = if payload_size > 0 { Some(UuxPayload::from_str(from_utf8(command.get_payload()).map_err(|e| PayloadError::Utf8Error(e))?)?) } else { None };

        Ok(Self{
            tr_id,
            payload_size,
            payload,
        })
    }
}

impl Uux {
    pub fn get_ok_response(&self) -> Uux {
        Uux {
            tr_id: self.tr_id,
            payload_size: 0,
            payload: None,
        }
    }
}


pub enum UuxPayload {
    PrivateEndpointData(PrivateEndpointData),
    Unknown(String)
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

impl SerializeMsnp for Uux {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
    
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;


    use crate::msnp::{error::{CommandError, PayloadError}, notification::{command::uux::{Uux, UuxPayload}, models::msnp_version::MsnpVersion}, raw_command_parser::RawCommand};

    use super::UuxClient;



    #[test]
    fn request_deserialization_unknown_success() {
        let uux = UuxClient::try_from(RawCommand::from_str("UUX 8 1\r\n").unwrap()).unwrap();

        assert_eq!(8, uux.tr_id);
        assert_eq!(1, uux.payload_size);
        assert!(matches!(uux.payload, Some(UuxPayload::Unknown(_))));

    }

    #[test]
    fn request_deserialization_private_endpoint_data_payload() {
        let payload = "<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>";

        

        let uux = UuxClient::try_from(RawCommand::with_payload(&format!("UUX 8 {}\r\n", payload.len()), payload.as_bytes().to_vec()).unwrap()).unwrap();

        assert_eq!(8, uux.tr_id);
        assert_eq!(payload.len(), uux.payload_size);
        assert!(matches!(uux.payload, Some(UuxPayload::PrivateEndpointData(_))));
    }

    #[test]
    fn request_deserialization_no_payload() {

        let uux = UuxClient::try_from(RawCommand::from_str("UUX 8 0\r\n").unwrap()).unwrap();

        assert_eq!(8, uux.tr_id);
        assert_eq!(0, uux.payload_size);
        assert!(matches!(uux.payload, None));
    }

    #[test]
    fn request_deserialization_bad_payload() {
        let payload = "<PrivateEndpointData><malformed";

        let uux = UuxClient::try_from(RawCommand::with_payload(&format!("UUX 8 {}\r\n", payload.len()), payload.as_bytes().to_vec()).unwrap());

        assert!(matches!(uux, Err(CommandError::PayloadError(PayloadError::StringPayloadParsingError { .. }))));
    }

    #[test]
    fn request_serialization_no_payload() {
        let uux = Uux { tr_id: 1, payload_size: 0, payload: None };
        let ser = uux.to_string();

        assert_eq!("UUX 1 0\r\n", ser);
    }

    #[test]
    fn request_serialization_unknown() {
        let uux = Uux { tr_id: 1, payload_size: 5, payload: Some(UuxPayload::Unknown("Hello".to_string())) };
        let ser = uux.to_string();

        assert_eq!("UUX 1 5\r\nHello", ser);
    }

 

}