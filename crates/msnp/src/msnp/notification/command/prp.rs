use std::collections::VecDeque;
use std::fmt::Display;
use std::str::FromStr;
use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}};
use anyhow::anyhow;
use crate::shared::traits::{MSNPCommand, MSNPCommandPart};

pub struct Prp{
    tr_id: u128,
    operation: PrpOperation
}

pub type PrpClient = Prp;
pub type PrpServer = Prp;

pub enum PrpOperation {
    ModifyName {display_name: String}
}

impl Display for PrpOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModifyName{display_name} => {
                write!(f, "MFN {}", urlencoding::encode(display_name))
            }
        }
    }
}

impl MSNPCommandPart for PrpOperation {
    type Err = CommandError;

    fn try_from_split(mut split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> {
        let raw_operation = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "operation".into(), 2))?;

        match raw_operation.as_str() {
            "MFN" => {
                let raw_display_name = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "display_name".into(), 3))?;
                let display_name = urlencoding::decode(&raw_display_name)?.to_string();
                Ok(Self::ModifyName { display_name })
            },
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_operation.to_string(), command: command.to_string(), source: anyhow!("Unkown enum variant for PrpOperation") })
            }
        }

    }
}

impl MSNPCommand for Prp {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let operation = PrpOperation::try_from_split(split, &raw.command)?;

        Ok(Self { tr_id, operation })
    }

    fn to_bytes(self) -> Vec<u8> {
        todo!()
    }
}

impl Display for Prp {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRP {tr_id} {op}\r\n", tr_id = self.tr_id, op = self.operation )
    }

}