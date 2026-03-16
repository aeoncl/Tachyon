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

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
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
    fn try_from_bytes(_bytes: Vec<u8>) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl IntoBytes for MsgPayload {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.into_bytes() }
        }
    }
}

