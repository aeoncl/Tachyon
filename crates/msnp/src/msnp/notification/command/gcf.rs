use std::fmt::Display;
use std::str::FromStr;

use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

pub struct GcfServer {
    pub tr_id: u128,
    pub payload: String,
}

impl Display for GcfServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GCF {tr_id} {len}\r\n{payload}", tr_id = self.tr_id, len = self.payload.len(), payload = self.payload)
    }
}

impl TryFromRawCommand for GcfServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let payload = String::from_utf8(raw.payload).map_err(|e| CommandError::from(crate::msnp::error::PayloadError::StringPayloadParsingError {
            payload: String::from_utf8_lossy(e.as_bytes()).to_string(),
            source: e.into(),
        }))?;

        Ok(Self { tr_id, payload })
    }
}

impl IntoBytes for GcfServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::msnp::raw_command_parser::ServerCommandParser;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::GcfServer;

    #[test]
    fn round_trips() {
        let gcf = GcfServer {
            tr_id: 0,
            payload: "<Policies><Policy type=\"SHIELDS\"/></Policies>".to_string(),
        };
        let bytes = gcf.into_bytes();

        let mut parser = ServerCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = GcfServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.tr_id, 0);
        assert_eq!(parsed.payload, "<Policies><Policy type=\"SHIELDS\"/></Policies>");
    }
}
