use std::fmt::Display;
use std::str::FromStr;

use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

pub struct SbsServer {
    pub tr_id: u128,
    pub status: String,
}

impl Display for SbsServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SBS {tr_id} {status}\r\n", tr_id = self.tr_id, status = self.status)
    }
}

impl TryFromRawCommand for SbsServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let status = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "status".into(), 2))?;

        Ok(Self { tr_id, status })
    }
}

impl IntoBytes for SbsServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::msnp::raw_command_parser::RawCommandParser;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::SbsServer;

    #[test]
    fn round_trips() {
        let bytes = SbsServer { tr_id: 0, status: "null".to_string() }.into_bytes();
        assert_eq!(bytes, b"SBS 0 null\r\n");

        let mut parser = RawCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = SbsServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.tr_id, 0);
        assert_eq!(parsed.status, "null");
    }
}
