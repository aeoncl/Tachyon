use std::str::FromStr;

use super::{error::CommandError, msnp_version::MsnpVersion};

pub struct VerCommand {
    pub tr_id: u128,
    pub first_candidate : MsnpVersion,
    pub second_candidate : MsnpVersion,
    pub cvr: String
}

impl FromStr for VerCommand {
    type Err = CommandError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = command.split_whitespace().collect::<Vec<&str>>();
        if split.len() != 5 {
            return Err(Self::Err::TooManyArguments {command: command.to_owned(), expected: 5, received: split.len() as u32 });
        }

        let tr_id_as_str = split.get(1).expect("tr_id to be present");
        let tr_id = u128::from_str(tr_id_as_str).map_err(|e| Self::Err::InvalidTrId{ tr_id: tr_id_as_str.to_string(), source: e } )?;
        
        let first_candidate_as_str = split.get(2).expect("first version candidate to be present");
        let first_candidate = MsnpVersion::from_str(first_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
    
        let second_candidate_as_str = split.get(3).expect("second version candidate to be present");
        let second_candidate = MsnpVersion::from_str(second_candidate_as_str).map_err(|e| Self::Err::ArgumentParseError { argument: first_candidate_as_str.to_string(), command: command.to_string(), source: e.into() })?;
        
        let cvr = split.get(4).expect("cvr to be present").to_string();

        Ok(VerCommand {tr_id, first_candidate, second_candidate, cvr})
    }
}