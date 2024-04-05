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
use crate::msnp::error::CommandError;
use crate::msnp::error::CommandError::ArgumentParseError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::command::command::{get_split_part, parse_tr_id};
use crate::shared::payload::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::SerializeMsnp;

pub struct MsgClient {

    tr_id: u128,
    ack_type: MsgAcknowledgment,
    payload: RawMsgPayload
}

impl TryFrom<RawCommand> for MsgClient {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let ack_type = MsgAcknowledgment::from_str(get_split_part(2, &split, command.get_command(), "ack_type")?).map_err(|e| ArgumentParseError {
            argument: "ack_type".to_string(),
            command: command.get_command().to_string(),
            source: e.into(),
        })?;

        //TODO RawMsgPayload from bytes
        Ok(MsgClient{
            tr_id,
            ack_type,
            payload: Default::default(),
        })
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

impl SerializeMsnp for MsgServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        let mut payload = self.payload.serialize_msnp();
        let cmd = format!("MSG {} {} {}\r\n", self.sender, self.display_name, payload.len());

        let mut out = Vec::with_capacity(cmd.len()+payload.len());
        out.extend_from_slice(cmd.as_bytes());
        out.append(&mut payload);

        out
    }
}

pub enum MsgPayload {
    Raw(RawMsgPayload),
}

impl SerializeMsnp for MsgPayload {
    fn serialize_msnp(&self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.serialize_msnp() }
        }
    }
}
