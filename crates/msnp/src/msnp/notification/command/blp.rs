
use std::str::FromStr;

use strum_macros::{Display, EnumString};
use std::fmt::Display;
use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}};
use crate::shared::traits::{MSNPCommand};

pub struct Blp{
    tr_id: u128,
    list_type: ListType
}

pub type BlpClient = Blp;

pub type BlpServer = Blp;


impl MSNPCommand for Blp {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;

        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_list_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "list_type".into(), 2))?;

        let list_type = ListType::from_str(&raw_list_type)?;
        Ok(Self { tr_id, list_type })    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[derive(Display, EnumString)]
pub enum ListType {
    #[strum(serialize = "AL")]
    AllowList,
    #[strum(serialize = "BL")]
    BlockList
}

impl Display for Blp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BLP {} {}\r\n", self.tr_id, self.list_type)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::{notification::command::blp::ListType, raw_command_parser::RawCommand};
    use crate::shared::traits::MSNPCommand;

    use super::Blp;

    
    #[test]
    fn test_deserialize_al() {
        let blp = Blp::try_from_raw(RawCommand::from_str("BLP 1 AL\r\n").unwrap()).unwrap();

        assert_eq!(1, blp.tr_id);
        assert!(matches!(blp.list_type, ListType::AllowList));

    }

    #[test]
    fn test_deserialize_bl() {
        let blp = Blp::try_from_raw(RawCommand::from_str("BLP 1 BL\r\n").unwrap()).unwrap();

        assert_eq!(1, blp.tr_id);
        assert!(matches!(blp.list_type, ListType::BlockList));

    }

    #[test]
    fn test_serialize() {
        let blp = Blp{ tr_id: 1, list_type: ListType::AllowList };
        let ser = blp.to_string();
        assert_eq!("BLP 1 AL\r\n", ser.as_str());
    }
}

