//      0   1  2 3
// >>> MSG 231 U 91\r\npayload
// <<< ACK 231           on success
// <<< NAK 231          on failure
// The 2nd parameter is the type of ack the clients wants.
// N: ack only when the message was not received
// A + D: always send an ack
// U: never ack

use strum_macros::Display;
use crate::shared::payload::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::SerializeMsnp;

pub struct MsgClient {

    tr_id: u128,
    ack_type: MsgAcknowledgment,
    payload: RawMsgPayload
}

#[derive(Display)]
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
