
use std::str::FromStr;

use strum_macros::{Display,EnumString};
use std::fmt::Display;
use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}, shared::command::command::{get_split_part, parse_tr_id, SerializeMsnp}};

pub struct Blp{
    tr_id: u128,
    list_type: ListType
}

pub type BlpClient = Blp;

pub type BlpServer = Blp;

impl TryFrom<RawCommand> for Blp {
    
    type Error = CommandError;
    
    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let raw_list_type = get_split_part(2, &split, command.get_command(), "list_type")?;
        let list_type = ListType::from_str(raw_list_type).map_err(|e| CommandError::ArgumentParseError { argument: raw_list_type.to_string(), command: command.get_command().to_string(), source: e.into() } )?;
        Ok(Self { tr_id, list_type })
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

impl SerializeMsnp for Blp {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::{notification::command::blp::ListType, raw_command_parser::RawCommand};

    use super::Blp;

    
    #[test]
    fn test_deserialize_al() {
        let blp = Blp::try_from(RawCommand::from_str("BLP 1 AL\r\n").unwrap()).unwrap();

        assert_eq!(1, blp.tr_id);
        assert!(matches!(blp.list_type, ListType::AllowList));

    }

    #[test]
    fn test_deserialize_bl() {
        let blp = Blp::try_from(RawCommand::from_str("BLP 1 BL\r\n").unwrap()).unwrap();

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

