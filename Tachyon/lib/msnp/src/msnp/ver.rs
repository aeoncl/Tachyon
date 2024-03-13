use std::{fmt::Display, str::FromStr};

use super::{command::{parse_tr_id, split_raw_command, MSNPCommand}, error::CommandError, msnp_version::MsnpVersion, raw_command_parser::RawCommand};

pub struct VerRequest {
    pub tr_id: u128,
    pub first_candidate : MsnpVersion,
    pub second_candidate : MsnpVersion,
    pub cvr: String
}

impl VerRequest {

    pub fn new(tr_id: u128, first_candidate : MsnpVersion, second_candidate : MsnpVersion) -> Self {
        Self {
            tr_id,
            first_candidate,
            second_candidate,
            cvr: "CVR0".to_string()
        }
    }

    pub fn get_response_for(&self, agreed_version: MsnpVersion) -> VerResponse {
        VerResponse::new(self.tr_id, agreed_version)
    }
}

impl MSNPCommand for VerRequest {

    fn get_operand(&self) -> &str {
        "VER"
    }
}

impl TryFrom<RawCommand> for VerRequest {
    type Error = CommandError;

    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {
        VerRequest::from_str(value.command.as_str())
    }
}

impl FromStr for VerRequest {
    type Err = CommandError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = split_raw_command(command, 5)?;
        let tr_id = parse_tr_id(&split)?;
        
        let first_candidate_as_str = split.get(2).expect("first version candidate to be present");
        let first_candidate = MsnpVersion::from_str(first_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
    
        let second_candidate_as_str = split.get(3).expect("second version candidate to be present");
        let second_candidate = MsnpVersion::from_str(second_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
        
        let cvr = split.get(4).expect("cvr to be present").to_string();

        Ok(VerRequest {tr_id, first_candidate, second_candidate, cvr})
    }
}

impl Display for VerRequest {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} {first_candidate} {second_candidate} {cvr}\r\n",operand = self.get_operand(), tr_id = self.tr_id, first_candidate = self.first_candidate, second_candidate = self.second_candidate, cvr = self.cvr)
    }
}

pub struct VerResponse {
    pub tr_id: u128,
    pub agreed_version: MsnpVersion
}

impl VerResponse {

    pub fn new(tr_id: u128, agreed_version: MsnpVersion) -> Self {
        Self {
            tr_id,
            agreed_version
        }
    }
}

impl MSNPCommand for VerResponse {

    fn get_operand(&self) -> &str {
        "VER"
    }
}

impl FromStr for VerResponse {
    type Err = CommandError;
    
    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = split_raw_command(command, 3)?;
        let tr_id = parse_tr_id(&split)?;

        let agreed_version_as_str = split.get(2).expect("agreed_version to be present");
        let agreed_version = MsnpVersion::from_str(agreed_version_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: agreed_version_as_str.to_string(), command: command.to_string(), source: e.into() })?;
        
        Ok(Self { tr_id, agreed_version })
    }


}

impl Display for VerResponse {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}\r\n", self.get_operand(), self.tr_id, self.agreed_version)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::msnp_version::MsnpVersion;

    use super::{VerRequest, VerResponse};


    #[test]
    fn request_deserialization_success() {
        let request = VerRequest::from_str("VER 1 MSNP18 MSNP17 CVR0").unwrap();
        assert_eq!(request.tr_id, 1);
        assert_eq!(request.first_candidate, MsnpVersion::MSNP18);
        assert_eq!(request.second_candidate, MsnpVersion::MSNP17);
        assert_eq!(&request.cvr, "CVR0");
    }

    #[test]
    fn request_deserialization_invalid_params() {
        let request = VerRequest::from_str("VER 1 AOL18 MSNP17 CVR0");
        assert!(request.is_err());
    }

    #[test]
    fn request_serialization() {
        let request = VerRequest::from_str("VER 1 MSNP17 MSNP17 CVR0").unwrap();
        assert_eq!(request.to_string().as_str(), "VER 1 MSNP17 MSNP17 CVR0\r\n");
    }

    #[test]
    fn response_deserialization_success() {
        let response = VerResponse::from_str("VER 1 MSNP18").unwrap();
        assert_eq!(response.tr_id, 1);
        assert_eq!(response.agreed_version, MsnpVersion::MSNP18);
    }

    #[test]
    fn response_serialization_success() {
        let response = VerResponse::from_str("VER 1 MSNP18").unwrap();
        assert_eq!(response.to_string().as_str(), "VER 1 MSNP18\r\n");
    }

}