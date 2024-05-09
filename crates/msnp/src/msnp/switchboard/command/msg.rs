//      0   1  2 3
// >>> MSG 231 U 91\r\npayload
// <<< ACK 231           on success
// <<< NAK 231          on failure
// The 2nd parameter is the type of ack the clients wants.
// N: ack only when the message was not received
// A + D: always send an ack
// U: never ack

use std::str::FromStr;

use strum_macros::{Display, EnumString};

use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{MSNPCommand, MSNPPayload};

pub struct MsgClient {
    tr_id: u128,
    ack_type: MsgAcknowledgment,
    payload: RawMsgPayload
}

impl MSNPCommand for MsgClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_ack_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "ack_type".into(), 1))?;
        let ack_type = MsgAcknowledgment::from_str(&raw_ack_type)?;

        //TODO RawMsgPayload from bytes
        Ok(MsgClient{
            tr_id,
            ack_type,
            payload: Default::default(),
        })
    }

    fn to_bytes(self) -> Vec<u8> {
        todo!()
    }
}


#[derive(Display, EnumString)]
pub enum MsgAcknowledgment {
    #[strum(serialize = "U")]
    NoAck,
    #[strum(serialize = "N")]
    AckOnFailure,
    #[strum(serialize = "A")]
    AckA,
    #[strum(serialize = "D")]
    AckD
}

pub struct MsgServer {
    pub sender: String,
    pub display_name: String,
    pub payload: MsgPayload
}

impl MSNPCommand for MsgServer {
    type Err = ();

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.to_bytes();
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
        let rawMsgPayload = RawMsgPayload::try_from_bytes(bytes)?;


        todo!()
    }

    fn to_bytes(self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.to_bytes() }
            _ => {
                todo!()
            }
        }
    }
}
