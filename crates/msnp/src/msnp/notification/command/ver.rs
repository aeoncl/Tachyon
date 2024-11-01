use std::{fmt::Display, str::FromStr};

use crate::msnp::{error::CommandError, notification::models::msnp_version::MsnpVersion, raw_command_parser::RawCommand};
use crate::shared::traits::MSNPCommand;

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
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_first_candidate = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "first_candidate".into(), 2))?;
        let first_candidate = MsnpVersion::from_str(&raw_first_candidate)?;

        let raw_second_candidate = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "second_candidate".into(), 3))?;
        let second_candidate = MsnpVersion::from_str(&raw_second_candidate)?;

        let cvr = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "cvr".into(), 4))?;

        Ok(VerClient {tr_id, first_candidate, second_candidate, cvr})

    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for VerClient {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} {first_candidate} {second_candidate} {cvr}\r\n",operand = "VER", tr_id = self.tr_id, first_candidate = self.first_candidate, second_candidate = self.second_candidate, cvr = self.cvr)
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
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_agreed_version = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "agreed_version".into(), 2))?;
        let agreed_version = MsnpVersion::from_str(&raw_agreed_version)?;

        Ok(Self { tr_id, agreed_version })

    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for VerServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}\r\n", "VER", self.tr_id, self.agreed_version)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::notification::models::msnp_version::MsnpVersion;
    use crate::msnp::raw_command_parser::RawCommand;
    use crate::shared::traits::MSNPCommand;

    use super::{VerClient, VerServer};

    #[test]
    fn request_deserialization_success() {
        let request = VerClient::try_from_raw(RawCommand::from_str("VER 1 MSNP18 MSNP17 CVR0").unwrap()).unwrap();
        assert_eq!(request.tr_id, 1);
        assert_eq!(request.first_candidate, MsnpVersion::MSNP18);
        assert_eq!(request.second_candidate, MsnpVersion::MSNP17);
        assert_eq!(&request.cvr, "CVR0");
    }

    #[test]
    fn request_deserialization_invalid_params() {
        let request = VerClient::try_from_raw(RawCommand::from_str("VER 1 AOL18 MSNP17 CVR0").unwrap());
        assert!(request.is_err());
    }

    #[test]
    fn request_serialization() {
        let request = VerClient::try_from_raw(RawCommand::from_str("VER 1 MSNP17 MSNP17 CVR0").unwrap()).unwrap();
        assert_eq!(request.to_string().as_str(), "VER 1 MSNP17 MSNP17 CVR0\r\n");
    }

    #[test]
    fn response_deserialization_success() {
        let response = VerServer::try_from_raw(RawCommand::from_str("VER 1 MSNP18").unwrap()).unwrap();
        assert_eq!(response.tr_id, 1);
        assert_eq!(response.agreed_version, MsnpVersion::MSNP18);
    }

    #[test]
    fn response_serialization_success() {
        let response = VerServer::try_from_raw(RawCommand::from_str("VER 1 MSNP18").unwrap()).unwrap();
        assert_eq!(response.to_string().as_str(), "VER 1 MSNP18\r\n");
    }

}