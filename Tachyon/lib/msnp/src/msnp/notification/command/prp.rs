use std::fmt::Display;
use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}, shared::command::command::{get_split_part, parse_tr_id}};
use anyhow::anyhow;
use crate::shared::traits::SerializeMsnp;

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

impl TryFrom<RawCommand> for PrpOperation {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let raw_operation = get_split_part(2, &split, command.get_command(), "operation")?;

        match raw_operation {
            "MFN" => {
                let raw_display_name = get_split_part(3, &split, command.get_command(), "display_name")?;
                let display_name = urlencoding::decode(raw_display_name).map_err(|e| CommandError::ArgumentParseError { argument: raw_display_name.to_string(), command: command.get_command().to_string(), source: e.into() })?.to_string();
                Ok(Self::ModifyName { display_name })
            },
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_operation.to_string(), command: command.get_command().to_string(), source: anyhow!("Unkown enum variant for PrpOperation") })
            }
        }
        
    }
}


impl TryFrom<RawCommand> for Prp {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {

        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let operation = PrpOperation::try_from(command)?;
        
        Ok(Self { tr_id, operation })
    }
}

impl Display for Prp {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRP {tr_id} {op}\r\n", tr_id = self.tr_id, op = self.operation )
    }

}

impl SerializeMsnp for Prp {
    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}