use std::fmt::Display;
use std::str::FromStr;

use anyhow::anyhow;

use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

pub struct OkCommand {
    pub operand: String,
    pub tr_id: u128
}

impl Display for OkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} OK\r\n", operand = self.operand, tr_id = self.tr_id)
    }
}

impl TryFromRawCommand for OkCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;

        let operand = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "operand".into(), 0))?;
        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let ok = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "OK".into(), 2))?;
        if ok != "OK" {
            return Err(CommandError::ArgumentParseError {
                argument: ok,
                command: raw.command,
                source: anyhow!("expected the OK acknowledgement"),
            });
        }

        Ok(Self { operand, tr_id })
    }

}

impl IntoBytes for OkCommand {

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::msnp::raw_command_parser::ServerCommandParser;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::OkCommand;

    #[test]
    fn parses_the_acknowledged_operand() {
        let mut parser = ServerCommandParser::new();
        let raw = parser.parse_message(b"ADL 7 OK\r\n").unwrap().pop().unwrap();

        let ok = OkCommand::try_from_raw(raw).unwrap();
        assert_eq!(ok.operand, "ADL");
        assert_eq!(ok.tr_id, 7);
    }

    #[test]
    fn round_trips_through_the_display_form() {
        let ok = OkCommand { operand: "UUN".to_string(), tr_id: 9 };
        let bytes = ok.into_bytes();
        assert_eq!(String::from_utf8(bytes.clone()).unwrap(), "UUN 9 OK\r\n");

        let mut parser = ServerCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = OkCommand::try_from_raw(raw).unwrap();

        assert_eq!(parsed.operand, "UUN");
        assert_eq!(parsed.tr_id, 9);
    }
}
