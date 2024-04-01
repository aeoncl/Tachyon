use std::{fmt::Display, str::FromStr};

use crate::{msnp::{error::CommandError, notification::models::msnp_version::MsnpVersion, raw_command_parser::RawCommand}, shared::command::command::{parse_tr_id, split_raw_command, MSNPCommand, SerializeMsnp}};


pub struct VerClient {
    pub tr_id: u128,
    pub first_candidate : MsnpVersion,
    pub second_candidate : MsnpVersion,
    pub cvr: String
}

impl VerClient {

    pub fn new(tr_id: u128, first_candidate : MsnpVersion, second_candidate : MsnpVersion) -> Self {
        Self {
            tr_id,
            first_candidate,
            second_candidate,
            cvr: "CVR0".to_string()
        }
    }

    pub fn get_response_for(&self, agreed_version: MsnpVersion) -> VerServer {
        VerServer::new(self.tr_id, agreed_version)
    }
}

impl MSNPCommand for VerClient {

    fn get_operand(&self) -> &str {
        "VER"
    }
}

impl TryFrom<RawCommand> for VerClient {
    type Error = CommandError;

    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {
        VerClient::from_str(value.get_command())
    }
}

impl FromStr for VerClient {
    type Err = CommandError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = split_raw_command(command, 5)?;
        let tr_id = parse_tr_id(&split)?;
        
        let first_candidate_as_str = split.get(2).expect("first version candidate to be present");
        let first_candidate = MsnpVersion::from_str(first_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
    
        let second_candidate_as_str = split.get(3).expect("second version candidate to be present");
        let second_candidate = MsnpVersion::from_str(second_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
        
        let cvr = split.get(4).expect("cvr to be present").to_string();

        Ok(VerClient {tr_id, first_candidate, second_candidate, cvr})
    }
}

impl Display for VerClient {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} {first_candidate} {second_candidate} {cvr}\r\n",operand = self.get_operand(), tr_id = self.tr_id, first_candidate = self.first_candidate, second_candidate = self.second_candidate, cvr = self.cvr)
    }
}

impl SerializeMsnp for VerClient {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

pub struct VerServer {
    pub tr_id: u128,
    pub agreed_version: MsnpVersion
}

impl VerServer {

    pub fn new(tr_id: u128, agreed_version: MsnpVersion) -> Self {
        Self {
            tr_id,
            agreed_version
        }
    }
}

impl MSNPCommand for VerServer {

    fn get_operand(&self) -> &str {
        "VER"
    }
}

impl FromStr for VerServer {
    type Err = CommandError;
    
    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = split_raw_command(command, 3)?;
        let tr_id = parse_tr_id(&split)?;

        let agreed_version_as_str = split.get(2).expect("agreed_version to be present");
        let agreed_version = MsnpVersion::from_str(agreed_version_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: agreed_version_as_str.to_string(), command: command.to_string(), source: e.into() })?;
        
        Ok(Self { tr_id, agreed_version })
    }


}

impl Display for VerServer {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}\r\n", self.get_operand(), self.tr_id, self.agreed_version)
    }
}

impl SerializeMsnp for VerServer {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;


    use crate::msnp::notification::models::msnp_version::MsnpVersion;

    use super::{VerClient, VerServer};


    #[test]
    fn request_deserialization_success() {
        let request = VerClient::from_str("VER 1 MSNP18 MSNP17 CVR0").unwrap();
        assert_eq!(request.tr_id, 1);
        assert_eq!(request.first_candidate, MsnpVersion::MSNP18);
        assert_eq!(request.second_candidate, MsnpVersion::MSNP17);
        assert_eq!(&request.cvr, "CVR0");
    }

    #[test]
    fn request_deserialization_invalid_params() {
        let request = VerClient::from_str("VER 1 AOL18 MSNP17 CVR0");
        assert!(request.is_err());
    }

    #[test]
    fn request_serialization() {
        let request = VerClient::from_str("VER 1 MSNP17 MSNP17 CVR0").unwrap();
        assert_eq!(request.to_string().as_str(), "VER 1 MSNP17 MSNP17 CVR0\r\n");
    }

    #[test]
    fn response_deserialization_success() {
        let response = VerServer::from_str("VER 1 MSNP18").unwrap();
        assert_eq!(response.tr_id, 1);
        assert_eq!(response.agreed_version, MsnpVersion::MSNP18);
    }

    #[test]
    fn response_serialization_success() {
        let response = VerServer::from_str("VER 1 MSNP18").unwrap();
        assert_eq!(response.to_string().as_str(), "VER 1 MSNP18\r\n");
    }

}