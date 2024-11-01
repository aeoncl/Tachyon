use std::str::FromStr;

use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{MSNPCommand, MSNPPayload};

pub struct MsgServer {
    pub sender: String,
    pub display_name: String,
    pub payload: MsgPayload
}


impl MSNPCommand for MsgServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

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

impl MSNPPayload for MsgPayload {
    type Err = PayloadError;
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        todo!()
    }
    fn into_bytes(self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.into_bytes() }
        }
    }
}

