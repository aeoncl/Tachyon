use std::str::FromStr;

use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::display_name::DisplayName;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand};

pub struct MsgServer {
    pub sender: String,
    pub display_name: DisplayName,
    pub payload: MsgPayload
}


impl TryFromRawCommand for MsgServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let sender = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "sender".into(), 1))?;
        let raw_display_name = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "display_name".into(), 2))?;
        let display_name = DisplayName::from_str(&raw_display_name)?;

        let payload = MsgPayload::try_from_bytes(raw.payload)?;

        Ok(Self { sender, display_name, payload })
    }

}

impl IntoBytes for MsgServer {
    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();
        let mut cmd = format!("MSG {} {} {}\r\n", self.sender, self.display_name, payload.len()).into_bytes();
        cmd.append(&mut payload);
        cmd
    }
}


pub enum MsgPayload {
    Raw(RawMsgPayload),
}

impl TryFromBytes for MsgPayload {
    type Err = PayloadError;
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        Ok(Self::Raw(RawMsgPayload::try_from_bytes(bytes)?))
    }
}

impl IntoBytes for MsgPayload {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.into_bytes() }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::msnp::raw_command_parser::ServerCommandParser;
    use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::{MsgPayload, MsgServer};

    #[test]
    fn deserializes_sender_display_name_and_payload() {
        let payload = RawMsgPayload::new(MsgContentType::Profile, false);
        let mut wire = format!("MSG Hotmail Hotmail {}\r\n", payload.clone().into_bytes().len()).into_bytes();
        wire.extend_from_slice(&payload.into_bytes());

        let mut parser = ServerCommandParser::new();
        let raw = parser.parse_message(&wire).unwrap().pop().unwrap();

        let msg = MsgServer::try_from_raw(raw).unwrap();

        assert_eq!(msg.sender, "Hotmail");
        assert_eq!(msg.display_name.value(), "Hotmail");
        let MsgPayload::Raw(payload) = msg.payload else {
            panic!("expected a raw payload");
        };
        assert_eq!(payload.body.as_ref(), b"");
    }

    #[test]
    fn round_trips_a_payload_with_a_body() {
        let mut payload = RawMsgPayload::new(MsgContentType::Profile, false);
        payload.body = bytes::Bytes::from_static(b"the body");
        let mut wire = format!("MSG Hotmail Hotmail {}\r\n", payload.clone().into_bytes().len()).into_bytes();
        wire.extend_from_slice(&payload.into_bytes());

        let mut parser = ServerCommandParser::new();
        let raw = parser.parse_message(&wire).unwrap().pop().unwrap();

        let msg = MsgServer::try_from_raw(raw).unwrap();

        let MsgPayload::Raw(parsed) = msg.payload else {
            panic!("expected a raw payload");
        };
        assert_eq!(parsed.body.as_ref(), b"the body");
    }
}
